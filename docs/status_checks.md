# Status, health and readiness checks

It is often useful to know about the state of the node, especially after it has been started.
However, information about the state of the node is quite opaque aside from telemetry.

This document lists the kinds of data that we would like to collect and display as a status health check of the node.

* Age of the certificate
* Frequency and duration of RPC/smart contract call failures with L1
  - If L1 takes more than 3 minutes of RPC to respond, something is wrong
* Any failure in submitting a certificate and settling it on L1
* Any storage call returning an error
  - If too many errors happen, maybe restarting the node is a good idea and would fix
* Clock not making progress
  - If the epoch has not moved in twice of the expected amount of time, something is wrong
* Multiple errors in trying to retrieve the signing key from GCP
* No new RPC calls to the node within an hour (an epoch)
  - Most likely a problem as we do expect traffic to settle funds across blockchains
* Certificate status invariant violation (e.g. settled certs appearing in PendingStore, pending certs appearing in EpochStore, etc.)
  - Most probably cannot perform this check cheaply, but nice to have
* Rate limiter not working
  - Would cause us to spend too much tx fees
* Taking too long to perform a read or write (something like 10 seconds) to RocksDB
