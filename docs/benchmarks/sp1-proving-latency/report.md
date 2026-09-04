# SP1 proving-latency matrix

Is PP proving latency set by the PP program, or by the SNARK wrap?

## Run quality

- 66/66 requests fulfilled.
- Fulfillment strategy recorded by the network: `RESERVED` on every request.
- Fulfiller(s): `[Fulfiller] Polygon Dev`.
- Requests were issued strictly sequentially, round-robin across cells, order reversed on
  alternate repetitions. Latency is server-side `fulfilled_at - created_at`, so client-side
  ELF upload and local simulation are excluded by construction.

## Workloads

| workload | exits (own + imported) | ELF bytes | local cycles | local Succinct gas |
| -------- | ---------------------- | --------- | ------------ | --------------- |
| `noop` | — | 80,136 | 4,598 | 6,520 |
| `pp` | 1 + 1 | 484,656 | 1,071,395 | 1,826,863 |
| `pp-large` | 100 + 100 | 484,656 | 32,891,434 | 51,835,420 |

## Latency (server-side)

| workload | mode | n | min | median | max | sd | CV | queue (median) | cycles |
| -------- | ---- | - | --- | ------ | --- | -- | -- | -------------- | ------ |
| `noop` | compressed | 10 | 27s | 28s | 40s | 4.1s | 14% | 0.3s | 4,598 |
| `noop` | plonk | 6 | 1min51s | 2min42s | 3min28s | 38.5s | 23% | 0.4s | 4,598 |
| `noop` | groth16 | 6 | 44s | 46.5s | 1min03s | 7.0s | 14% | 0.3s | 4,598 |
| `pp` | compressed | 10 | 27s | 30s | 38s | 4.0s | 13% | 0.3s | 1,072,063 |
| `pp` | plonk | 7 | 2min29s | 3min06s | 3min34s | 23.5s | 13% | 0.3s | 1,072,063 |
| `pp` | groth16 | 6 | 41s | 46s | 49s | 2.9s | 6% | 0.4s | 1,072,063 |
| `pp-large` | compressed | 9 | 31s | 33s | 49s | 5.6s | 16% | 0.3s | 32,890,697 |
| `pp-large` | plonk | 6 | 2min15s | 2min35s | 3min00s | 18.3s | 12% | 0.3s | 32,890,697 |
| `pp-large` | groth16 | 6 | 44s | 46s | 53s | 3.4s | 7% | 0.3s | 32,890,697 |

`sd` is the sample standard deviation, omitted below n=3 where it is meaningless. `CV` is
the coefficient of variation (sd / mean) — dispersion scaled by magnitude, so cells with
very different latencies are comparable. `queue` is submission to first observed
`Assigned`, at 2s poll granularity.

## The same question on production requests

The synthetic matrix above varies the load deliberately. This is the opposite check: 99 real fulfilled requests from `0xacfe00ba538e753cd0a73ede2c5c27cc44a02fa8`, whose load varies on its own. Spanning 75 hours, mode(s) `plonk`, 1 distinct program(s).

| metric | n | mean | median | sd | CV | min | max |
| ------ | - | ---- | ------ | -- | -- | --- | --- |
| latency | 99 | 3min06s | 2min59s | 54.0s | 29% | 1min45s | 8min44s |
| cycles | 99 | 1,942,598 | 1,976,416 | 924,135 | 48% | 775,576 | 6,726,563 |
| Succinct gas | 99 | 2,539,417 | 2,416,264 | 1,499,826 | 59% | 1,337,493 | 10,729,309 |

Work varies 8.7x across these requests, yet it does not predict latency: r = -0.04, so the load accounts for about 0.1% of the spread.

## What the numbers decompose into

Pipeline floor (`noop`, an empty program, compressed): **28s**.

| workload | cycles | PP cost vs noop | 95% CI (bootstrap) | plonk wrap | groth16 wrap | plonk vs groth16 |
| -------- | ------ | ------------------ | ------------------ | ---------- | ------------ | ---------------- |
| `noop` | 4,598 | — | — | 2min14s | 18.5s | 1min56s |
| `pp` | 1,071,395 | 2s | -1s to 8s | 2min36s | 16s | 2min20s |
| `pp-large` | 32,891,434 | 5s | 2s to 9s | 2min02s | 13s | 1min49s |

