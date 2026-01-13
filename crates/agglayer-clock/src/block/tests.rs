use std::{
    num::NonZeroU64,
    sync::{atomic::AtomicU64, Arc},
    time::Duration,
};

use agglayer_types::EpochNumber;
use alloy::{
    node_bindings::Anvil,
    providers::{Provider, ProviderBuilder, WsConnect},
    rpc::client::ClientBuilder,
};
use fail::FailScenario;
use futures::StreamExt;
use rstest::rstest;
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

use crate::{
    block::{BlockClockError, BlockProvider},
    BlockClock, Clock, ClockRef, Event, BROADCAST_CHANNEL_SIZE,
};

impl BlockClock<BlockProvider> {
    async fn default_for_test(ws: WsConnect, genesis_block: u64, epoch_duration: u64) -> Self {
        BlockClock::new_with_ws(
            ws,
            genesis_block,
            epoch_duration.try_into().unwrap(),
            // note: high value for compatibility with existing tests
            Duration::from_secs(90),
        )
        .await
        .expect("Failed to create BlockClock")
    }
}

#[test]
fn test_block_calculation() {
    assert_eq!(
        0,
        BlockClock::new((), 0, NonZeroU64::new(3).unwrap()).calculate_block_number(0)
    );
    assert_eq!(
        2,
        BlockClock::new((), 0, NonZeroU64::new(3).unwrap()).calculate_block_number(2)
    );
    assert_eq!(
        1,
        BlockClock::new((), 1, NonZeroU64::new(3).unwrap()).calculate_block_number(2)
    );
    assert_eq!(
        0,
        BlockClock::new((), 2, NonZeroU64::new(3).unwrap()).calculate_block_number(2)
    );
}

#[tokio::test]
async fn test_block_clock() {
    let anvil = Anvil::new().block_time(1u64).spawn();
    let ws = WsConnect::new(anvil.ws_endpoint());

    let clock = BlockClock::default_for_test(ws, 0, 3).await;

    let token = CancellationToken::new();
    let clock_ref = clock.spawn(token).await.unwrap();
    assert_eq!(clock_ref.current_epoch(), EpochNumber::ZERO);

    let mut recv = clock_ref.subscribe().unwrap();

    assert_eq!(recv.recv().await, Ok(Event::EpochEnded(EpochNumber::ZERO)));
    assert_eq!(clock_ref.current_epoch(), EpochNumber::new(1));
    assert!(clock_ref.current_block_height() >= 3);
}

#[tokio::test]
async fn test_block_clock_with_genesis() {
    let anvil = Anvil::new().block_time(1u64).spawn();
    let ws = WsConnect::new(anvil.ws_endpoint());
    tokio::time::sleep(Duration::from_secs(3)).await;
    let clock = BlockClock::default_for_test(ws, 2, 3).await;

    let token = CancellationToken::new();
    let clock_ref = clock.spawn(token).await.unwrap();
    assert_eq!(clock_ref.current_epoch(), EpochNumber::ZERO);

    let mut recv = clock_ref.subscribe().unwrap();

    assert_eq!(recv.recv().await, Ok(Event::EpochEnded(EpochNumber::ZERO)));
    assert_eq!(clock_ref.current_epoch(), EpochNumber::new(1));
    assert!(clock_ref.current_block_height() >= 3);
}

#[test_log::test(tokio::test)]
async fn test_block_clock_with_genesis_in_future() {
    let anvil = Anvil::new().block_time(1u64).spawn();
    let ws = WsConnect::new(anvil.ws_endpoint());

    let clock = BlockClock::default_for_test(ws, 10, 2).await;

    let token = CancellationToken::new();
    let clock_ref = clock.spawn(token).await.unwrap();
    assert_eq!(clock_ref.current_epoch(), EpochNumber::ZERO);

    let mut recv = clock_ref.subscribe().unwrap();

    assert_eq!(recv.recv().await, Ok(Event::EpochEnded(EpochNumber::ZERO)));
    assert_eq!(clock_ref.current_epoch(), EpochNumber::new(1));
    assert!(clock_ref.current_block_height() >= 2);
}

#[tokio::test]
async fn test_block_clock_starting_with_genesis_in_future_should_trigger_epoch_0() {
    let anvil = Anvil::new().block_time(1u64).spawn();
    let ws = WsConnect::new(anvil.ws_endpoint());
    tokio::time::sleep(Duration::from_secs(1)).await;

    let clock = BlockClock::default_for_test(ws, 0, 3).await;

    let token = CancellationToken::new();
    let clock_ref = clock.spawn(token).await.unwrap();

    let mut recv = clock_ref.subscribe().unwrap();
    assert_eq!(recv.recv().await, Ok(Event::EpochEnded(EpochNumber::ZERO)));
    assert_eq!(clock_ref.current_epoch(), EpochNumber::new(1));
    assert!(clock_ref.current_block_height() >= 3);
}

