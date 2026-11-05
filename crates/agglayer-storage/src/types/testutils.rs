use super::generated::agglayer::storage::v0::{
    settlement_attempt_result, Address, AttemptSequenceNumber, BlockHash, BlockNumber,
    ContractCallMetadata, ContractCallOutcome, ContractCallResult, Nonce, SettlementAttemptResult,
    SettlementJobResult, TxHash,
};

fn contract_call_for_test(seed: u8, outcome: ContractCallOutcome) -> ContractCallResult {
    ContractCallResult {
        outcome: outcome as i32,
        metadata: Some(ContractCallMetadata {
            metadata: vec![seed, seed.wrapping_add(1)].into(),
        }),
        block_hash: Some(BlockHash {
            hash: vec![seed; 32].into(),
        }),
        block_number: Some(BlockNumber {
            number: seed as u64 + 100,
        }),
        tx_hash: Some(TxHash {
            hash: vec![seed.wrapping_add(2); 32].into(),
        }),
    }
}

impl SettlementAttemptResult {
    pub fn contract_call_success_for_test(seed: u8) -> Self {
        Self {
            result: Some(settlement_attempt_result::Result::ContractCallResult(
                contract_call_for_test(seed, ContractCallOutcome::Success),
            )),
        }
    }
}

impl SettlementJobResult {
    pub fn contract_call_success_for_test(seed: u8) -> Self {
        Self::for_test(seed, ContractCallOutcome::Success)
    }

    /// A terminal result whose on-chain call reverted.
    pub fn contract_call_revert_for_test(seed: u8) -> Self {
        Self::for_test(seed, ContractCallOutcome::Revert)
    }

    fn for_test(seed: u8, outcome: ContractCallOutcome) -> Self {
        Self {
            wallet: Some(Address {
                address: vec![seed.wrapping_add(3); 20].into(),
            }),
            nonce: Some(Nonce {
                nonce: seed as u64 + 200,
            }),
            attempt_number: Some(AttemptSequenceNumber {
                number: seed as u64 + 300,
            }),
            contract_call_result: Some(contract_call_for_test(seed, outcome)),
        }
    }
}