`PP cost` is `<workload>_compressed - noop_compressed`: what the program itself adds
before any SNARK wrap. `plonk wrap` and `groth16 wrap` are
`<workload>_<mode> - <workload>_compressed`: what the wrap adds on top of the STARK.

## What this suggests

These measurements suggest latency is mostly set by the SNARK wrap, not by the PP or by how much work it does.

- Plonk medians across the programs measured sat between 2min35s and 3min06s, in no order matching the load.
- Before any wrap, the heaviest program measured added 5s over an empty program (95% CI 2s to 9s).
- Queue time was 0.3s median, so these are proving times rather than waiting times.

Caveat: n=6-10 per cell, run quickly. The direction looks consistent; the exact seconds are rough, and only the loads listed above were tested.

## Requests

**`noop` / compressed** (10 requests)

- 01: 28s — https://explorer.reserved.succinct.xyz/request/0x3b7e03c03236865d7d265f55d8b5c4d7e6fbdcbc75069199a61ca70dde5a13d7
- 02: 40s — https://explorer.reserved.succinct.xyz/request/0x63b1e3f980e82dc3e8e644d9c76261aaed2e3239d4da8dae2cd04148e2b82c2a
- 03: 33s — https://explorer.reserved.succinct.xyz/request/0x4832e31b8ffd3598214d0916cd1f6773bccaaeaa6de2894bb4ac4f98c27bad1f
- 04: 29s — https://explorer.reserved.succinct.xyz/request/0x7aa6516afb970dfa47d597c118da947b42a43a3bda6f6f14a6d027f1de4a7a76
- 05: 30s — https://explorer.reserved.succinct.xyz/request/0x3f490f0c011d1b2f7512c0996ca21c6a7fb8db36a813f460bfe47eeead93c579
- 06: 28s — https://explorer.reserved.succinct.xyz/request/0x0de5f23d570ff6ee5ec83b3338f9c2a9e7bf5730c30dbfeabd8affbe990c0022
- 07: 28s — https://explorer.reserved.succinct.xyz/request/0x6072de39691e4733ba90c8758f608ba0dd0644f2f20462d72bde952a6f6da939
- 08: 27s — https://explorer.reserved.succinct.xyz/request/0xd917ef1ebd46476501a2ee0e632dba7cb134129ebf4f6c9c732b0ec203a1d854
- 09: 27s — https://explorer.reserved.succinct.xyz/request/0x388bd869e7988196ed57d70408ea43e5aaaa51304e18b9b4bd81125fc62ef9ac
- 10: 27s — https://explorer.reserved.succinct.xyz/request/0xc03f68cab3085a649cbb131a7ff3f951a2899ed20c80b6d36f00444bb395d77c

**`noop` / plonk** (6 requests)

- 01: 3min27s — https://explorer.reserved.succinct.xyz/request/0x2ebfd7a8cee467ecc04216456e6a20b499336d0b7104795b4cdd40840b32c6bd
- 02: 3min28s — https://explorer.reserved.succinct.xyz/request/0x3de59b1968fd4146b7b5a38cc718a4750f9aebbac51234f0ec789361440e0e8c
- 03: 1min51s — https://explorer.reserved.succinct.xyz/request/0x72e2ab2d9fe515d785c5e6d317f060f1beb16cba586f3997bd9cb7105ebffd8d
- 04: 2min27s — https://explorer.reserved.succinct.xyz/request/0xb3d2876397c8e57b225e7b57d64a371e352e1df41e5f58cb8e3c7f973c1ba227
- 05: 2min57s — https://explorer.reserved.succinct.xyz/request/0xdb4875c7c231b95d5a2b1dc3f9f212054d45675d4f6c0a925f77e3ec5fd10ac3
- 06: 2min25s — https://explorer.reserved.succinct.xyz/request/0xbcfd8c7d44aa75d0c8da07bec251253ca4835e7503c7916e9db2eac33a308b86

**`noop` / groth16** (6 requests)

