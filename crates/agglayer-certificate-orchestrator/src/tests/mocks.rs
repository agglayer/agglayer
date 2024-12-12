use agglayer_types::{Height, LocalNetworkStateData, NetworkId};
use mockall::mock;
use pessimistic_proof::local_exit_tree::hasher::Keccak256Hasher;
use pessimistic_proof::multi_batch_header::MultiBatchHeader;
use pessimistic_proof::LocalNetworkState;

use crate::{error::CertificationError, Certifier, CertifierOutput};

mock! {
    pub Certifier {}

#[async_trait::async_trait]
    impl Certifier for Certifier {
        async fn certify(
            &self,
            state: agglayer_types::LocalNetworkStateData,
            network_id: NetworkId,
            height: Height,
        ) -> Result<CertifierOutput, CertificationError>;

        async fn witness_execution(
            &self,
            certificate: &agglayer_types::Certificate,
            state: &mut LocalNetworkStateData
        ) -> Result<(MultiBatchHeader<Keccak256Hasher>, LocalNetworkState), CertificationError>;
    }
}

mock! {
    pub Provider{}
}

mock! {
    #[derive(Debug)]
    pub L1MiddlewareError{}

    impl std::error::Error for L1MiddlewareError {
    }

    impl std::fmt::Display for L1MiddlewareError {
        fn fmt<'a>(&self, f: &mut std::fmt::Formatter<'a>) -> std::fmt::Result;
    }

    impl ethers::providers::MiddlewareError for L1MiddlewareError {
        type Inner = Self;

        fn as_inner(&self) -> Option<&'static MockL1MiddlewareError>;

        fn from_err(err: Self) -> Self;
    }
}

mock! {
    #[derive(Debug)]
    pub ProviderError{}

    impl std::error::Error for ProviderError {
    }

    impl std::fmt::Display for ProviderError {
        fn fmt<'a>(&self, f: &mut std::fmt::Formatter<'a>) -> std::fmt::Result;
    }

    impl ethers::providers::RpcError for ProviderError {
        fn as_error_response(&self) -> Option<&'static ethers::providers::JsonRpcError>;
        fn as_serde_error(&self) -> Option<&'static serde_json::Error>;
    }

    impl Into<ethers::providers::ProviderError> for ProviderError {
        fn into(self) -> ethers::providers::ProviderError;
    }
}

mock! {
    #[derive(Debug)]
    pub L1Rpc {
        async fn request<T, R>(&self, method: &str, params: T) -> Result<R, MockProviderError>
        where
            T: std::fmt::Debug + serde::Serialize + Send + Sync,
            R: serde::de::DeserializeOwned + Send,
            R: 'static,
            T: 'static;
        async fn get_transaction(&self, tx_hash: ethers::types::H256) -> Result<Option<ethers::types::Transaction>, MockProviderError>;
    }

    impl ethers::providers::Middleware for L1Rpc {
        type Error = MockL1MiddlewareError;
        type Provider = ethers::providers::MockProvider;
        type Inner = Self;

        fn inner(&self) -> &'static Self;
    }
}
