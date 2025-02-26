// @generated
impl serde::Serialize for ConfigurationErrorKind {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "CONFIGURATION_ERROR_KIND_UNSPECIFIED",
            Self::UnexpectedClockConfiguration => "CONFIGURATION_ERROR_KIND_UNEXPECTED_CLOCK_CONFIGURATION",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for ConfigurationErrorKind {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "CONFIGURATION_ERROR_KIND_UNSPECIFIED",
            "CONFIGURATION_ERROR_KIND_UNEXPECTED_CLOCK_CONFIGURATION",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ConfigurationErrorKind;

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
                    "CONFIGURATION_ERROR_KIND_UNSPECIFIED" => Ok(ConfigurationErrorKind::Unspecified),
                    "CONFIGURATION_ERROR_KIND_UNEXPECTED_CLOCK_CONFIGURATION" => Ok(ConfigurationErrorKind::UnexpectedClockConfiguration),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
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
impl serde::Serialize for GetCertificateHeaderRequest {
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
        let mut struct_ser = serializer.serialize_struct("agglayer.node.v1.GetCertificateHeaderRequest", len)?;
        if let Some(v) = self.certificate_id.as_ref() {
            struct_ser.serialize_field("certificateId", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetCertificateHeaderRequest {
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
            type Value = GetCertificateHeaderRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct agglayer.node.v1.GetCertificateHeaderRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GetCertificateHeaderRequest, V::Error>
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
                Ok(GetCertificateHeaderRequest {
                    certificate_id: certificate_id__,
                })
            }
        }
        deserializer.deserialize_struct("agglayer.node.v1.GetCertificateHeaderRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetCertificateHeaderResponse {
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
        let mut struct_ser = serializer.serialize_struct("agglayer.node.v1.GetCertificateHeaderResponse", len)?;
        if let Some(v) = self.certificate_header.as_ref() {
            struct_ser.serialize_field("certificateHeader", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetCertificateHeaderResponse {
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
            type Value = GetCertificateHeaderResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct agglayer.node.v1.GetCertificateHeaderResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GetCertificateHeaderResponse, V::Error>
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
                Ok(GetCertificateHeaderResponse {
                    certificate_header: certificate_header__,
                })
            }
        }
        deserializer.deserialize_struct("agglayer.node.v1.GetCertificateHeaderResponse", FIELDS, GeneratedVisitor)
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
impl serde::Serialize for GetLatestCertificateHeaderRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.r#type != 0 {
            len += 1;
        }
        if self.network_id != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("agglayer.node.v1.GetLatestCertificateHeaderRequest", len)?;
        if self.r#type != 0 {
            let v = LatestCertificateRequestType::try_from(self.r#type)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.r#type)))?;
            struct_ser.serialize_field("type", &v)?;
        }
        if self.network_id != 0 {
            struct_ser.serialize_field("networkId", &self.network_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetLatestCertificateHeaderRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "type",
            "network_id",
            "networkId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Type,
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
                            "type" => Ok(GeneratedField::Type),
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
            type Value = GetLatestCertificateHeaderRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct agglayer.node.v1.GetLatestCertificateHeaderRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GetLatestCertificateHeaderRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut r#type__ = None;
                let mut network_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Type => {
                            if r#type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("type"));
                            }
                            r#type__ = Some(map_.next_value::<LatestCertificateRequestType>()? as i32);
                        }
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
                Ok(GetLatestCertificateHeaderRequest {
                    r#type: r#type__.unwrap_or_default(),
                    network_id: network_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("agglayer.node.v1.GetLatestCertificateHeaderRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetLatestCertificateHeaderResponse {
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
        let mut struct_ser = serializer.serialize_struct("agglayer.node.v1.GetLatestCertificateHeaderResponse", len)?;
        if let Some(v) = self.certificate_header.as_ref() {
            struct_ser.serialize_field("certificateHeader", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetLatestCertificateHeaderResponse {
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
            type Value = GetLatestCertificateHeaderResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct agglayer.node.v1.GetLatestCertificateHeaderResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GetLatestCertificateHeaderResponse, V::Error>
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
                Ok(GetLatestCertificateHeaderResponse {
                    certificate_header: certificate_header__,
                })
            }
        }
        deserializer.deserialize_struct("agglayer.node.v1.GetLatestCertificateHeaderResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for LatestCertificateRequestType {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "LATEST_CERTIFICATE_REQUEST_TYPE_UNSPECIFIED",
            Self::Pending => "LATEST_CERTIFICATE_REQUEST_TYPE_PENDING",
            Self::Settled => "LATEST_CERTIFICATE_REQUEST_TYPE_SETTLED",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for LatestCertificateRequestType {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "LATEST_CERTIFICATE_REQUEST_TYPE_UNSPECIFIED",
            "LATEST_CERTIFICATE_REQUEST_TYPE_PENDING",
            "LATEST_CERTIFICATE_REQUEST_TYPE_SETTLED",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = LatestCertificateRequestType;

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
                    "LATEST_CERTIFICATE_REQUEST_TYPE_UNSPECIFIED" => Ok(LatestCertificateRequestType::Unspecified),
                    "LATEST_CERTIFICATE_REQUEST_TYPE_PENDING" => Ok(LatestCertificateRequestType::Pending),
                    "LATEST_CERTIFICATE_REQUEST_TYPE_SETTLED" => Ok(LatestCertificateRequestType::Settled),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
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
