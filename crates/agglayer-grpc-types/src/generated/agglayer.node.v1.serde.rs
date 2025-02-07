// @generated
impl serde::Serialize for ErrorKind {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "ERROR_KIND_UNSPECIFIED",
            Self::SignatureVerification => "ERROR_KIND_SIGNATURE_VERIFICATION",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for ErrorKind {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "ERROR_KIND_UNSPECIFIED",
            "ERROR_KIND_SIGNATURE_VERIFICATION",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ErrorKind;

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
                    "ERROR_KIND_UNSPECIFIED" => Ok(ErrorKind::Unspecified),
                    "ERROR_KIND_SIGNATURE_VERIFICATION" => Ok(ErrorKind::SignatureVerification),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for GetEpochConfigurationRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("agglayer.node.v1.GetEpochConfigurationRequest", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetEpochConfigurationRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
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
                            Err(serde::de::Error::unknown_field(value, FIELDS))
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GetEpochConfigurationRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct agglayer.node.v1.GetEpochConfigurationRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GetEpochConfigurationRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map_.next_key::<GeneratedField>()?.is_some() {
                    let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(GetEpochConfigurationRequest {
                })
            }
        }
        deserializer.deserialize_struct("agglayer.node.v1.GetEpochConfigurationRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetEpochConfigurationResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.epoch_configuration.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("agglayer.node.v1.GetEpochConfigurationResponse", len)?;
        if let Some(v) = self.epoch_configuration.as_ref() {
            struct_ser.serialize_field("epochConfiguration", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetEpochConfigurationResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "epoch_configuration",
            "epochConfiguration",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            EpochConfiguration,
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
                            "epochConfiguration" | "epoch_configuration" => Ok(GeneratedField::EpochConfiguration),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GetEpochConfigurationResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct agglayer.node.v1.GetEpochConfigurationResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GetEpochConfigurationResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut epoch_configuration__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::EpochConfiguration => {
                            if epoch_configuration__.is_some() {
                                return Err(serde::de::Error::duplicate_field("epochConfiguration"));
                            }
                            epoch_configuration__ = map_.next_value()?;
                        }
                    }
                }
                Ok(GetEpochConfigurationResponse {
                    epoch_configuration: epoch_configuration__,
                })
            }
        }
        deserializer.deserialize_struct("agglayer.node.v1.GetEpochConfigurationResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetLatestKnownCertificateHeaderRequest {
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
        let mut struct_ser = serializer.serialize_struct("agglayer.node.v1.GetLatestKnownCertificateHeaderRequest", len)?;
        if self.network_id != 0 {
            struct_ser.serialize_field("networkId", &self.network_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetLatestKnownCertificateHeaderRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "network_id",
            "networkId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            NetworkId,
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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GetLatestKnownCertificateHeaderRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct agglayer.node.v1.GetLatestKnownCertificateHeaderRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GetLatestKnownCertificateHeaderRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut network_id__ = None;
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
                    }
                }
                Ok(GetLatestKnownCertificateHeaderRequest {
                    network_id: network_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("agglayer.node.v1.GetLatestKnownCertificateHeaderRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetLatestKnownCertificateHeaderResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.certificate_header.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("agglayer.node.v1.GetLatestKnownCertificateHeaderResponse", len)?;
        if let Some(v) = self.certificate_header.as_ref() {
            struct_ser.serialize_field("certificateHeader", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetLatestKnownCertificateHeaderResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "certificate_header",
            "certificateHeader",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            CertificateHeader,
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
                            "certificateHeader" | "certificate_header" => Ok(GeneratedField::CertificateHeader),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GetLatestKnownCertificateHeaderResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct agglayer.node.v1.GetLatestKnownCertificateHeaderResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GetLatestKnownCertificateHeaderResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut certificate_header__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::CertificateHeader => {
                            if certificate_header__.is_some() {
                                return Err(serde::de::Error::duplicate_field("certificateHeader"));
                            }
                            certificate_header__ = map_.next_value()?;
                        }
                    }
                }
                Ok(GetLatestKnownCertificateHeaderResponse {
                    certificate_header: certificate_header__,
                })
            }
        }
        deserializer.deserialize_struct("agglayer.node.v1.GetLatestKnownCertificateHeaderResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetLatestPendingCertificateHeaderRequest {
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
        let mut struct_ser = serializer.serialize_struct("agglayer.node.v1.GetLatestPendingCertificateHeaderRequest", len)?;
        if self.network_id != 0 {
            struct_ser.serialize_field("networkId", &self.network_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetLatestPendingCertificateHeaderRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "network_id",
            "networkId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            NetworkId,
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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GetLatestPendingCertificateHeaderRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct agglayer.node.v1.GetLatestPendingCertificateHeaderRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GetLatestPendingCertificateHeaderRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut network_id__ = None;
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
                    }
                }
                Ok(GetLatestPendingCertificateHeaderRequest {
                    network_id: network_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("agglayer.node.v1.GetLatestPendingCertificateHeaderRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetLatestPendingCertificateHeaderResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.certificate_header.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("agglayer.node.v1.GetLatestPendingCertificateHeaderResponse", len)?;
        if let Some(v) = self.certificate_header.as_ref() {
            struct_ser.serialize_field("certificateHeader", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetLatestPendingCertificateHeaderResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "certificate_header",
            "certificateHeader",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            CertificateHeader,
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
                            "certificateHeader" | "certificate_header" => Ok(GeneratedField::CertificateHeader),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GetLatestPendingCertificateHeaderResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct agglayer.node.v1.GetLatestPendingCertificateHeaderResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GetLatestPendingCertificateHeaderResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut certificate_header__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::CertificateHeader => {
                            if certificate_header__.is_some() {
                                return Err(serde::de::Error::duplicate_field("certificateHeader"));
                            }
                            certificate_header__ = map_.next_value()?;
                        }
                    }
                }
                Ok(GetLatestPendingCertificateHeaderResponse {
                    certificate_header: certificate_header__,
                })
            }
        }
        deserializer.deserialize_struct("agglayer.node.v1.GetLatestPendingCertificateHeaderResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetLatestSettledCertificateHeaderRequest {
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
        let mut struct_ser = serializer.serialize_struct("agglayer.node.v1.GetLatestSettledCertificateHeaderRequest", len)?;
        if self.network_id != 0 {
            struct_ser.serialize_field("networkId", &self.network_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetLatestSettledCertificateHeaderRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "network_id",
            "networkId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            NetworkId,
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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GetLatestSettledCertificateHeaderRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct agglayer.node.v1.GetLatestSettledCertificateHeaderRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GetLatestSettledCertificateHeaderRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut network_id__ = None;
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
                    }
                }
                Ok(GetLatestSettledCertificateHeaderRequest {
                    network_id: network_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("agglayer.node.v1.GetLatestSettledCertificateHeaderRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetLatestSettledCertificateHeaderResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.certificate_header.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("agglayer.node.v1.GetLatestSettledCertificateHeaderResponse", len)?;
        if let Some(v) = self.certificate_header.as_ref() {
            struct_ser.serialize_field("certificateHeader", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetLatestSettledCertificateHeaderResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "certificate_header",
            "certificateHeader",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            CertificateHeader,
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
                            "certificateHeader" | "certificate_header" => Ok(GeneratedField::CertificateHeader),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GetLatestSettledCertificateHeaderResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct agglayer.node.v1.GetLatestSettledCertificateHeaderResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GetLatestSettledCertificateHeaderResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut certificate_header__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::CertificateHeader => {
                            if certificate_header__.is_some() {
                                return Err(serde::de::Error::duplicate_field("certificateHeader"));
                            }
                            certificate_header__ = map_.next_value()?;
                        }
                    }
                }
                Ok(GetLatestSettledCertificateHeaderResponse {
                    certificate_header: certificate_header__,
                })
            }
        }
        deserializer.deserialize_struct("agglayer.node.v1.GetLatestSettledCertificateHeaderResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SubmitCertificateRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.certificate.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("agglayer.node.v1.SubmitCertificateRequest", len)?;
        if let Some(v) = self.certificate.as_ref() {
            struct_ser.serialize_field("certificate", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SubmitCertificateRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "certificate",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Certificate,
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
                            "certificate" => Ok(GeneratedField::Certificate),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SubmitCertificateRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct agglayer.node.v1.SubmitCertificateRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SubmitCertificateRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut certificate__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Certificate => {
                            if certificate__.is_some() {
                                return Err(serde::de::Error::duplicate_field("certificate"));
                            }
                            certificate__ = map_.next_value()?;
                        }
                    }
                }
                Ok(SubmitCertificateRequest {
                    certificate: certificate__,
                })
            }
        }
        deserializer.deserialize_struct("agglayer.node.v1.SubmitCertificateRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SubmitCertificateResponse {
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
        let mut struct_ser = serializer.serialize_struct("agglayer.node.v1.SubmitCertificateResponse", len)?;
        if let Some(v) = self.certificate_id.as_ref() {
            struct_ser.serialize_field("certificateId", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SubmitCertificateResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "certificate_id",
            "certificateId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            CertificateId,
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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SubmitCertificateResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct agglayer.node.v1.SubmitCertificateResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SubmitCertificateResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut certificate_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::CertificateId => {
                            if certificate_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("certificateId"));
                            }
                            certificate_id__ = map_.next_value()?;
                        }
                    }
                }
                Ok(SubmitCertificateResponse {
                    certificate_id: certificate_id__,
                })
            }
        }
        deserializer.deserialize_struct("agglayer.node.v1.SubmitCertificateResponse", FIELDS, GeneratedVisitor)
    }
}
