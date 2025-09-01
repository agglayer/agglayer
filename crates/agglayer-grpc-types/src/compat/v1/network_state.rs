use agglayer_interop::grpc::v1::FixedBytes32;
use agglayer_rpc::network_state::{
    NetworkState as AgglayerRpcNetworkState, NetworkStatus as AgglayerRpcNetworkStatus,
    NetworkType as AgglayerRpcNetworkType,
};
use agglayer_types::CertificateStatus;

use crate::node::types::v1;

impl From<AgglayerRpcNetworkStatus> for v1::NetworkStatus {
    fn from(value: AgglayerRpcNetworkStatus) -> Self {
        match value {
            AgglayerRpcNetworkStatus::Active => v1::NetworkStatus::Active,
            AgglayerRpcNetworkStatus::Syncing => v1::NetworkStatus::Syncing,
            AgglayerRpcNetworkStatus::Error => v1::NetworkStatus::Error,
        }
    }
}

impl From<AgglayerRpcNetworkType> for v1::NetworkType {
    fn from(value: AgglayerRpcNetworkType) -> Self {
        match value {
            AgglayerRpcNetworkType::Ecdsa => v1::NetworkType::Ecdsa,
            AgglayerRpcNetworkType::Generic => v1::NetworkType::Generic,
            AgglayerRpcNetworkType::MultisigOnly => v1::NetworkType::MultisigOnly,
            AgglayerRpcNetworkType::MultisigAndAggchainProof => {
                v1::NetworkType::MultisigAndAggchainProof
            }
        }
    }
}

impl From<AgglayerRpcNetworkState> for v1::NetworkState {
    fn from(value: AgglayerRpcNetworkState) -> Self {
        let network_status: v1::NetworkStatus = value.network_status.into();
        let network_type: v1::NetworkType = value.network_type.into();
        let settled_pp_root = value.settled_pp_root.map(FixedBytes32::from);
        let settled_ler = value.settled_ler.map(FixedBytes32::from);

        let settled_claim = value.settled_claim.map(|claim| v1::SettledClaim {
            global_index: Some(FixedBytes32::from(claim.global_index)),
            bridge_exit_hash: Some(FixedBytes32::from(claim.bridge_exit_hash)),
        });

        let latest_pending_status = value.latest_pending_status.map(|status| match status {
            CertificateStatus::Pending => 1,
            CertificateStatus::Proven => 2,
            CertificateStatus::Candidate => 3,
            CertificateStatus::InError { .. } => 4,
            CertificateStatus::Settled => 5,
        });

        let latest_pending_error =
            value
                .latest_pending_error
                .map(|error| v1::CertificateStatusError {
                    message: error.to_string().into_bytes().into(),
                });

        v1::NetworkState {
            network_status: network_status as i32,
            network_type: network_type as i32,
            network_id: value.network_id.into(),
            settled_height: value.settled_height.map(|value| value.as_u64()),
            settled_certificate_id: value.settled_certificate_id.map(|id| id.into()),
            settled_pp_root,
            settled_ler,
            settled_let_leaf_count: value.settled_let_leaf_count,
            settled_claim,
            latest_pending_height: value.latest_pending_height,
            latest_pending_status,
            latest_pending_error,
            latest_epoch_with_settlement: value.latest_epoch_with_settlement,
        }
    }
}