- 01: 47s — https://explorer.reserved.succinct.xyz/request/0x1abb80781243e6e5c0d8fec651431bb873aa51626f4cae89eed0e622e8903351
- 02: 46s — https://explorer.reserved.succinct.xyz/request/0x360e0a89996c782e710b21f45231d389ba228a90ceae1eea97d25475f8120b25
- 03: 44s — https://explorer.reserved.succinct.xyz/request/0xfd6bbbfa48ab92268251c8325f8cc97f48d17f4832e3a426d9cc045a1f5ccef9
- 04: 48s — https://explorer.reserved.succinct.xyz/request/0x9793dbf0596d820fe9198aeb4adac0d34a7b2a788f6c2baff1b01b7af7c9763f
- 05: 46s — https://explorer.reserved.succinct.xyz/request/0x6ad98837c61c0e9165dd86c2616ed607cfd57fe82f0da8785518dc6978503e45
- 06: 1min03s — https://explorer.reserved.succinct.xyz/request/0xce47174240df9059a2ae677935e30b20fe15c48ee77ea882af33400ecad77fa2

**`pp` / compressed** (10 requests)

- 01: 37s — https://explorer.reserved.succinct.xyz/request/0x032ab48433eba9de4a651ae0b645ceca4f5f2803e4c6d0d2f760f7964f1fb26e
- 02: 29s — https://explorer.reserved.succinct.xyz/request/0xa125cfcf4bfe879512a9efd1f0655367f3fe06885114402d76e0dffa64411884
- 03: 38s — https://explorer.reserved.succinct.xyz/request/0xca1fd10f86baa3d81b30467649725670134e34a7154154844f64820ffa72176f
- 04: 29s — https://explorer.reserved.succinct.xyz/request/0xb4935d63d2e9e2d39d8a908cbcbc9efb262ef1fdf0354414acdc8b8aca3a60b3
- 05: 27s — https://explorer.reserved.succinct.xyz/request/0x722db3f7f31a2926f4de78231cbcaa0c2bb9dd97b109549a5eb97174cca5477b
- 06: 28s — https://explorer.reserved.succinct.xyz/request/0x026b6f38792f2665c473459688e1ed6d35809615a5bf6af2d0ddb920250345c9
- 07: 31s — https://explorer.reserved.succinct.xyz/request/0x03a2ec45c0a675692783f3c3d3b8a39b126c37a42ba99afcdd11c541b08d7074
- 08: 31s — https://explorer.reserved.succinct.xyz/request/0x412f56e88b0ac9a610ec916f181ea3adcb49b531c99cf65cfab7e7b50a0088ce
- 09: 29s — https://explorer.reserved.succinct.xyz/request/0xd7236f158c1ef7ae93eb4c791a5b1d5898dc0bb870237c8f490644a6431334f7
- 10: 36s — https://explorer.reserved.succinct.xyz/request/0x9bc5a7dbadddc6233ca325a7bc6eb035159ab370a89712ef457ddc8630eab178

**`pp` / plonk** (7 requests)

- 01: 3min25s — https://explorer.reserved.succinct.xyz/request/0x2e8dc6410e9479ea9d4d96d3b28b2a4733e3dff58fde6c37fd5991244bfe4441
- 02: 2min29s — https://explorer.reserved.succinct.xyz/request/0xa991265de073e49ae41540ace195f848476a23cfaa0de23f4a2500085772be2e
- 03: 3min06s — https://explorer.reserved.succinct.xyz/request/0x1328f2150eeaafa56cbe24226a2add512939974703770ad3135f58448d838866
- 04: 2min50s — https://explorer.reserved.succinct.xyz/request/0x527d04aa42dd73849419e612c91542e421b382813c963485fc8e2d144512d995
- 05: 3min25s — https://explorer.reserved.succinct.xyz/request/0x1e00bb75adcf28994579fdae66158e3d1d92849c5ea4398fe0cb05f4f05b0d52
- 06: 2min53s — https://explorer.reserved.succinct.xyz/request/0xab395f422145ba23ced1c2ba959e4174a2f92df13aeb30b3c8fd5ce0b9f9b30a
- 07: 3min34s — https://explorer.reserved.succinct.xyz/request/0xc143f191497abaa285cc796f553b9e1121b3c03b297acf76dc3337fcebd8e113

**`pp` / groth16** (6 requests)

