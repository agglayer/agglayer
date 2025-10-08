// @generated
impl serde::Serialize for Certificate {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.network_id != 0 {
            len += 1;
        }
        if self.height != 0 {
            len += 1;
        }
        if self.prev_local_exit_root.is_some() {
            len += 1;
        }
        if self.new_local_exit_root.is_some() {
            len += 1;
        }
        if !self.bridge_exits.is_empty() {
            len += 1;
        }
        if !self.imported_bridge_exits.is_empty() {
            len += 1;
        }
        if self.metadata.is_some() {
            len += 1;
        }
        if self.aggchain_data.is_some() {
            len += 1;
        }
        if !self.custom_chain_data.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("agglayer.node.v1.types.Certificate", len)?;
        if self.network_id != 0 {
            struct_ser.serialize_field("networkId", &self.network_id)?;
        }
        if self.height != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("height", ToString::to_string(&self.height).as_str())?;
        }
        if let Some(v) = self.prev_local_exit_root.as_ref() {
            struct_ser.serialize_field("prevLocalExitRoot", v)?;
        }
        if let Some(v) = self.new_local_exit_root.as_ref() {
            struct_ser.serialize_field("newLocalExitRoot", v)?;
        }
        if !self.bridge_exits.is_empty() {
            struct_ser.serialize_field("bridgeExits", &self.bridge_exits)?;
        }
        if !self.imported_bridge_exits.is_empty() {
            struct_ser.serialize_field("importedBridgeExits", &self.imported_bridge_exits)?;
        }
        if let Some(v) = self.metadata.as_ref() {
            struct_ser.serialize_field("metadata", v)?;
        }
        if let Some(v) = self.aggchain_data.as_ref() {
            struct_ser.serialize_field("aggchainData", v)?;
        }
        if !self.custom_chain_data.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("customChainData", pbjson::private::base64::encode(&self.custom_chain_data).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Certificate {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "network_id",
            "networkId",
            "height",
            "prev_local_exit_root",
            "prevLocalExitRoot",
            "new_local_exit_root",
            "newLocalExitRoot",
            "bridge_exits",
            "bridgeExits",
            "imported_bridge_exits",
            "importedBridgeExits",
            "metadata",
            "aggchain_data",
            "aggchainData",
            "custom_chain_data",
            "customChainData",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            NetworkId,
            Height,
            PrevLocalExitRoot,
            NewLocalExitRoot,
            BridgeExits,
            ImportedBridgeExits,
            Metadata,
            AggchainData,
            CustomChainData,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "networkId" | "network_id" => Ok(GeneratedField::NetworkId),
                            "height" => Ok(GeneratedField::Height),
                            "prevLocalExitRoot" | "prev_local_exit_root" => Ok(GeneratedField::PrevLocalExitRoot),
                            "newLocalExitRoot" | "new_local_exit_root" => Ok(GeneratedField::NewLocalExitRoot),
                            "bridgeExits" | "bridge_exits" => Ok(GeneratedField::BridgeExits),
                            "importedBridgeExits" | "imported_bridge_exits" => Ok(GeneratedField::ImportedBridgeExits),
                            "metadata" => Ok(GeneratedField::Metadata),
                            "aggchainData" | "aggchain_data" => Ok(GeneratedField::AggchainData),
                            "customChainData" | "custom_chain_data" => Ok(GeneratedField::CustomChainData),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Certificate;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct agglayer.node.v1.types.Certificate")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Certificate, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut network_id__ = None;
                let mut height__ = None;
                let mut prev_local_exit_root__ = None;
                let mut new_local_exit_root__ = None;
                let mut bridge_exits__ = None;
                let mut imported_bridge_exits__ = None;
                let mut metadata__ = None;
                let mut aggchain_data__ = None;
                let mut custom_chain_data__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::NetworkId => {
                            if network_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("networkId"));
                            }
                            network_id__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Height => {
                            if height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("height"));
                            }
                            height__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::PrevLocalExitRoot => {
                            if prev_local_exit_root__.is_some() {
                                return Err(serde::de::Error::duplicate_field("prevLocalExitRoot"));
                            }
                            prev_local_exit_root__ = map_.next_value()?;
                        }
                        GeneratedField::NewLocalExitRoot => {
                            if new_local_exit_root__.is_some() {
                                return Err(serde::de::Error::duplicate_field("newLocalExitRoot"));
                            }
                            new_local_exit_root__ = map_.next_value()?;
                        }
                        GeneratedField::BridgeExits => {
                            if bridge_exits__.is_some() {
                                return Err(serde::de::Error::duplicate_field("bridgeExits"));
                            }
                            bridge_exits__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ImportedBridgeExits => {
                            if imported_bridge_exits__.is_some() {
                                return Err(serde::de::Error::duplicate_field("importedBridgeExits"));
                            }
                            imported_bridge_exits__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Metadata => {
                            if metadata__.is_some() {
                                return Err(serde::de::Error::duplicate_field("metadata"));
                            }
                            metadata__ = map_.next_value()?;
                        }
                        GeneratedField::AggchainData => {
                            if aggchain_data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("aggchainData"));
                            }
                            aggchain_data__ = map_.next_value()?;
                        }
                        GeneratedField::CustomChainData => {
                            if custom_chain_data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("customChainData"));
                            }
                            custom_chain_data__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(Certificate {
                    network_id: network_id__.unwrap_or_default(),
                    height: height__.unwrap_or_default(),
                    prev_local_exit_root: prev_local_exit_root__,
                    new_local_exit_root: new_local_exit_root__,
                    bridge_exits: bridge_exits__.unwrap_or_default(),
                    imported_bridge_exits: imported_bridge_exits__.unwrap_or_default(),
                    metadata: metadata__,
                    aggchain_data: aggchain_data__,
                    custom_chain_data: custom_chain_data__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("agglayer.node.v1.types.Certificate", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CertificateHeader {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.network_id != 0 {
            len += 1;
        }
        if self.height != 0 {
            len += 1;
        }
        if self.epoch_number.is_some() {
            len += 1;
        }
        if self.certificate_index.is_some() {
            len += 1;
        }
        if self.certificate_id.is_some() {
            len += 1;
        }
        if self.prev_local_exit_root.is_some() {
            len += 1;
        }
        if self.new_local_exit_root.is_some() {
            len += 1;
        }
        if self.metadata.is_some() {
            len += 1;
        }
        if self.status != 0 {
            len += 1;
        }
        if self.error.is_some() {
            len += 1;
        }
        if self.settlement_tx_hash.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("agglayer.node.v1.types.CertificateHeader", len)?;
        if self.network_id != 0 {
            struct_ser.serialize_field("networkId", &self.network_id)?;
        }
        if self.height != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("height", ToString::to_string(&self.height).as_str())?;
        }
        if let Some(v) = self.epoch_number.as_ref() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("epochNumber", ToString::to_string(&v).as_str())?;
        }
        if let Some(v) = self.certificate_index.as_ref() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("certificateIndex", ToString::to_string(&v).as_str())?;
        }
        if let Some(v) = self.certificate_id.as_ref() {
            struct_ser.serialize_field("certificateId", v)?;
        }
        if let Some(v) = self.prev_local_exit_root.as_ref() {
            struct_ser.serialize_field("prevLocalExitRoot", v)?;
        }
        if let Some(v) = self.new_local_exit_root.as_ref() {
            struct_ser.serialize_field("newLocalExitRoot", v)?;
        }
        if let Some(v) = self.metadata.as_ref() {
            struct_ser.serialize_field("metadata", v)?;
        }
        if self.status != 0 {
            let v = CertificateStatus::try_from(self.status)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.status)))?;
            struct_ser.serialize_field("status", &v)?;
        }
        if let Some(v) = self.error.as_ref() {
            struct_ser.serialize_field("error", v)?;
        }
        if let Some(v) = self.settlement_tx_hash.as_ref() {
            struct_ser.serialize_field("settlementTxHash", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CertificateHeader {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "network_id",
            "networkId",
            "height",
            "epoch_number",
            "epochNumber",
            "certificate_index",
            "certificateIndex",
            "certificate_id",
            "certificateId",
            "prev_local_exit_root",
            "prevLocalExitRoot",
            "new_local_exit_root",
            "newLocalExitRoot",
            "metadata",
            "status",
            "error",
            "settlement_tx_hash",
            "settlementTxHash",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            NetworkId,
            Height,
            EpochNumber,
            CertificateIndex,
            CertificateId,
            PrevLocalExitRoot,
            NewLocalExitRoot,
            Metadata,
            Status,
            Error,
            SettlementTxHash,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "networkId" | "network_id" => Ok(GeneratedField::NetworkId),
                            "height" => Ok(GeneratedField::Height),
                            "epochNumber" | "epoch_number" => Ok(GeneratedField::EpochNumber),
                            "certificateIndex" | "certificate_index" => Ok(GeneratedField::CertificateIndex),
                            "certificateId" | "certificate_id" => Ok(GeneratedField::CertificateId),
                            "prevLocalExitRoot" | "prev_local_exit_root" => Ok(GeneratedField::PrevLocalExitRoot),
                            "newLocalExitRoot" | "new_local_exit_root" => Ok(GeneratedField::NewLocalExitRoot),
                            "metadata" => Ok(GeneratedField::Metadata),
                            "status" => Ok(GeneratedField::Status),
                            "error" => Ok(GeneratedField::Error),
                            "settlementTxHash" | "settlement_tx_hash" => Ok(GeneratedField::SettlementTxHash),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CertificateHeader;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct agglayer.node.v1.types.CertificateHeader")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CertificateHeader, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut network_id__ = None;
                let mut height__ = None;
                let mut epoch_number__ = None;
                let mut certificate_index__ = None;
                let mut certificate_id__ = None;
                let mut prev_local_exit_root__ = None;
                let mut new_local_exit_root__ = None;
                let mut metadata__ = None;
                let mut status__ = None;
                let mut error__ = None;
                let mut settlement_tx_hash__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::NetworkId => {
                            if network_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("networkId"));
                            }
                            network_id__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Height => {
                            if height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("height"));
                            }
                            height__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::EpochNumber => {
                            if epoch_number__.is_some() {
                                return Err(serde::de::Error::duplicate_field("epochNumber"));
                            }
                            epoch_number__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::CertificateIndex => {
                            if certificate_index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("certificateIndex"));
                            }
                            certificate_index__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::CertificateId => {
                            if certificate_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("certificateId"));
                            }
                            certificate_id__ = map_.next_value()?;
                        }
                        GeneratedField::PrevLocalExitRoot => {
                            if prev_local_exit_root__.is_some() {
                                return Err(serde::de::Error::duplicate_field("prevLocalExitRoot"));
                            }
                            prev_local_exit_root__ = map_.next_value()?;
                        }
                        GeneratedField::NewLocalExitRoot => {
                            if new_local_exit_root__.is_some() {
                                return Err(serde::de::Error::duplicate_field("newLocalExitRoot"));
                            }
                            new_local_exit_root__ = map_.next_value()?;
                        }
                        GeneratedField::Metadata => {
                            if metadata__.is_some() {
                                return Err(serde::de::Error::duplicate_field("metadata"));
                            }
                            metadata__ = map_.next_value()?;
                        }
                        GeneratedField::Status => {
                            if status__.is_some() {
                                return Err(serde::de::Error::duplicate_field("status"));
                            }
                            status__ = Some(map_.next_value::<CertificateStatus>()? as i32);
                        }
                        GeneratedField::Error => {
                            if error__.is_some() {
                                return Err(serde::de::Error::duplicate_field("error"));
                            }
                            error__ = map_.next_value()?;
                        }
                        GeneratedField::SettlementTxHash => {
                            if settlement_tx_hash__.is_some() {
                                return Err(serde::de::Error::duplicate_field("settlementTxHash"));
                            }
                            settlement_tx_hash__ = map_.next_value()?;
                        }
                    }
                }
                Ok(CertificateHeader {
                    network_id: network_id__.unwrap_or_default(),
                    height: height__.unwrap_or_default(),
                    epoch_number: epoch_number__,
                    certificate_index: certificate_index__,
                    certificate_id: certificate_id__,
                    prev_local_exit_root: prev_local_exit_root__,
                    new_local_exit_root: new_local_exit_root__,
                    metadata: metadata__,
                    status: status__.unwrap_or_default(),
                    error: error__,
                    settlement_tx_hash: settlement_tx_hash__,
                })
            }
        }
        deserializer.deserialize_struct("agglayer.node.v1.types.CertificateHeader", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CertificateId {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.value.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("agglayer.node.v1.types.CertificateId", len)?;
        if let Some(v) = self.value.as_ref() {
            struct_ser.serialize_field("value", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CertificateId {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "value",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Value,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "value" => Ok(GeneratedField::Value),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CertificateId;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct agglayer.node.v1.types.CertificateId")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CertificateId, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut value__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Value => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("value"));
                            }
                            value__ = map_.next_value()?;
                        }
                    }
                }
                Ok(CertificateId {
                    value: value__,
                })
            }
        }
        deserializer.deserialize_struct("agglayer.node.v1.types.CertificateId", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CertificateStatus {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "CERTIFICATE_STATUS_UNSPECIFIED",
            Self::Pending => "CERTIFICATE_STATUS_PENDING",
            Self::Proven => "CERTIFICATE_STATUS_PROVEN",
            Self::Candidate => "CERTIFICATE_STATUS_CANDIDATE",
            Self::InError => "CERTIFICATE_STATUS_IN_ERROR",
            Self::Settled => "CERTIFICATE_STATUS_SETTLED",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for CertificateStatus {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "CERTIFICATE_STATUS_UNSPECIFIED",
            "CERTIFICATE_STATUS_PENDING",
            "CERTIFICATE_STATUS_PROVEN",
            "CERTIFICATE_STATUS_CANDIDATE",
            "CERTIFICATE_STATUS_IN_ERROR",
            "CERTIFICATE_STATUS_SETTLED",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CertificateStatus;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "CERTIFICATE_STATUS_UNSPECIFIED" => Ok(CertificateStatus::Unspecified),
                    "CERTIFICATE_STATUS_PENDING" => Ok(CertificateStatus::Pending),
                    "CERTIFICATE_STATUS_PROVEN" => Ok(CertificateStatus::Proven),
                    "CERTIFICATE_STATUS_CANDIDATE" => Ok(CertificateStatus::Candidate),
                    "CERTIFICATE_STATUS_IN_ERROR" => Ok(CertificateStatus::InError),
                    "CERTIFICATE_STATUS_SETTLED" => Ok(CertificateStatus::Settled),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for CertificateStatusError {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.message.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("agglayer.node.v1.types.CertificateStatusError", len)?;
        if !self.message.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("message", pbjson::private::base64::encode(&self.message).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CertificateStatusError {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "message",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Message,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "message" => Ok(GeneratedField::Message),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CertificateStatusError;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct agglayer.node.v1.types.CertificateStatusError")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CertificateStatusError, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut message__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Message => {
                            if message__.is_some() {
                                return Err(serde::de::Error::duplicate_field("message"));
                            }
                            message__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(CertificateStatusError {
                    message: message__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("agglayer.node.v1.types.CertificateStatusError", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EpochConfiguration {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.genesis_block != 0 {
            len += 1;
        }
        if self.epoch_duration != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("agglayer.node.v1.types.EpochConfiguration", len)?;
        if self.genesis_block != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("genesisBlock", ToString::to_string(&self.genesis_block).as_str())?;
        }
        if self.epoch_duration != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("epochDuration", ToString::to_string(&self.epoch_duration).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EpochConfiguration {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "genesis_block",
            "genesisBlock",
            "epoch_duration",
            "epochDuration",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            GenesisBlock,
            EpochDuration,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "genesisBlock" | "genesis_block" => Ok(GeneratedField::GenesisBlock),
                            "epochDuration" | "epoch_duration" => Ok(GeneratedField::EpochDuration),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = EpochConfiguration;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct agglayer.node.v1.types.EpochConfiguration")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<EpochConfiguration, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut genesis_block__ = None;
                let mut epoch_duration__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::GenesisBlock => {
                            if genesis_block__.is_some() {
                                return Err(serde::de::Error::duplicate_field("genesisBlock"));
                            }
                            genesis_block__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::EpochDuration => {
                            if epoch_duration__.is_some() {
                                return Err(serde::de::Error::duplicate_field("epochDuration"));
                            }
                            epoch_duration__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(EpochConfiguration {
                    genesis_block: genesis_block__.unwrap_or_default(),
                    epoch_duration: epoch_duration__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("agglayer.node.v1.types.EpochConfiguration", FIELDS, GeneratedVisitor)
    }
}
