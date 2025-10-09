# Status, health and readiness checks

It is often useful to know about the state of the node, especially after it has been started.
However, information about the state of the node is quite opaque aside from telemetry.

This document lists the signals that we would like to collect and display as a status health check of the node:

1. Age of the certificate
2. Frequency and duration of RPC/smart contract call failures with L1
3. Any failure in submitting a certificate and settling it on L1
4. Any storage call returning an error
5. Clock not making progress
6. Multiple errors in trying to retrieve the signing key from GCP
7. No new RPC calls to the node within an hour (an epoch)
8. Certificate status invariant violation (e.g. settled certs appearing in PendingStore, pending certs appearing in EpochStore, etc.)
9. Rate limiter not working
10. Taking too long to perform a read or write to RocksDB

## Heuristics in determining healthiness

Each signal should also be paired with a criteria in determining how it can be considered healthy or not.

1. A certificate of an age > 1 day is problematic.
2. Something is wrong if L1 takes more than 3 minutes of RPC to respond.
3. Certificate submission and settlement should never emit any errors.
4. Storage call returning errors is unusual but may be fixed by a node restart.
5. If epoch has not progressed in twice the expected amount of time (i.e. more than 2 hours if each epoch is about an hour), it hints at an unhealthy status.
6. Signing key retrieval from GCP emitting an error once or twice is tolerated, but more than that hints at a latent issue.
7. Not having new RPC calls to the node within an epoch is highly unusual and should be flagged.
8. Any certificate status invariant violation should be reported and is highly unusual.
9. Rate limiter throwing errors and not working as expected would cause us to spend a lot on tx fees on L1.
10. Taking around 10 seconds or more to write to RocksDB is unusual.

## API

In addition, there are a few API setups in which we communicate unhealthiness back to the client who requests for this information:

1. Have multiple endpoints that is specialized for a particular service, and if the signals for that service is unhealthy, we return the corresponding HTTP status code (503 Service Unavailable or 500 Internal server error). Healthy services would simply just return a 200 OK. We may return a JSON as well if desired.

2. One dedicated endpoint containing a JSON that contains fields corresponding to the service's status and the signals/telemtry used to determine the healthiness of the checkpoint.

## Healthiness schema

In terms of the scheme in determining the healthiness of a service, we also have a few options:

1. A simple boolean that indicates healthiness. `true` being healthy; `false` unhealthy. We could also include the telemetry used to determine the healthiness of that service.

2. A score showing the overall and individual healthiness of each service.

3. A red-yellow-green scheme to indicate services that requires attention, should be monitored and are healthy respectively.
