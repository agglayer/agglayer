use super::*;

#[test]
fn client_error_round_trip() {
    for kind in [
        ClientErrorType::NonceAlreadyUsed,
        ClientErrorType::SettlementSucceededElsewhere,
        ClientErrorType::AbandonedByAdmin,
    ] {
        let error = ClientError {
            kind,
            message: "some client error".to_string(),
        };

        let proto: v0::ClientError = (&error).into();
        let decoded = ClientError::try_from(proto).unwrap();

        assert_eq!(decoded, error);
    }
}

#[test]
fn invalid_client_error_type_fails() {
    let proto = v0::ClientError {
        error_type: 999,
        error_message: "oops".to_string(),
    };

    assert!(ClientError::try_from(proto).is_err());
}