#[rstest]
#[timeout(Duration::from_secs(15))]
#[test_log::test(tokio::test)]
async fn test_block_clock_starting_with_genesis() {
    let anvil = Anvil::new().block_time(1u64).spawn();
    let ws = WsConnect::new(anvil.ws_endpoint());
    let client = ClientBuilder::default()
        .ws(ws.clone())
        .await
        .expect("Failed to connect to Anvil");
    let client: BlockProvider = ProviderBuilder::default()
        .with_recommended_fillers()
        .connect_client(client);

    let test_client = client.clone();
    let mut subscribe = test_client.subscribe_blocks().await.unwrap().into_stream();

    let clock = BlockClock::default_for_test(ws, 10, 1).await;

    let token = CancellationToken::new();
    let clock_ref = clock.spawn(token).await.unwrap();
    let mut recv = clock_ref.subscribe().unwrap();

    while let Some(block) = subscribe.next().await {
        let block_number = block.number;

        if block_number >= 11 {
            assert!(matches!(
                recv.recv().await,
                Ok(Event::EpochEnded(EpochNumber::ZERO))
            ));
            assert_eq!(clock_ref.current_epoch(), EpochNumber::new(1));
            assert!(clock_ref.current_block_height() >= 1);
            break;
        } else {
            assert!(recv.try_recv().is_err());
            assert!(clock_ref.current_block_height() == 0);
        }
    }
}

#[rstest]
#[timeout(Duration::from_secs(13))]
#[test_log::test(tokio::test)]
async fn test_block_clock_starting_with_genesis_already_passed() {
    let anvil = Anvil::new().block_time(1u64).spawn();

    tokio::time::sleep(Duration::from_secs(10)).await;
    let ws = WsConnect::new(anvil.ws_endpoint());
    let clock = BlockClock::default_for_test(ws, 0, 3).await;

    let token = CancellationToken::new();
    let clock_ref = clock.spawn(token).await.unwrap();

    let mut recv = clock_ref.subscribe().unwrap();
    assert_eq!(
        recv.recv().await,
        Ok(Event::EpochEnded(EpochNumber::new(3)))
    );
    assert_eq!(clock_ref.current_epoch(), EpochNumber::new(4));
    assert!(clock_ref.current_block_height() >= 10);
}

#[tokio::test]
async fn test_block_clock_overflow() {
    let scenario = FailScenario::setup();
    let anvil = Anvil::new().block_time(1u64).spawn();
    let ws = WsConnect::new(anvil.ws_endpoint());

    let mut clock = BlockClock::default_for_test(ws, 0, 3).await;
    let blocks = clock.block_height.clone();
    let (sender, _receiver) = broadcast::channel(BROADCAST_CHANNEL_SIZE);

    let token = CancellationToken::new();

    fail::cfg_callback(
        "block_clock::BlockClock::run::overwrite_block_number",
        move || {
            // Overflow the block height on next poll
            blocks.store(u64::MAX - 1, std::sync::atomic::Ordering::SeqCst);
        },
    )
    .unwrap();

    let (start_sender, _start_receiver) = tokio::sync::oneshot::channel();
    let handle = tokio::spawn(async move { clock.run(sender, start_sender, token).await });

    let res = tokio::time::timeout(Duration::from_secs(10), handle)
        .await
        .expect("Timeout waiting for task to finish")
        .expect("Task Join error");

    assert!(matches!(
        res,
        Err(BlockClockError::SetBlockHeight(height)) if height == u64::MAX - 1
    ));
    scenario.teardown();
}

