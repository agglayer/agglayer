# Agglayer Clock

The `agglayer-clock` crate provides timing and epoch management for the Agglayer system. It defines the pace of the Agglayer in terms of epochs and supports two clock implementations:

- **BlockClock**: Synchronizes with L1 blockchain blocks (production use)
- **TimeClock**: Time-based epochs (testing and development)

## Features

- **Epoch Management**: Tracks and broadcasts epoch transitions
- **L1 Integration**: Synchronizes with Ethereum L1 blocks via WebSocket
- **Observability**: Comprehensive metrics and structured logging
- **Resilience**: Automatic reconnection and error recovery

## Observability

### Metrics

All clock metrics are managed in the `agglayer-telemetry` crate under the `clock` module. Key metrics include:

- `agglayer_node_clock_current_block_height` - Current block height processed
- `agglayer_node_clock_current_epoch` - **Critical:** Current epoch number
- `agglayer_node_clock_connection_status` - WebSocket connection status
- `agglayer_node_clock_health_status` - Overall health indicator

See the `agglayer-telemetry` crate documentation for complete metrics list.

## Configuration

The clock is configured through the main Agglayer configuration:

```toml
[epoch.block-clock]
epoch-duration = 6        # Number of L1 blocks per epoch
genesis-block = 18000000  # L1 block number to start from

[l1]
ws-node-url = "wss://ethereum-node.example.com"
max-reconnection-elapsed-time = "5m"
```
