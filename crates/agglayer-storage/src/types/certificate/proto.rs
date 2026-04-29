use super::*;

macro_rules! required_field {
    ($from:expr, $field:ident) => {
        $from
            .$field
            .ok_or(CertificateConversionError::MissingField(stringify!($field)))
    };
    ($value:expr, $field:expr $(,)?) => {
        $value.ok_or(CertificateConversionError::MissingField($field))
    };
}

macro_rules! proto_digest_bytes32 {
    ($proto_ty:ident, $field_name:literal) => {
        impl TryFrom<proto::$proto_ty> for Digest {
            type Error = CertificateConversionError;

            fn try_from(value: proto::$proto_ty) -> Result<Self, Self::Error> {
                Ok(Self::from(expect_bytes::<32>(
                    value.value.as_ref(),
                    $field_name,
                )?))
            }
        }

        impl From<Digest> for proto::$proto_ty {
            fn from(value: Digest) -> Self {
                Self {
                    value: value.0.to_vec().into(),
                }
            }
        }
    };
}

impl From<Signature> for proto::Signature {
    fn from(value: Signature) -> Self {
        Self {
            value: value.as_bytes().to_vec().into(),
        }
    }
}

fn parse_address(value: proto::Address) -> Result<Address, CertificateConversionError> {
    Ok(Address::from(expect_bytes::<20>(
        value.address.as_ref(),
        "address",
    )?))
}

proto_digest_bytes32!(MerkleRoot, "merkle_root");
proto_digest_bytes32!(GlobalExitRoot, "global_exit_root");
proto_digest_bytes32!(LocalExitRoot, "local_exit_root");
proto_digest_bytes32!(PessimisticRoot, "pessimistic_root");
proto_digest_bytes32!(RollupExitRoot, "rollup_exit_root");
proto_digest_bytes32!(MainnetExitRoot, "mainnet_exit_root");
proto_digest_bytes32!(L1InfoRoot, "l1_info_root");
proto_digest_bytes32!(AggchainHash, "aggchain_hash");
proto_digest_bytes32!(Metadata, "metadata");
proto_digest_bytes32!(CommitImportedBridgeExits, "commit_imported_bridge_exits");
proto_digest_bytes32!(AggchainParams, "aggchain_params");

impl TryFrom<proto::BlockHash> for Digest {
    type Error = CertificateConversionError;

    fn try_from(value: proto::BlockHash) -> Result<Self, Self::Error> {
        Ok(Self::from(expect_bytes::<32>(
            value.hash.as_ref(),
            "block_hash",
        )?))
    }
}

impl From<Digest> for proto::BlockHash {
    fn from(value: Digest) -> Self {
        Self {
            hash: value.0.to_vec().into(),
        }
    }
}

impl TryFrom<proto::LocalExitRoot> for LocalExitRoot {
    type Error = CertificateConversionError;

    fn try_from(value: proto::LocalExitRoot) -> Result<Self, Self::Error> {
        Ok(Self::from(Digest::try_from(value)?))
    }
}

impl From<LocalExitRoot> for proto::LocalExitRoot {
    fn from(value: LocalExitRoot) -> Self {
        Digest::from(value).into()
    }
}

impl TryFrom<proto::Metadata> for Metadata {
    type Error = CertificateConversionError;

    fn try_from(value: proto::Metadata) -> Result<Self, Self::Error> {
        Ok(Self::new(Digest::try_from(value)?))
    }
}

impl From<Metadata> for proto::Metadata {
    fn from(value: Metadata) -> Self {
        (*value).into()
    }
}

impl TryFrom<proto::Amount> for U256 {
    type Error = CertificateConversionError;

    fn try_from(value: proto::Amount) -> Result<Self, Self::Error> {
        Ok(Self::from_be_bytes(expect_bytes::<32>(
            value.value.as_ref(),
            "amount",
        )?))
    }
}

impl From<U256> for proto::Amount {
    fn from(value: U256) -> Self {
        Self {
            value: value.to_be_bytes::<32>().to_vec().into(),
        }
    }
}

impl TryFrom<proto::TokenInfo> for TokenInfo {
    type Error = CertificateConversionError;

