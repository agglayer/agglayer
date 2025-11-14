use agglayer_interop::grpc::v1::FixedBytes32;
use agglayer_types::{
    CertificateStatus as AgglayerCertificateStatus, NetworkInfo as AgglayerRpcNetworkInfo,
    NetworkStatus as AgglayerRpcNetworkStatus, NetworkType as AgglayerRpcNetworkType,
};

use crate::node::types::v1;

impl From<AgglayerRpcNetworkStatus> for v1::NetworkStatus {
    fn from(value: AgglayerRpcNetworkStatus) -> Self {
        match value {
            AgglayerRpcNetworkStatus::Unknown => v1::NetworkStatus::Unspecified,
            AgglayerRpcNetworkStatus::Active => v1::NetworkStatus::Active,
            AgglayerRpcNetworkStatus::Syncing => v1::NetworkStatus::Syncing,
            AgglayerRpcNetworkStatus::Error => v1::NetworkStatus::Error,
            AgglayerRpcNetworkStatus::Disabled => v1::NetworkStatus::Disabled,
        }
    }
}

impl From<AgglayerRpcNetworkType> for v1::NetworkType {
    fn from(value: AgglayerRpcNetworkType) -> Self {
        match value {
            AgglayerRpcNetworkType::Unspecified => v1::NetworkType::Unspecified,
            AgglayerRpcNetworkType::Ecdsa => v1::NetworkType::Ecdsa,
            AgglayerRpcNetworkType::Generic => v1::NetworkType::Generic,
            AgglayerRpcNetworkType::MultisigOnly => v1::NetworkType::MultisigOnly,
            AgglayerRpcNetworkType::MultisigAndAggchainProof => {
                v1::NetworkType::MultisigAndAggchainProof
            }
        }
    }
}

impl From<AgglayerCertificateStatus> for v1::CertificateStatus {
    fn from(value: AgglayerCertificateStatus) -> Self {
        match value {
            AgglayerCertificateStatus::Pending => v1::CertificateStatus::Pending,
            AgglayerCertificateStatus::Proven => v1::CertificateStatus::Proven,
            AgglayerCertificateStatus::Candidate => v1::CertificateStatus::Candidate,
            AgglayerCertificateStatus::InError { .. } => v1::CertificateStatus::InError,
            AgglayerCertificateStatus::Settled => v1::CertificateStatus::Settled,
        }
    }
}

impl From<AgglayerRpcNetworkInfo> for v1::NetworkInfo {
    fn from(value: AgglayerRpcNetworkInfo) -> Self {
        let network_status: v1::NetworkStatus = value.network_status.into();
        let network_type: v1::NetworkType = value.network_type.into();
        let settled_pp_root = value.settled_pp_root.map(FixedBytes32::from);
        let settled_ler = value.settled_ler.map(FixedBytes32::from);

        let settled_claim = value.settled_claim.map(|claim| v1::SettledClaim {
            global_index: Some(FixedBytes32::from(claim.global_index)),
            bridge_exit_hash: Some(FixedBytes32::from(claim.bridge_exit_hash)),
        });

        let latest_pending_status: Option<v1::CertificateStatus> =
            value.latest_pending_status.map(v1::CertificateStatus::from);

        let latest_pending_error =
            value
                .latest_pending_error
                .map(|error| v1::CertificateStatusError {
                    message: error.to_string().into_bytes().into(),
                });

        v1::NetworkInfo {
            network_status: network_status as i32,
            network_type: network_type as i32,
            network_id: value.network_id.into(),
            settled_height: value.settled_height.map(|value| value.as_u64()),
            settled_certificate_id: value.settled_certificate_id.map(|id| id.into()),
            settled_pp_root,
            settled_ler,
            settled_let_leaf_count: value.settled_let_leaf_count,
            settled_claim,
            latest_pending_height: value.latest_pending_height.map(|value| value.as_u64()),
            latest_pending_status: latest_pending_status.map(|status| status as i32),
            latest_pending_error,
            latest_epoch_with_settlement: value.latest_epoch_with_settlement,
        }
    }
}
