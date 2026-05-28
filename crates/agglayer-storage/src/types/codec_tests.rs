macro_rules! codec_tests {
    ($value:expr $(,)?) => {
        #[test]
        fn codec_bolero_round_trip() {
            fn check<T>(_: &T)
            where
                T: $crate::schema::Codec
                    + std::fmt::Debug
                    + PartialEq
                    + 'static
                    + for<'a> arbitrary::Arbitrary<'a>,
            {
                bolero::check!()
                    .with_arbitrary::<T>()
                    .for_each(|value: &T| {
                        let encoded = value.encode().expect("codec encode must succeed");
                        let decoded = T::decode(&encoded).expect("encoded value must decode");

                        assert_eq!(&decoded, value);
                    });
            }

            check(&$value);
        }

        #[test]
        fn codec_bolero_decode_never_panics() {
            fn check<T>(_: &T)
            where
                T: $crate::schema::Codec,
            {
                bolero::check!().for_each(|bytes| {
                    let _ = T::decode(bytes);
                });
            }

            check(&$value);
        }

        #[test]
        fn codec_encoding_snapshot() {
            fn check<T>(value: T)
            where
                T: $crate::schema::Codec + std::fmt::Debug + PartialEq,
            {
                let encoded = value.encode().expect("codec encode must succeed");
                let encoded_hex = hex::encode(&encoded);
                insta::assert_snapshot!("codec_encoding_snapshot", encoded_hex);

                let decoded = T::decode(&encoded).expect("encoded value must decode");
                assert_eq!(decoded, value);
            }

            check($value);
        }
    };
}

pub(crate) use codec_tests;