    fn try_from(value: proto::TokenInfo) -> Result<Self, Self::Error> {
        Ok(Self {
            origin_network: NetworkId::new(value.origin_network),
            origin_token_address: parse_address(required_field!(value, origin_token_address)?)?,
        })
    }
}

impl From<TokenInfo> for proto::TokenInfo {
    fn from(value: TokenInfo) -> Self {
        Self {
            origin_network: value.origin_network.to_u32(),
            origin_token_address: Some(value.origin_token_address.into()),
        }
    }
}

impl TryFrom<proto::BridgeExit> for BridgeExit {
    type Error = CertificateConversionError;

    fn try_from(value: proto::BridgeExit) -> Result<Self, Self::Error> {
        Ok(Self {
            leaf_type: match value.leaf_type {
                x if x == proto::LeafType::Transfer as i32 => LeafType::Transfer,
                x if x == proto::LeafType::Message as i32 => LeafType::Message,
                other => {
                    return Err(CertificateConversionError::InvalidData {
                        field: "leaf_type",
                        reason: format!("unsupported leaf type `{other}`"),
                    });
                }
            },
            token_info: required_field!(value, token_info)?.try_into()?,
            dest_network: NetworkId::new(value.dest_network),
            dest_address: parse_address(required_field!(value, dest_address)?)?,
            amount: required_field!(value, amount)?.try_into()?,
            metadata: value.metadata.map(TryInto::try_into).transpose()?,
        })
    }
}

impl From<BridgeExit> for proto::BridgeExit {
    fn from(value: BridgeExit) -> Self {
        Self {
            leaf_type: match value.leaf_type {
                LeafType::Transfer => proto::LeafType::Transfer as i32,
                LeafType::Message => proto::LeafType::Message as i32,
            },
            token_info: Some(value.token_info.into()),
            dest_network: value.dest_network.to_u32(),
            dest_address: Some(value.dest_address.into()),
            amount: Some(value.amount.into()),
            metadata: value.metadata.map(Into::into),
        }
    }
}

impl TryFrom<proto::MerkleProof> for MerkleProof {
    type Error = CertificateConversionError;

    fn try_from(value: proto::MerkleProof) -> Result<Self, Self::Error> {
        if value.siblings.len() != 32 {
            return Err(CertificateConversionError::InvalidData {
                field: "siblings",
                reason: format!("expected 32 siblings, got {}", value.siblings.len()),
            });
        }

        let siblings: Vec<Digest> = value
            .siblings
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()?;

        Ok(Self::new(
            required_field!(value, root)?.try_into()?,
            siblings.try_into().unwrap(),
        ))
    }
}

impl From<MerkleProof> for proto::MerkleProof {
    fn from(value: MerkleProof) -> Self {
        Self {
            root: Some(value.root.into()),
            siblings: value.siblings().iter().copied().map(Into::into).collect(),
        }
    }
}

impl TryFrom<proto::L1InfoTreeLeaf> for L1InfoTreeLeafInner {
    type Error = CertificateConversionError;

    fn try_from(value: proto::L1InfoTreeLeaf) -> Result<Self, Self::Error> {
        Ok(Self {
            global_exit_root: required_field!(value, global_exit_root)?.try_into()?,
            block_hash: required_field!(value, block_hash)?.try_into()?,
            timestamp: value.timestamp,
        })
    }
}

impl From<L1InfoTreeLeafInner> for proto::L1InfoTreeLeaf {
    fn from(value: L1InfoTreeLeafInner) -> Self {
        Self {
            global_exit_root: Some(value.global_exit_root.into()),
            block_hash: Some(value.block_hash.into()),
            timestamp: value.timestamp,
        }
    }
}

impl TryFrom<proto::L1InfoTreeLeafWithContext> for L1InfoTreeLeaf {
    type Error = CertificateConversionError;

    fn try_from(value: proto::L1InfoTreeLeafWithContext) -> Result<Self, Self::Error> {
        Ok(Self {
            l1_info_tree_index: value.l1_info_tree_index,
            rer: required_field!(value, rer)?.try_into()?,
            mer: required_field!(value, mer)?.try_into()?,
            inner: required_field!(value, inner)?.try_into()?,
        })
    }
}

