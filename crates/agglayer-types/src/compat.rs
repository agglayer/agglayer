use serde::{Deserialize, Serialize};

use super::{AggchainData, BridgeExit, Digest, Height, ImportedBridgeExit, Metadata, NetworkId};

/// Certificate type used for deserialization.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Certificate {
    pub network_id: NetworkId,
    pub height: Height,
    pub prev_local_exit_root: Digest,
    pub new_local_exit_root: Digest,
    pub bridge_exits: Vec<BridgeExit>,
    pub imported_bridge_exits: Vec<ImportedBridgeExit>,
    pub metadata: Metadata,
    #[serde(flatten)]
    pub aggchain_data: AggchainData,
    #[serde(default)]
    pub custom_chain_data: Vec<u8>,
    #[serde(default)]
    pub l1_info_tree_leaf_count: Option<u32>,
}

impl From<Certificate> for super::Certificate {
    fn from(certificate: Certificate) -> Self {
        let Certificate {
            network_id,
            height,
            prev_local_exit_root,
            new_local_exit_root,
            bridge_exits,
            imported_bridge_exits,
            metadata,
            aggchain_data,
            custom_chain_data,
            l1_info_tree_leaf_count,
        } = certificate;

        let l1_info_tree_leaf_count = l1_info_tree_leaf_count.unwrap_or_else(|| {
            imported_bridge_exits
                .iter()
                .map(|ibe| ibe.l1_leaf_index() + 1)
                .max()
                .unwrap_or(0)
        });

        super::Certificate {
            network_id,
            height,
            prev_local_exit_root,
            new_local_exit_root,
            bridge_exits,
            imported_bridge_exits,
            metadata,
            aggchain_data,
            custom_chain_data,
            l1_info_tree_leaf_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use pessimistic_proof_test_suite::event_data::load_json_data_file;

    #[test]
    fn explicit_and_implicit_leaf_count_agree() {
        let explicit = load_json_data_file::<crate::Certificate>("cert_h0_leafcount.json");
        let implicit = load_json_data_file::<crate::Certificate>("cert_h0.json");

        assert_eq!(
            explicit.l1_info_tree_leaf_count,
            implicit.l1_info_tree_leaf_count,
        );

        // Note using debug output since we have no `Eq` on certificate
        assert_eq!(format!("{explicit:?}"), format!("{implicit:?}"));
    }

    #[test]
    fn explicit_leaf_count() {
        let cert = load_json_data_file::<crate::Certificate>("cert_h0_leafcount99k.json");
        assert_eq!(cert.l1_info_tree_leaf_count, 99_000);
    }
}
