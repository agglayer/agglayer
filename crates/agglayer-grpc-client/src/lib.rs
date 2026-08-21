pub mod node {
    pub mod v1 {
        #![allow(clippy::needless_lifetimes)]
        // Generated client methods return `Result<_, tonic::Status>`, whose
        // `Err` variant exceeds the clippy 1.98 `result_large_err` threshold.
        #![allow(clippy::result_large_err)]
        use agglayer_grpc_types::node::v1::*;
        include!("generated/agglayer.node.v1.tonic.rs");
    }
}