impl From<L1InfoTreeLeaf> for proto::L1InfoTreeLeafWithContext {
    fn from(value: L1InfoTreeLeaf) -> Self {
        Self {
            l1_info_tree_index: value.l1_info_tree_index,
            rer: Some(value.rer.into()),
            mer: Some(value.mer.into()),
            inner: Some(value.inner.into()),
        }
    }
}

impl TryFrom<proto::ClaimFromMainnet> for ClaimFromMainnet {
    type Error = CertificateConversionError;

    fn try_from(value: proto::ClaimFromMainnet) -> Result<Self, Self::Error> {
        Ok(Self {
            proof_leaf_mer: required_field!(value, proof_leaf_mer)?.try_into()?,
            proof_ger_l1root: required_field!(value, proof_ger_l1root)?.try_into()?,
            l1_leaf: required_field!(value, l1_leaf)?.try_into()?,
        })
    }
}

impl From<ClaimFromMainnet> for proto::ClaimFromMainnet {
    fn from(value: ClaimFromMainnet) -> Self {
        Self {
            proof_leaf_mer: Some(value.proof_leaf_mer.into()),
            proof_ger_l1root: Some(value.proof_ger_l1root.into()),
            l1_leaf: Some(value.l1_leaf.into()),
        }
    }
}

impl TryFrom<proto::ClaimFromRollup> for ClaimFromRollup {
    type Error = CertificateConversionError;

    fn try_from(value: proto::ClaimFromRollup) -> Result<Self, Self::Error> {
        Ok(Self {
            proof_leaf_ler: required_field!(value, proof_leaf_ler)?.try_into()?,
            proof_ler_rer: required_field!(value, proof_ler_rer)?.try_into()?,
            proof_ger_l1root: required_field!(value, proof_ger_l1root)?.try_into()?,
            l1_leaf: required_field!(value, l1_leaf)?.try_into()?,
        })
    }
}

impl From<ClaimFromRollup> for proto::ClaimFromRollup {
    fn from(value: ClaimFromRollup) -> Self {
        Self {
            proof_leaf_ler: Some(value.proof_leaf_ler.into()),
            proof_ler_rer: Some(value.proof_ler_rer.into()),
            proof_ger_l1root: Some(value.proof_ger_l1root.into()),
            l1_leaf: Some(value.l1_leaf.into()),
        }
    }
}

impl TryFrom<proto::imported_bridge_exit::Claim> for Claim {
    type Error = CertificateConversionError;

    fn try_from(value: proto::imported_bridge_exit::Claim) -> Result<Self, Self::Error> {
        Ok(match value {
            proto::imported_bridge_exit::Claim::Mainnet(value) => {
                Claim::Mainnet(Box::new(value.try_into()?))
            }
            proto::imported_bridge_exit::Claim::Rollup(value) => {
                Claim::Rollup(Box::new(value.try_into()?))
            }
        })
    }
}

impl From<Claim> for proto::imported_bridge_exit::Claim {
    fn from(value: Claim) -> Self {
        match value {
            Claim::Mainnet(value) => Self::Mainnet((*value).into()),
            Claim::Rollup(value) => Self::Rollup((*value).into()),
        }
    }
}

impl TryFrom<proto::GlobalIndex> for GlobalIndex {
    type Error = CertificateConversionError;

    fn try_from(value: proto::GlobalIndex) -> Result<Self, Self::Error> {
        GlobalIndex::from_u256(U256::from_be_bytes(expect_bytes::<32>(
            value.value.as_ref(),
            "global_index",
        )?))
        .map_err(|err| CertificateConversionError::InvalidData {
            field: "global_index",
            reason: err.to_string(),
        })
    }
}

impl From<GlobalIndex> for proto::GlobalIndex {
    fn from(value: GlobalIndex) -> Self {
        Self {
            value: value.into_u256().to_be_bytes::<32>().to_vec().into(),
        }
    }
}

impl TryFrom<proto::ImportedBridgeExit> for ImportedBridgeExit {
    type Error = CertificateConversionError;

    fn try_from(value: proto::ImportedBridgeExit) -> Result<Self, Self::Error> {
        Ok(Self {
            bridge_exit: required_field!(value, bridge_exit)?.try_into()?,
            claim_data: required_field!(value, claim)?.try_into()?,
            global_index: required_field!(value, global_index)?.try_into()?,
        })
    }
}