- 01: 46s — https://explorer.reserved.succinct.xyz/request/0xdc66367088221fe93a406afd5e85e7111b0fc75538f509e616abcfd6d4e5ad3e
- 02: 48s — https://explorer.reserved.succinct.xyz/request/0x6a36c28ac44dc4885fd845a8abd55ca52d5cbdce604ed172060e4cdf5ff5fc37
- 03: 46s — https://explorer.reserved.succinct.xyz/request/0x793708db5080e0f036913e57ba81a264557690c169a7783a02665114cc421c7a
- 04: 41s — https://explorer.reserved.succinct.xyz/request/0x3c4871e4376977c4dd720c59335737f3188b3ab400af97126ce2d19943e47f25
- 05: 44s — https://explorer.reserved.succinct.xyz/request/0xcbb397d1d1790b216a54ba43232685b711b060e2f2f1e943d80e1559c34dd085
- 06: 49s — https://explorer.reserved.succinct.xyz/request/0x6b84ca506ac39b5a8d695b6e8685b14f4883e9e23c977d832ed191873da18533

**`pp-large` / compressed** (9 requests)

- 01: 33s — https://explorer.reserved.succinct.xyz/request/0x43a1286bc4b8627723b2dcde9a2cc4a48cad81a35edb5344c09ae5d7ddfc5665
- 02: 32s — https://explorer.reserved.succinct.xyz/request/0x0665769130132d915756c48006a4c480cd9fe7fe4c745ef99ee2342a7891c3bd
- 03: 32s — https://explorer.reserved.succinct.xyz/request/0xa9cb608d1f7e39e8ea7b057626137ca1b9c91c009b7d3a6876a882cc2a262cec
- 04: 33s — https://explorer.reserved.succinct.xyz/request/0xf31aef27cc8407e39ceccc6193cd96349d0032743cc9c0424f9e4f5b2c2fe1fb
- 05: 38s — https://explorer.reserved.succinct.xyz/request/0x44d0dce53b434836f09dac15fa59713fa81ebfbd7be6be2b5d991206bcca7059
- 06: 31s — https://explorer.reserved.succinct.xyz/request/0x836dd6f9d1dadb78b97fb8f07cbe0cac198563e32f5be7e2719013a1c70ee03e
- 07: 34s — https://explorer.reserved.succinct.xyz/request/0x347857c179e953de2f60d5cfa5971c2a4714fb3e39fc149116c1ad1dfb5cd958
- 08: 33s — https://explorer.reserved.succinct.xyz/request/0x333123ca1359979123a4594cce526c62d87cd3977e9f9c973e1ab44eb780a17e
- 09: 49s — https://explorer.reserved.succinct.xyz/request/0xf79c2f4ffd5ecc110cc6c5c843eb9b9e7c3a2f060ef65daf52218541fb4e35ec

**`pp-large` / plonk** (6 requests)

- 01: 2min37s — https://explorer.reserved.succinct.xyz/request/0xa1c5210fb111f2ee65bf241061e0b84fd853a58492c9fb350cb5cd0a3b6d2f69
- 02: 2min24s — https://explorer.reserved.succinct.xyz/request/0x26bfe4ce6a14911c3746f4ca6f7c1acd0c7f25e34ecbb5f9d7600bc6fe9bddc2
- 03: 2min33s — https://explorer.reserved.succinct.xyz/request/0x015c64a65d90339e1176e31fec6211080474ef99a99ba8d3635cf1f25325708c
- 04: 3min00s — https://explorer.reserved.succinct.xyz/request/0x605db3fa25f66d7ae87d8797a601afa15cc454e70c0a0c71b2ad0b643403e882
- 05: 2min59s — https://explorer.reserved.succinct.xyz/request/0x53352b236a8612ed35c3db15f768f2bbdead4e3150ed9e5823eda3232ff25e3f
- 06: 2min15s — https://explorer.reserved.succinct.xyz/request/0x4b1c446749b1aac33641bfca36b2c89613b2962df2315d0f74da84b04539ccef

**`pp-large` / groth16** (6 requests)

