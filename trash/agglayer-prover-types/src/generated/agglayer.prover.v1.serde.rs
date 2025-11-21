// @generated
impl serde::Serialize for ErrorKind {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "ERROR_KIND_UNSPECIFIED",
            Self::UnableToExecuteProver => "ERROR_KIND_UNABLE_TO_EXECUTE_PROVER",
            Self::ProverFailed => "ERROR_KIND_PROVER_FAILED",
            Self::ProofVerificationFailed => "ERROR_KIND_PROOF_VERIFICATION_FAILED",
            Self::ExecutorFailed => "ERROR_KIND_EXECUTOR_FAILED",
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
            "ERROR_KIND_UNABLE_TO_EXECUTE_PROVER",
            "ERROR_KIND_PROVER_FAILED",
            "ERROR_KIND_PROOF_VERIFICATION_FAILED",
            "ERROR_KIND_EXECUTOR_FAILED",
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
                    "ERROR_KIND_UNABLE_TO_EXECUTE_PROVER" => Ok(ErrorKind::UnableToExecuteProver),
                    "ERROR_KIND_PROVER_FAILED" => Ok(ErrorKind::ProverFailed),
                    "ERROR_KIND_PROOF_VERIFICATION_FAILED" => Ok(ErrorKind::ProofVerificationFailed),
                    "ERROR_KIND_EXECUTOR_FAILED" => Ok(ErrorKind::ExecutorFailed),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for GenerateProofError {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.error.is_empty() {
            len += 1;
        }
        if self.error_type != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("agglayer.prover.v1.GenerateProofError", len)?;
        if !self.error.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("error", pbjson::private::base64::encode(&self.error).as_str())?;
        }
        if self.error_type != 0 {
            let v = ErrorKind::try_from(self.error_type)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.error_type)))?;
            struct_ser.serialize_field("errorType", &v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GenerateProofError {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "error",
            "error_type",
            "errorType",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Error,
            ErrorType,
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
                            "error" => Ok(GeneratedField::Error),
                            "errorType" | "error_type" => Ok(GeneratedField::ErrorType),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GenerateProofError;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct agglayer.prover.v1.GenerateProofError")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GenerateProofError, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut error__ = None;
                let mut error_type__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Error => {
                            if error__.is_some() {
                                return Err(serde::de::Error::duplicate_field("error"));
                            }
                            error__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ErrorType => {
                            if error_type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("errorType"));
                            }
                            error_type__ = Some(map_.next_value::<ErrorKind>()? as i32);
                        }
                    }
                }
                Ok(GenerateProofError {
                    error: error__.unwrap_or_default(),
                    error_type: error_type__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("agglayer.prover.v1.GenerateProofError", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GenerateProofRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.stdin.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("agglayer.prover.v1.GenerateProofRequest", len)?;
        if let Some(v) = self.stdin.as_ref() {
            match v {
                generate_proof_request::Stdin::Sp1Stdin(v) => {
                    #[allow(clippy::needless_borrow)]
                    #[allow(clippy::needless_borrows_for_generic_args)]
                    struct_ser.serialize_field("sp1Stdin", pbjson::private::base64::encode(&v).as_str())?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GenerateProofRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "sp1_stdin",
            "sp1Stdin",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Sp1Stdin,
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
                            "sp1Stdin" | "sp1_stdin" => Ok(GeneratedField::Sp1Stdin),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GenerateProofRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct agglayer.prover.v1.GenerateProofRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GenerateProofRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut stdin__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Sp1Stdin => {
                            if stdin__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sp1Stdin"));
                            }
                            stdin__ = map_.next_value::<::std::option::Option<::pbjson::private::BytesDeserialize<_>>>()?.map(|x| generate_proof_request::Stdin::Sp1Stdin(x.0));
                        }
                    }
                }
                Ok(GenerateProofRequest {
                    stdin: stdin__,
                })
            }
        }
        deserializer.deserialize_struct("agglayer.prover.v1.GenerateProofRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GenerateProofResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.proof.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("agglayer.prover.v1.GenerateProofResponse", len)?;
        if !self.proof.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("proof", pbjson::private::base64::encode(&self.proof).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GenerateProofResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proof",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Proof,
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
                            "proof" => Ok(GeneratedField::Proof),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GenerateProofResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct agglayer.prover.v1.GenerateProofResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GenerateProofResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proof__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Proof => {
                            if proof__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proof"));
                            }
                            proof__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(GenerateProofResponse {
                    proof: proof__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("agglayer.prover.v1.GenerateProofResponse", FIELDS, GeneratedVisitor)
    }
}
