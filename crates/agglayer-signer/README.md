# agglayer-signer

This crate provides a [`Signer`](trait@alloy_signer::Signer)
implementation that can house either a local keystore or a GCP KMS signer.
(more signers can be added in the future)

See: [`ConfiguredSigner`](enum@ConfiguredSigner)
