// @generated
impl serde::Serialize for LatestPendingCertificateInfo {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.height != 0 {
            len += 1;
        }
        if !self.id.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("agglayer.storage.v0.LatestPendingCertificateInfo", len)?;
        if self.height != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("height", ToString::to_string(&self.height).as_str())?;
        }
        if !self.id.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("id", pbjson::private::base64::encode(&self.id).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for LatestPendingCertificateInfo {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "height",
            "id",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Height,
            Id,
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
                            "height" => Ok(GeneratedField::Height),
                            "id" => Ok(GeneratedField::Id),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = LatestPendingCertificateInfo;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct agglayer.storage.v0.LatestPendingCertificateInfo")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<LatestPendingCertificateInfo, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut height__ = None;
                let mut id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Height => {
                            if height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("height"));
                            }
                            height__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Id => {
                            if id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(LatestPendingCertificateInfo {
                    height: height__.unwrap_or_default(),
                    id: id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("agglayer.storage.v0.LatestPendingCertificateInfo", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for NetworkInfoValue {
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
        let mut struct_ser = serializer.serialize_struct("agglayer.storage.v0.NetworkInfoValue", len)?;
        if let Some(v) = self.value.as_ref() {
            match v {
                network_info_value::Value::NetworkType(v) => {
                    let v = NetworkType::try_from(*v)
                        .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", *v)))?;
                    struct_ser.serialize_field("networkType", &v)?;
                }
                network_info_value::Value::SettledCertificate(v) => {
                    struct_ser.serialize_field("settledCertificate", v)?;
                }
                network_info_value::Value::SettledClaim(v) => {
                    struct_ser.serialize_field("settledClaim", v)?;
                }
                network_info_value::Value::LatestPendingCertificateInfo(v) => {
                    struct_ser.serialize_field("latestPendingCertificateInfo", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for NetworkInfoValue {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "network_type",
            "networkType",
            "settled_certificate",
            "settledCertificate",
            "settled_claim",
            "settledClaim",
            "latest_pending_certificate_info",
            "latestPendingCertificateInfo",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            NetworkType,
            SettledCertificate,
            SettledClaim,
            LatestPendingCertificateInfo,
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
                            "networkType" | "network_type" => Ok(GeneratedField::NetworkType),
                            "settledCertificate" | "settled_certificate" => Ok(GeneratedField::SettledCertificate),
                            "settledClaim" | "settled_claim" => Ok(GeneratedField::SettledClaim),
                            "latestPendingCertificateInfo" | "latest_pending_certificate_info" => Ok(GeneratedField::LatestPendingCertificateInfo),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = NetworkInfoValue;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct agglayer.storage.v0.NetworkInfoValue")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<NetworkInfoValue, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut value__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::NetworkType => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("networkType"));
                            }
                            value__ = map_.next_value::<::std::option::Option<NetworkType>>()?.map(|x| network_info_value::Value::NetworkType(x as i32));
                        }
                        GeneratedField::SettledCertificate => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("settledCertificate"));
                            }
                            value__ = map_.next_value::<::std::option::Option<_>>()?.map(network_info_value::Value::SettledCertificate)
;
                        }
                        GeneratedField::SettledClaim => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("settledClaim"));
                            }
                            value__ = map_.next_value::<::std::option::Option<_>>()?.map(network_info_value::Value::SettledClaim)
;
                        }
                        GeneratedField::LatestPendingCertificateInfo => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("latestPendingCertificateInfo"));
                            }
                            value__ = map_.next_value::<::std::option::Option<_>>()?.map(network_info_value::Value::LatestPendingCertificateInfo)
;
                        }
                    }
                }
                Ok(NetworkInfoValue {
                    value: value__,
                })
            }
        }
        deserializer.deserialize_struct("agglayer.storage.v0.NetworkInfoValue", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for NetworkType {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "NETWORK_TYPE_UNSPECIFIED",
            Self::Ecdsa => "NETWORK_TYPE_ECDSA",
            Self::Generic => "NETWORK_TYPE_GENERIC",
            Self::MultisigOnly => "NETWORK_TYPE_MULTISIG_ONLY",
            Self::MultisigAndAggchainProof => "NETWORK_TYPE_MULTISIG_AND_AGGCHAIN_PROOF",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for NetworkType {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "NETWORK_TYPE_UNSPECIFIED",
            "NETWORK_TYPE_ECDSA",
            "NETWORK_TYPE_GENERIC",
            "NETWORK_TYPE_MULTISIG_ONLY",
            "NETWORK_TYPE_MULTISIG_AND_AGGCHAIN_PROOF",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = NetworkType;

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
                    "NETWORK_TYPE_UNSPECIFIED" => Ok(NetworkType::Unspecified),
                    "NETWORK_TYPE_ECDSA" => Ok(NetworkType::Ecdsa),
                    "NETWORK_TYPE_GENERIC" => Ok(NetworkType::Generic),
                    "NETWORK_TYPE_MULTISIG_ONLY" => Ok(NetworkType::MultisigOnly),
                    "NETWORK_TYPE_MULTISIG_AND_AGGCHAIN_PROOF" => Ok(NetworkType::MultisigAndAggchainProof),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for SettledCertificate {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.certificate_id.is_some() {
            len += 1;
        }
        if self.new_pp_root.is_some() {
            len += 1;
        }
        if self.let_leaf_count.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("agglayer.storage.v0.SettledCertificate", len)?;
        if let Some(v) = self.certificate_id.as_ref() {
            struct_ser.serialize_field("certificateId", v)?;
        }
        if let Some(v) = self.new_pp_root.as_ref() {
            struct_ser.serialize_field("newPpRoot", v)?;
        }
        if let Some(v) = self.let_leaf_count.as_ref() {
            struct_ser.serialize_field("letLeafCount", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SettledCertificate {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "certificate_id",
            "certificateId",
            "new_pp_root",
            "newPpRoot",
            "let_leaf_count",
            "letLeafCount",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            CertificateId,
            NewPpRoot,
            LetLeafCount,
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
                            "certificateId" | "certificate_id" => Ok(GeneratedField::CertificateId),
                            "newPpRoot" | "new_pp_root" => Ok(GeneratedField::NewPpRoot),
                            "letLeafCount" | "let_leaf_count" => Ok(GeneratedField::LetLeafCount),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SettledCertificate;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct agglayer.storage.v0.SettledCertificate")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SettledCertificate, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut certificate_id__ = None;
                let mut new_pp_root__ = None;
                let mut let_leaf_count__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::CertificateId => {
                            if certificate_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("certificateId"));
                            }
                            certificate_id__ = map_.next_value()?;
                        }
                        GeneratedField::NewPpRoot => {
                            if new_pp_root__.is_some() {
                                return Err(serde::de::Error::duplicate_field("newPpRoot"));
                            }
                            new_pp_root__ = map_.next_value()?;
                        }
                        GeneratedField::LetLeafCount => {
                            if let_leaf_count__.is_some() {
                                return Err(serde::de::Error::duplicate_field("letLeafCount"));
                            }
                            let_leaf_count__ = map_.next_value()?;
                        }
                    }
                }
                Ok(SettledCertificate {
                    certificate_id: certificate_id__,
                    new_pp_root: new_pp_root__,
                    let_leaf_count: let_leaf_count__,
                })
            }
        }
        deserializer.deserialize_struct("agglayer.storage.v0.SettledCertificate", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SettledCertificateId {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.id.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("agglayer.storage.v0.SettledCertificateId", len)?;
        if !self.id.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("id", pbjson::private::base64::encode(&self.id).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SettledCertificateId {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "id",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Id,
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
                            "id" => Ok(GeneratedField::Id),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SettledCertificateId;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct agglayer.storage.v0.SettledCertificateId")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SettledCertificateId, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Id => {
                            if id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(SettledCertificateId {
                    id: id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("agglayer.storage.v0.SettledCertificateId", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SettledClaim {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.global_index.is_empty() {
            len += 1;
        }
        if !self.bridge_exit_hash.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("agglayer.storage.v0.SettledClaim", len)?;
        if !self.global_index.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("globalIndex", pbjson::private::base64::encode(&self.global_index).as_str())?;
        }
        if !self.bridge_exit_hash.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("bridgeExitHash", pbjson::private::base64::encode(&self.bridge_exit_hash).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SettledClaim {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "global_index",
            "globalIndex",
            "bridge_exit_hash",
            "bridgeExitHash",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            GlobalIndex,
            BridgeExitHash,
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
                            "globalIndex" | "global_index" => Ok(GeneratedField::GlobalIndex),
                            "bridgeExitHash" | "bridge_exit_hash" => Ok(GeneratedField::BridgeExitHash),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SettledClaim;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct agglayer.storage.v0.SettledClaim")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SettledClaim, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut global_index__ = None;
                let mut bridge_exit_hash__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::GlobalIndex => {
                            if global_index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("globalIndex"));
                            }
                            global_index__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::BridgeExitHash => {
                            if bridge_exit_hash__.is_some() {
                                return Err(serde::de::Error::duplicate_field("bridgeExitHash"));
                            }
                            bridge_exit_hash__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(SettledClaim {
                    global_index: global_index__.unwrap_or_default(),
                    bridge_exit_hash: bridge_exit_hash__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("agglayer.storage.v0.SettledClaim", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SettledLocalExitRoot {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.root.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("agglayer.storage.v0.SettledLocalExitRoot", len)?;
        if !self.root.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("root", pbjson::private::base64::encode(&self.root).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SettledLocalExitRoot {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "root",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Root,
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
                            "root" => Ok(GeneratedField::Root),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SettledLocalExitRoot;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct agglayer.storage.v0.SettledLocalExitRoot")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SettledLocalExitRoot, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut root__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Root => {
                            if root__.is_some() {
                                return Err(serde::de::Error::duplicate_field("root"));
                            }
                            root__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(SettledLocalExitRoot {
                    root: root__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("agglayer.storage.v0.SettledLocalExitRoot", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SettledLocalExitTreeLeafCount {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.settled_let_leaf_count != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("agglayer.storage.v0.SettledLocalExitTreeLeafCount", len)?;
        if self.settled_let_leaf_count != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("settledLetLeafCount", ToString::to_string(&self.settled_let_leaf_count).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SettledLocalExitTreeLeafCount {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "settled_let_leaf_count",
            "settledLetLeafCount",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SettledLetLeafCount,
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
                            "settledLetLeafCount" | "settled_let_leaf_count" => Ok(GeneratedField::SettledLetLeafCount),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SettledLocalExitTreeLeafCount;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct agglayer.storage.v0.SettledLocalExitTreeLeafCount")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SettledLocalExitTreeLeafCount, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut settled_let_leaf_count__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::SettledLetLeafCount => {
                            if settled_let_leaf_count__.is_some() {
                                return Err(serde::de::Error::duplicate_field("settledLetLeafCount"));
                            }
                            settled_let_leaf_count__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(SettledLocalExitTreeLeafCount {
                    settled_let_leaf_count: settled_let_leaf_count__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("agglayer.storage.v0.SettledLocalExitTreeLeafCount", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SettledPessimisticProofRoot {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.root.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("agglayer.storage.v0.SettledPessimisticProofRoot", len)?;
        if !self.root.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("root", pbjson::private::base64::encode(&self.root).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SettledPessimisticProofRoot {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "root",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Root,
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
                            "root" => Ok(GeneratedField::Root),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SettledPessimisticProofRoot;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct agglayer.storage.v0.SettledPessimisticProofRoot")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SettledPessimisticProofRoot, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut root__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Root => {
                            if root__.is_some() {
                                return Err(serde::de::Error::duplicate_field("root"));
                            }
                            root__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(SettledPessimisticProofRoot {
                    root: root__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("agglayer.storage.v0.SettledPessimisticProofRoot", FIELDS, GeneratedVisitor)
    }
}
