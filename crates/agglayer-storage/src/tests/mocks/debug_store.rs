use mockall::mock;

use crate::stores::{DebugReader, DebugWriter};

mock! {
    pub DebugStore {}

    impl DebugWriter for DebugStore {
        fn add_certificate(&self, certificate: &agglayer_types::Certificate) -> Result<(), crate::error::Error>;
    }

    impl DebugReader for DebugStore {
        fn get_certificate(&self, certificate_id: &agglayer_types::CertificateId)
                -> Result<Option<agglayer_types::Certificate>, crate::error::Error>;
    }
}
