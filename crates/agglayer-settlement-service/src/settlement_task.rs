use std::{sync::OnceLock, time::SystemTime};

use agglayer_config::Multiplier;
use alloy::primitives::{Address, BlockHash, Bytes, U128, U256};
use tokio::sync::mpsc;
use ulid::Ulid;

#[derive(Debug)]
pub struct SettlementJob {
    contract_address: Address,
    calldata: Bytes,
    eth_value: U256,

    num_confirmations: u32,
    gas_limit: U128,
    max_fee_per_gas_ceiling: U128,
    max_fee_per_gas_floor: U128,
    max_fee_per_gas_multiplier: Multiplier,
    max_priority_fee_per_gas_ceiling: U128,
    max_priority_fee_per_gas_floor: U128,
    max_priority_fee_per_gas_multiplier: Multiplier,
}

#[derive(Clone)]
pub enum SettlementJobResult {
    ClientError(ClientError),
    ContractCallResult(ContractCallResult),
    ReorganizedResult(ReorganizedResult),
}

#[derive(Clone)]
pub struct ClientError {
    pub kind: ClientErrorType,
    pub message: String,
}

#[derive(Clone)]
pub enum ClientErrorType {
    Transient,
    Permanent,
}

#[derive(Clone)]
pub struct ContractCallResult {
    pub outcome: ContractCallOutcome,
    pub metadata: Bytes,
    pub block_hash: BlockHash,
}

#[derive(Clone)]
pub enum ContractCallOutcome {
    Success,
    Revert,
}

#[derive(Clone)]
pub struct ReorganizedResult {
    pub reorg_detection_time: SystemTime,
    pub previous_result: Box<SettlementJobResult>,
}

pub enum StoredSettlementJob {
    Pending(SettlementTask),
    Completed(SettlementJob, SettlementJobResult),
}

pub enum TaskAdminCommand {
    Abort,
    ReloadAndRestart,
}

pub struct SettlementTask {
    id: Ulid,
    job: SettlementJob,
    admin_commands: mpsc::Receiver<TaskAdminCommand>,
}

static ID_GENERATOR: OnceLock<std::sync::Mutex<ulid::Generator>> = OnceLock::new();

impl SettlementTask {
    pub async fn create(
        job: SettlementJob,
        admin_commands: mpsc::Receiver<TaskAdminCommand>,
    ) -> eyre::Result<(Ulid, Self)> {
        let id = loop {
            if let Ok(id) = ID_GENERATOR
                .get_or_init(|| std::sync::Mutex::new(ulid::Generator::new()))
                .lock()
                .unwrap()
                .generate()
            {
                break id;
            }
            tokio::time::sleep(std::time::Duration::from_micros(100)).await;
        };
        let this = Self {
            id,
            job,
            admin_commands,
        };
        // TODO: write settlement job data to rocksdb
        Ok((id, this))
    }

    pub async fn load(
        _id: Ulid,
        _admin_commands: mpsc::Receiver<TaskAdminCommand>,
    ) -> eyre::Result<StoredSettlementJob> {
        // TODO: load settlement job data from rocksdb
        todo!()
    }

    pub async fn run(&mut self) -> SettlementJobResult {
        // TODO: see https://app.excalidraw.com/s/65UEf35l1DW/7LG2MXrRiQX ; starting from "read all previous settlement attempts"
        todo!()
    }
}
