# Protobuf and gRPC

Agglayer uses protobuf as the schema boundary for gRPC and storage payloads.
This chapter defines file ownership,
crate responsibilities,
and safe evolution workflows.

## Proto layout

Schemas live under `proto/agglayer/`.

- `proto/agglayer/node/` contains public node/service definitions.
- `proto/agglayer/storage/` contains storage-related protobuf schemas.

Generation and compatibility are configured via:

- `buf.yaml`
- `buf.rust.gen.yaml`
- `buf.storage.gen.yaml`

## gRPC crate responsibilities

| Crate | Responsibility |
|---|---|
| `agglayer-grpc-api` | Service traits and API-facing request/response contracts |
| `agglayer-grpc-types` | Generated types and compatibility conversions |
| `agglayer-grpc-server` | Tonic server implementation and endpoint wiring |
| `agglayer-grpc-client` | Tonic client wrappers used by consumers/tests |

## Standard workflow for schema changes

1. Edit the `.proto` source file under `proto/agglayer/`.
2. Regenerate artifacts:

   ```bash
   cargo make generate-proto
   ```

3. Update server/client behavior in the relevant gRPC crates.
4. Run verification checks and ensure generated outputs are committed.

Never edit generated code by hand.
Generated files are outputs,
not the source of truth.

## Standard workflow for adding a new gRPC endpoint

1. Add or extend the service definition in `proto/agglayer/node/...`.
2. Regenerate protobuf and tonic outputs.
3. Add conversion logic in `agglayer-grpc-types` if required.
4. Implement server behavior in `agglayer-grpc-server`.
5. Add/update client calls in `agglayer-grpc-client`.
6. Add API and integration tests.

## Compatibility rules

- Do not rename or repurpose existing field numbers.
- Prefer adding new optional fields over changing existing meaning.
- Use `reserved` fields/messages when removing obsolete numbers or names.
- Keep wire compatibility in mind for rolling deployments.
