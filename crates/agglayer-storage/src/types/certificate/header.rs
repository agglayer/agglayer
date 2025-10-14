use agglayer_types::{
    CertificateHeader, CertificateId, CertificateIndex, CertificateStatus, EpochNumber, Height,
    LocalExitRoot, Metadata, NetworkId, SettlementTxHash,
};

use super::{bincode_codec, CodecError};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
struct CertificateHeaderV0 {
    pub network_id: NetworkId,
    pub height: Height,
    pub epoch_number: Option<EpochNumber>,
    pub certificate_index: Option<CertificateIndex>,
    pub certificate_id: CertificateId,
    pub prev_local_exit_root: LocalExitRoot,
    pub new_local_exit_root: LocalExitRoot,
    pub metadata: Metadata,
    pub status: CertificateStatus,
    pub settlement_tx_hashes: SettlementTxHashSequence,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum SettlementTxHashSequence {
    None,
    One(SettlementTxHash),
    Many(Vec<SettlementTxHash>),
}

impl serde::Serialize for SettlementTxHashSequence {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::None => (0_u8, ()).serialize(serializer),
            Self::One(hash) => (1_u8, hash).serialize(serializer),
            Self::Many(hashes) => (2_u8, hashes).serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for SettlementTxHashSequence {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de;

        struct SettlementTxHashSequenceVisitor;
        impl<'de> de::Visitor<'de> for SettlementTxHashSequenceVisitor {
            type Value = SettlementTxHashSequence;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("settlement hash sequence tag")
            }

            fn visit_seq<A: de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                match seq.next_element::<u8>()? {
                    None => Err(de::Error::missing_field("settlement tx hash sequence tag")),
                    Some(0) => {
                        let () = seq
                            .next_element::<()>()?
                            .ok_or(de::Error::missing_field("no hash"))?;
                        Ok(Self::Value::None)
                    }
                    Some(1) => {
                        let hash = seq
                            .next_element::<SettlementTxHash>()?
                            .ok_or(de::Error::missing_field("settlement tx hash"))?;
                        Ok(Self::Value::One(hash))
                    }
                    Some(2) => {
                        let hashes = seq
                            .next_element::<Vec<SettlementTxHash>>()?
                            .ok_or(de::Error::missing_field("settlement tx hashes"))?;
                        Ok(Self::Value::Many(hashes))
                    }
                    Some(_) => Err(de::Error::custom("invalid SettlementTxHashSequence tag")),
                }
            }
        }

        deserializer.deserialize_tuple(2, SettlementTxHashSequenceVisitor)
    }
}

impl SettlementTxHashSequence {
    fn from_vec(val: Vec<SettlementTxHash>) -> Self {
        match val.as_slice() {
            &[] => Self::None,
            &[hash] => Self::One(hash),
            _ => Self::Many(val),
        }
    }

    fn into_vec(self) -> Vec<SettlementTxHash> {
        match self {
            Self::None => Vec::new(),
            Self::One(hash) => vec![hash],
            Self::Many(hashes) => hashes,
        }
    }
}

impl From<CertificateHeader> for CertificateHeaderV0 {
    fn from(value: CertificateHeader) -> Self {
        let CertificateHeader {
            network_id,
            height,
            epoch_number,
            certificate_index,
            certificate_id,
            prev_local_exit_root,
            new_local_exit_root,
            metadata,
            status,
            settlement_tx_hashes,
        } = value;

        CertificateHeaderV0 {
            network_id,
            height,
            epoch_number,
            certificate_index,
            certificate_id,
            prev_local_exit_root,
            new_local_exit_root,
            metadata,
            status,
            settlement_tx_hashes: SettlementTxHashSequence::from_vec(settlement_tx_hashes),
        }
    }
}

impl From<CertificateHeaderV0> for CertificateHeader {
    fn from(value: CertificateHeaderV0) -> Self {
        let CertificateHeaderV0 {
            network_id,
            height,
            epoch_number,
            certificate_index,
            certificate_id,
            prev_local_exit_root,
            new_local_exit_root,
            metadata,
            status,
            settlement_tx_hashes,
        } = value;

        CertificateHeader {
            network_id,
            height,
            epoch_number,
            certificate_index,
            certificate_id,
            prev_local_exit_root,
            new_local_exit_root,
            metadata,
            status,
            settlement_tx_hashes: settlement_tx_hashes.into_vec(),
        }
    }
}

type CurrentCertificateHeader = CertificateHeaderV0;

impl crate::columns::Codec for CertificateHeader {
    fn encode_into<W: std::io::Write>(&self, writer: W) -> Result<(), CodecError> {
        let cur_hdr = CurrentCertificateHeader::from(self.clone());
        Ok(bincode_codec().serialize_into(writer, &cur_hdr)?)
    }

    fn decode(bytes: &[u8]) -> Result<Self, CodecError> {
        Ok(bincode_codec()
            .deserialize::<CurrentCertificateHeader>(bytes)?
            .into())
    }
}