impl From<ImportedBridgeExit> for proto::ImportedBridgeExit {
    fn from(value: ImportedBridgeExit) -> Self {
        Self {
            bridge_exit: Some(value.bridge_exit.into()),
            global_index: Some(value.global_index.into()),
            claim: Some(value.claim_data.into()),
        }
    }
}

impl TryFrom<proto::AggchainProofPublicValues> for AggchainProofPublicValues {
    type Error = CertificateConversionError;

    fn try_from(value: proto::AggchainProofPublicValues) -> Result<Self, Self::Error> {
        Ok(Self {
            prev_local_exit_root: required_field!(value, prev_local_exit_root)?.try_into()?,
            new_local_exit_root: required_field!(value, new_local_exit_root)?.try_into()?,
            l1_info_root: required_field!(value, l1_info_root)?.try_into()?,
            origin_network: NetworkId::new(value.origin_network),
            commit_imported_bridge_exits: required_field!(
                value.commit_imported_bridge_exits,
                "commit_imported_bridge_exits",
            )?
            .try_into()?,
            aggchain_params: required_field!(value, aggchain_params)?.try_into()?,
        })
    }
}

impl From<AggchainProofPublicValues> for proto::AggchainProofPublicValues {
    fn from(value: AggchainProofPublicValues) -> Self {
        Self {
            prev_local_exit_root: Some(value.prev_local_exit_root.into()),
            new_local_exit_root: Some(value.new_local_exit_root.into()),
            l1_info_root: Some(value.l1_info_root.into()),
            origin_network: value.origin_network.to_u32(),
            commit_imported_bridge_exits: Some(value.commit_imported_bridge_exits.into()),
            aggchain_params: Some(value.aggchain_params.into()),
        }
    }
}

impl TryFrom<proto::MultisigPayload> for MultisigPayload {
    type Error = CertificateConversionError;

    fn try_from(value: proto::MultisigPayload) -> Result<Self, Self::Error> {
        Ok(Self(
            value
                .signatures
                .into_iter()
                .map(|entry| {
                    entry
                        .signature
                        .map(|sig| parse_signature(sig, "multisig.signature"))
                        .transpose()
                })
                .collect::<Result<_, _>>()?,
        ))
    }
}

impl From<MultisigPayload> for proto::MultisigPayload {
    fn from(value: MultisigPayload) -> Self {
        Self {
            signatures: value
                .0
                .into_iter()
                .map(|signature| proto::MultisigEntry {
                    signature: signature.map(Into::into),
                })
                .collect(),
        }
    }
}

impl TryFrom<proto::AggchainData> for AggchainData {
    type Error = CertificateConversionError;

    fn try_from(value: proto::AggchainData) -> Result<Self, Self::Error> {
        Ok(match required_field!(value, data)? {
            proto::aggchain_data::Data::Ecdsa(signature) => Self::ECDSA {
                signature: parse_signature(signature, "aggchain_data.ecdsa")?,
            },
            proto::aggchain_data::Data::Generic(generic) => Self::Generic {
                proof: Proof::try_from(required_field!(
                    generic.proof,
                    "aggchain_data.generic.proof"
                )?)?,
                aggchain_params: required_field!(
                    generic.aggchain_params,
                    "aggchain_data.generic.aggchain_params",
                )?
                .try_into()?,
                signature: generic
                    .signature
                    .map(|signature| {
                        parse_signature(signature, "aggchain_data.generic.signature").map(Box::new)
                    })
                    .transpose()?,
                public_values: generic
                    .public_values
                    .map(TryInto::try_into)
                    .transpose()?
                    .map(Box::new),
            },
            proto::aggchain_data::Data::MultisigOnly(multisig_only) => Self::MultisigOnly {
                multisig: required_field!(
                    multisig_only.multisig,
                    "aggchain_data.multisig_only.multisig"
                )?
                .try_into()?,
            },
            proto::aggchain_data::Data::MultisigAndAggchainProof(multisig_and_proof) => {
                Self::MultisigAndAggchainProof {
                    multisig: required_field!(
                        multisig_and_proof.multisig,
                        "aggchain_data.multisig_and_aggchain_proof.multisig",
                    )?
                    .try_into()?,
                    aggchain_proof: AggchainProof {
                        proof: Proof::try_from(required_field!(
                            multisig_and_proof.proof,
                            "aggchain_data.multisig_and_aggchain_proof.proof",
                        )?)?,
                        aggchain_params: required_field!(
                            multisig_and_proof.aggchain_params,
                            "aggchain_data.multisig_and_aggchain_proof.aggchain_params",
                        )?
                        .try_into()?,
                        public_values: multisig_and_proof
                            .public_values
                            .map(TryInto::try_into)
                            .transpose()?
                            .map(Box::new),
                    },
                }
            }
        })
    }
}

