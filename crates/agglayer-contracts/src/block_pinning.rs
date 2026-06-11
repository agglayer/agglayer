//! Resolve the L1 block at which to evaluate a view call so that it observes
//! the state immediately *before* a given transaction was included.
//!
//! Stateful aggchain contracts (e.g. `AggchainFEP`) mutate their on-chain state
//! as certificates settle, so a view call such as `getAggchainHash` or
//! `rollupIDToRollupDataV2` must sometimes be pinned to the pre-settlement
//! block to observe the value a now-settled certificate was checked against.
//!
//! Both [`crate::aggchain`] and [`crate::rollup`] share this resolution; they
//! differ only in how they treat a transaction that did not successfully
//! advance the state (not mined, reverted, or whose receipt could not be
//! fetched). Each caller decides that policy by mapping [`UnresolvedBlock`]:
//! the aggchain path falls back to `latest`, while the rollup path surfaces a
//! typed error.

use alloy::{eips::BlockId, primitives::TxHash, providers::Provider};

/// Reason a pre-transaction block could not be resolved from a receipt.
///
/// Returned by [`block_before_tx`] when the transaction did not successfully
/// advance L1 state. Callers decide whether this is a hard error or a cue to
/// fall back to the current (`latest`) state.
#[derive(Debug)]
pub(crate) enum UnresolvedBlock {
    /// The transaction receipt could not be fetched from L1.
    FetchFailed(eyre::Error),
    /// The transaction has no receipt yet, i.e. it is not mined.
    NotMined,
    /// The transaction is mined but reverted.
    Reverted,
}

/// Resolve the block immediately preceding `tx_hash`'s inclusion block.
///
/// On success returns the block before inclusion (saturating at zero), or
/// [`BlockId::latest`] when the inclusion block number is unknown. Returns an
/// [`UnresolvedBlock`] when the receipt cannot be fetched, the transaction is
/// not yet mined, or it reverted.
pub(crate) async fn block_before_tx<P: Provider>(
    rpc: &P,
    tx_hash: TxHash,
) -> Result<BlockId, UnresolvedBlock> {
    let receipt = rpc
        .get_transaction_receipt(tx_hash)
        .await
        .map_err(|err| UnresolvedBlock::FetchFailed(err.into()))?
        .map(|receipt| (receipt.status(), receipt.block_number));

    block_before_inclusion(receipt)
}

/// Resolve the query block from a transaction's `(succeeded, inclusion_block)`
/// receipt projection, or `None` when the transaction has no receipt.
///
/// A successful transaction included in block `n` resolves to block `n - 1`
/// (saturating at zero); a successful transaction with an unknown inclusion
/// block resolves to [`BlockId::latest`]. A reverted or unmined transaction
/// yields the corresponding [`UnresolvedBlock`].
fn block_before_inclusion(
    receipt: Option<(bool, Option<u64>)>,
) -> Result<BlockId, UnresolvedBlock> {
    match receipt {
        Some((true, Some(block))) => Ok(BlockId::number(block.saturating_sub(1))),
        Some((true, None)) => Ok(BlockId::latest()),
        Some((false, _)) => Err(UnresolvedBlock::Reverted),
        None => Err(UnresolvedBlock::NotMined),
    }
}

#[cfg(test)]
mod tests {
    use alloy::eips::BlockId;

    use super::{block_before_inclusion, UnresolvedBlock};

    #[test]
    fn successful_receipt_resolves_to_preceding_block() {
        assert_eq!(
            block_before_inclusion(Some((true, Some(100)))).unwrap(),
            BlockId::number(99),
        );
    }

    #[test]
    fn inclusion_in_genesis_block_saturates_at_zero() {
        assert_eq!(
            block_before_inclusion(Some((true, Some(0)))).unwrap(),
            BlockId::number(0),
        );
    }

    #[test]
    fn successful_receipt_without_block_resolves_to_latest() {
        assert_eq!(
            block_before_inclusion(Some((true, None))).unwrap(),
            BlockId::latest(),
        );
    }

    #[test]
    fn reverted_transaction_is_unresolved() {
        assert!(matches!(
            block_before_inclusion(Some((false, Some(100)))),
            Err(UnresolvedBlock::Reverted),
        ));
    }

    #[test]
    fn reverted_transaction_without_block_is_unresolved() {
        assert!(matches!(
            block_before_inclusion(Some((false, None))),
            Err(UnresolvedBlock::Reverted),
        ));
    }

    #[test]
    fn missing_receipt_is_not_mined() {
        assert!(matches!(
            block_before_inclusion(None),
            Err(UnresolvedBlock::NotMined),
        ));
    }
}
