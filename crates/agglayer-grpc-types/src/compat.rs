pub mod v1 {
    use agglayer_types::Digest;

    use crate::protocol::types::v1;

    impl TryFrom<v1::FixedBytes32> for Digest {
        type Error = ();

        fn try_from(digest: v1::FixedBytes32) -> Result<Self, Self::Error> {
            digest.value.to_vec().try_into().map_err(|_| ())
        }
    }
}
