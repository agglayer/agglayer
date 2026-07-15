use std::time::Duration;

use super::*;

#[tokio::test(start_paused = true)]
async fn same_wallet_lock_blocks_until_guard_drops() {
    let locks = Arc::new(WalletNonceLocks::default());
    let wallet = Address::from([0xAB; 20]);

    let guard = locks.lock(wallet).await;

    let contender = tokio::spawn({
        let locks = locks.clone();
        async move { locks.lock(wallet).await }
    });

    // The second acquisition must not complete while the guard is held.
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert!(
        !contender.is_finished(),
        "lock must be exclusive per wallet"
    );

    drop(guard);

    tokio::time::timeout(Duration::from_secs(5), contender)
        .await
        .expect("contender should acquire the lock after the guard drops")
        .expect("contender task should not panic");
}

#[tokio::test]
async fn different_wallets_lock_independently() {
    let locks = WalletNonceLocks::default();
    let wallet_a = Address::from([0xAA; 20]);
    let wallet_b = Address::from([0xBB; 20]);

    let _guard_a = locks.lock(wallet_a).await;

    // Must complete immediately: wallet B has its own lock.
    tokio::time::timeout(Duration::from_secs(5), locks.lock(wallet_b))
        .await
        .expect("distinct wallets must not contend");
}