impl TryFrom<&AggchainData> for proto::AggchainData {
    type Error = CertificateConversionError;

    fn try_from(value: &AggchainData) -> Result<Self, Self::Error> {
        Ok(Self {
            data: Some(match value {
                AggchainData::ECDSA { signature } => {
                    proto::aggchain_data::Data::Ecdsa((*signature).into())
                }
                AggchainData::Generic {
                    proof,
                    aggchain_params,
                    signature,
                    public_values,
                } => proto::aggchain_data::Data::Generic(proto::Generic {
                    proof: Some(proto::Proof::try_from(proof)?),
                    aggchain_params: Some((*aggchain_params).into()),
                    signature: signature.as_ref().map(|signature| (**signature).into()),
                    public_values: public_values.as_ref().map(|value| (**value).clone().into()),
                }),
                AggchainData::MultisigOnly { multisig } => {
                    proto::aggchain_data::Data::MultisigOnly(proto::MultisigOnly {
                        multisig: Some(multisig.clone().into()),
                    })
                }
                AggchainData::MultisigAndAggchainProof {
                    multisig,
                    aggchain_proof,
                } => proto::aggchain_data::Data::MultisigAndAggchainProof(
                    proto::MultisigAndAggchainProof {
                        multisig: Some(multisig.clone().into()),
                        proof: Some(proto::Proof::try_from(&aggchain_proof.proof)?),
                        aggchain_params: Some(aggchain_proof.aggchain_params.into()),
                        public_values: aggchain_proof
                            .public_values
                            .as_ref()
                            .map(|value| (**value).clone().into()),
                    },
                ),
            }),
        })
    }
}

impl TryFrom<proto::Certificate> for Certificate {
    type Error = CertificateConversionError;

    fn try_from(value: proto::Certificate) -> Result<Self, Self::Error> {
        Ok(Self {
            network_id: NetworkId::new(value.network_id),
            height: Height::new(value.height),
            prev_local_exit_root: required_field!(value, prev_local_exit_root)?.try_into()?,
            new_local_exit_root: required_field!(value, new_local_exit_root)?.try_into()?,
            bridge_exits: value
                .bridge_exits
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            imported_bridge_exits: value
                .imported_bridge_exits
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            metadata: required_field!(value, metadata)?.try_into()?,
            aggchain_data: required_field!(value, aggchain_data)?.try_into()?,
            custom_chain_data: value.custom_chain_data.to_vec(),
            l1_info_tree_leaf_count: value.l1_info_tree_leaf_count,
        })
    }
}

impl TryFrom<&Certificate> for proto::Certificate {
    type Error = CertificateConversionError;

    fn try_from(value: &Certificate) -> Result<Self, Self::Error> {
        Ok(Self {
            network_id: value.network_id.to_u32(),
            height: value.height.as_u64(),
            prev_local_exit_root: Some(value.prev_local_exit_root.into()),
            new_local_exit_root: Some(value.new_local_exit_root.into()),
            bridge_exits: value.bridge_exits.iter().cloned().map(Into::into).collect(),
            imported_bridge_exits: value
                .imported_bridge_exits
                .iter()
                .cloned()
                .map(Into::into)
                .collect(),
            metadata: Some(value.metadata.into()),
            aggchain_data: Some((&value.aggchain_data).try_into()?),
            custom_chain_data: value.custom_chain_data.clone().into(),
            l1_info_tree_leaf_count: value.l1_info_tree_leaf_count,
        })
    }
}