#[tokio::test]
async fn test_block_epoch_calculation() {
    let (sender, _receive) = broadcast::channel(10);
    let block_height = Arc::new(AtomicU64::new(0));
    let block_per_epoch = Arc::new(NonZeroU64::new(300).unwrap());
    let clock_ref = ClockRef::new(sender, block_height.clone(), block_per_epoch);

    assert_eq!(clock_ref.current_epoch(), EpochNumber::ZERO);
    block_height.fetch_add(301, std::sync::atomic::Ordering::SeqCst);
    assert_eq!(clock_ref.current_epoch(), EpochNumber::new(1));
}

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(Duration::from_secs(20))]
async fn regression_block_disconnection() {
    let anvil = Anvil::new().block_time(1u64).spawn();
    let port = anvil.port();
    let ws = WsConnect::new(anvil.ws_endpoint());

    let clock =
        BlockClock::new_with_ws(ws, 0, NonZeroU64::new(3).unwrap(), Duration::from_secs(90))
            .await
            .unwrap();
    let token = CancellationToken::new();
    let clock_ref = clock.spawn(token).await.unwrap();

    assert_eq!(clock_ref.current_epoch(), EpochNumber::ZERO);

    let mut recv = clock_ref.subscribe().unwrap();

    // Assert that we read the first epoch
    assert_eq!(recv.recv().await, Ok(Event::EpochEnded(EpochNumber::ZERO)));

    // Kill & restart using the same port so we end up with the same endpoint url:
    drop(anvil);

    // Add some delay to make the reconnect fails
    tokio::time::sleep(Duration::from_secs(5)).await;

    let _anvil = Anvil::new().port(port).block_time(1u64).spawn();

    // Wait for the next epoch on existing subscription.
    assert_eq!(
        recv.recv().await,
        Ok(Event::EpochEnded(EpochNumber::new(1)))
    );
}

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(Duration::from_secs(50))]
async fn disconnection_with_timeout() {
    let anvil = Anvil::new().block_time(1u64).spawn();
    let port = anvil.port();
    let ws = WsConnect::new(anvil.ws_endpoint());

    let clock = BlockClock::default_for_test(ws, 0, 3).await;
    let token = CancellationToken::new();
    let clock_ref = clock.spawn(token.clone()).await.unwrap();

    assert_eq!(clock_ref.current_epoch(), EpochNumber::ZERO);

    let mut recv = clock_ref.subscribe().unwrap();

    // Assert that we read the first epoch
    assert_eq!(recv.recv().await, Ok(Event::EpochEnded(EpochNumber::ZERO)));

    // Kill & restart using the same port so we end up with the same endpoint url:
    drop(anvil);

    // Add some delay to make the reconnect fails. There are 10 tries spread 3
    // seconds apart each.
    tokio::time::sleep(Duration::from_secs(35)).await;

    let _anvil = Anvil::new().port(port).block_time(1u64).spawn();

    assert!(token.is_cancelled());
}

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(Duration::from_secs(20))]
async fn can_catchup_on_disconnection() {
    let scenario = FailScenario::setup();
    let anvil = Anvil::new().block_time(1u64).spawn();
    let port = anvil.port();
    let ws = WsConnect::new(anvil.ws_endpoint());

    let clock =
        BlockClock::new_with_ws(ws, 0, NonZeroU64::new(1).unwrap(), Duration::from_secs(90))
            .await
            .unwrap();
    let token = CancellationToken::new();
    let clock_ref = clock.spawn(token).await.unwrap();

    assert_eq!(clock_ref.current_epoch(), EpochNumber::ZERO);

    let mut recv = clock_ref.subscribe().unwrap();

    // Assert that we read the first epoch
    assert_eq!(recv.recv().await, Ok(Event::EpochEnded(EpochNumber::ZERO)));

    // Kill & restart using the same port so we end up with the same endpoint url:
    drop(anvil);

    // Add some delay to make the reconnect fails
    tokio::time::sleep(Duration::from_secs(5)).await;
    let _anvil = Anvil::new().port(port).block_time(1u64).spawn();
    fail::cfg(
        "block_clock::PubSubConnect::try_reconnect::add_delay",
        "sleep(5000)",
    )
    .unwrap();

    // Wait for the next epoch on existing subscription.
    assert_eq!(
        recv.recv().await,
        Ok(Event::EpochEnded(EpochNumber::new(1)))
    );

    tokio::time::timeout(Duration::from_secs(1), async move {
        assert_eq!(recv.try_recv(), Ok(Event::EpochEnded(EpochNumber::new(2))));
        assert_eq!(recv.try_recv(), Ok(Event::EpochEnded(EpochNumber::new(3))));
        assert_eq!(recv.try_recv(), Ok(Event::EpochEnded(EpochNumber::new(4))));
    })
    .await
    .expect("Timeout");
    scenario.teardown();
}

#[rstest::rstest]
// The test should still work without a lag.
#[case("off")]
// Anvil bounds its broadcast channel size to 16 by default.
// At one block per second, we need over 16 seconds for it to start lagging.
#[case("1*return(20)")]
// This one is fine during initialization but lags later on.
#[case("4*off->1*return(20)")]
// Limit the total running time.
#[timeout(Duration::from_secs(60))]
#[test_log::test(tokio::test)]
async fn skipped_blocks_are_handled(#[case] lag_cfg: &str) {
    let scenario = FailScenario::setup();

    fail::cfg("block_clock::BlockClock::recv_block::before", lag_cfg).unwrap();

    let anvil = Anvil::new().block_time(1u64).spawn();
    let ws = WsConnect::new(anvil.ws_endpoint());

    let clock = BlockClock::default_for_test(ws, 0, 3).await;
    let token = CancellationToken::new();

    let start = tokio::time::Instant::now();
    let clock_ref = clock.spawn(token.clone()).await.unwrap();
    let mut recv = clock_ref.subscribe().unwrap();

    tokio::time::sleep_until(start + Duration::from_secs(25)).await;

    let Event::EpochEnded(last_epoch) = {
        let mut last_epoch = None;
        loop {
            match recv.try_recv() {
                Ok(epoch) => last_epoch = Some(epoch),
                Err(broadcast::error::TryRecvError::Lagged(_)) => (),
                Err(broadcast::error::TryRecvError::Empty) => {
                    break last_epoch.expect("Nothing in the epoch broadcast channel")
                }
                Err(broadcast::error::TryRecvError::Closed) => {
                    panic!("Clock channel unexpectedly closed; the clock is probably dead")
                }
            }
        }
    };

    // After 25 seconds, we should be at epoch 7. This should be independent
    // of the lag as the clock should have recovered from it by this point.
    assert_eq!(last_epoch, EpochNumber::new(7));

    scenario.teardown();
}
