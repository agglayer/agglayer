# Commit message examples

```text
fix(prover): reject malformed proof payload
```

```text
feat(grpc): add rate limiting to certificate submission

Introduces a token-bucket rate limiter per chain ID to prevent
excessive certificate submissions from overwhelming the pipeline.

CONFIG-CHANGE: New `rate_limit` section in agglayer.toml.
```
