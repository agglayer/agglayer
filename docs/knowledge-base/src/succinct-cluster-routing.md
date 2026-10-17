# Per-network Succinct RPC endpoints

Agglayer optionally accepts a top-level mapping of certificate origin network
IDs to Succinct RPC URLs. The existing prover endpoint remains the default for
networks without an override:

```toml
[prover.network-prover]
sp1-cluster-endpoint = "https://shared-cluster.example/"

[succinct-cluster]
48 = "https://private-cluster.example/"
```

An explicit network entry takes precedence over
`[prover.network-prover].sp1-cluster-endpoint`. Networks without an entry retain
that existing endpoint. If `sp1-cluster-endpoint` is omitted, its existing
`SP1_CLUSTER_ENDPOINT` environment-variable default applies, falling back to
`https://rpc.production.succinct.xyz/`. The `[succinct-cluster]` section accepts
only network IDs as keys and can be omitted entirely without changing existing
behavior. Multiple networks can name the same endpoint; they share a buffered
prover service.

An override is binding for every Succinct RPC operation for that network. The
node prepares all configured routes before startup completes; initialization
failure prevents startup. Request errors and timeouts surface errors and never
redirect the network to another cluster. SP1 artifact transfers still use URLs
supplied by the selected cluster. CPU and mock provers retain their configured
modes.

At startup, one log is emitted per applied override, in network-ID order. It
shows only the endpoint origin (scheme, host and port), so credentials in the
URL's user-info, path or query never reach the logs:

```text
INFO Using configured Succinct RPC endpoint network_id=48 rpc_origin=https://private-cluster.example
```

Restart the node after changing the configuration. Removing an entry restores
the existing prover endpoint for that network on the next startup.
