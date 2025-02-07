pub mod node {
    pub mod v1 {
        #![allow(clippy::needless_lifetimes)]
        use agglayer_grpc_types::node::v1::*;
        include!("generated/agglayer.node.v1.tonic.rs");
    }
}