- 01: 45s — https://explorer.reserved.succinct.xyz/request/0x5d45581b8b145a7548e98ee4c0a94b60bcb5dc37b326721f52601d4d0954303e
- 02: 45s — https://explorer.reserved.succinct.xyz/request/0x4870a57220660fb211a078446b3d930ceb5666b438f82d84c70e3f07a3c3c948
- 03: 47s — https://explorer.reserved.succinct.xyz/request/0xb3af35c7a21009921843952c6bf97d6871e6abd6f0b4304a1f0a32f5cfacd72f
- 04: 44s — https://explorer.reserved.succinct.xyz/request/0x053ae9359c3c66a79d8144f6e9762a09a482f8641b91ca4633b9c565e7cf8e06
- 05: 49s — https://explorer.reserved.succinct.xyz/request/0xb4d68c5f48a756a164eaa37b26b5ab5a807436bea7057e4dde14642fd4aad23b
- 06: 53s — https://explorer.reserved.succinct.xyz/request/0xb768e410e4925d1aa4e56b2317f993acdc94b3c5c95131d7d1210a8eb22cb22e

## Parameters

```json
[
  {
    "source": "2026-09-02T15-07-15Z.json",
    "adaptive_threshold": 0.25,
    "circuit_version": "v6.1.0",
    "n_exits": 1,
    "n_imported_exits": 1,
    "network_mode": "Reserved",
    "reps": 2,
    "rpc_url": "https://rpc.production.succinct.xyz",
    "sequential": true,
    "strategy": "RESERVED",
    "timeout_secs": 1200,
    "warmup_discarded": true
  },
  {
    "source": "2026-09-03T11-03-52Z.json",
    "adaptive_threshold": 0.25,
    "circuit_version": "v6.1.0",
    "n_exits": 1,
    "n_imported_exits": 1,
    "network_mode": "Reserved",
    "reps": 2,
    "rpc_url": "https://rpc.production.succinct.xyz",
    "sequential": true,
    "strategy": "RESERVED",
    "timeout_secs": 2400,
    "warmup_discarded": false
  },
  {
    "source": "2026-09-03T13-16-33Z.json",
    "adaptive_threshold": 999.0,
    "circuit_version": "v6.1.0",
    "modes": "compressed",
    "n_exits": 1,
    "n_imported_exits": 1,
    "network_mode": "Reserved",
    "reps": 7,
    "rpc_url": "https://rpc.production.succinct.xyz",
    "sequential": true,
    "strategy": "RESERVED",
    "timeout_secs": 2400,
    "warmup_discarded": false,
    "workloads_run": "noop,pp,pp-large"
  },
  {
    "source": "2026-09-03T13-35-15Z.json",
    "adaptive_threshold": 999.0,
    "circuit_version": "v6.1.0",
    "modes": "groth16",
    "n_exits": 1,
    "n_imported_exits": 1,
    "network_mode": "Reserved",
    "reps": 4,
    "rpc_url": "https://rpc.production.succinct.xyz",
    "sequential": true,
    "strategy": "RESERVED",
    "timeout_secs": 2400,
    "warmup_discarded": false,
    "workloads_run": "noop,pp,pp-large"
  },
  {
    "source": "2026-09-03T14-25-19Z.json",
    "adaptive_threshold": 999.0,
    "circuit_version": "v6.1.0",
    "modes": "plonk",
    "n_exits": 1,
    "n_imported_exits": 1,
    "network_mode": "Reserved",
    "reps": 4,
    "rpc_url": "https://rpc.production.succinct.xyz",
    "sequential": true,
    "strategy": "RESERVED",
    "timeout_secs": 2400,
    "warmup_discarded": false,
    "workloads_run": "noop,pp,pp-large"
  }
]
```

## Environment

- Apple M4 Pro, 14 cores, Darwin 25.5.0, rustc 1.98.0 (88d9e12ae 2026-08-18)
- Latency is measured on the SP1 prover network, not locally. The host only submits and
  polls, so host load does not affect the reported numbers.
- Poll granularity 2s, so `queue` and the client-side split carry ±2s. Server-side
  `fulfilled_at - created_at` has 1s resolution.

## Comparing runs

Only comparable against another run with a matching Parameters block and the same circuit
version. Absolute latencies move with network conditions and with which fulfiller picks up
the request; the *differences* within one run are the durable result.
