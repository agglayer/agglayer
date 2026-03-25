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
- `agglayer_node_clock_connection_status` - WebSocket connection status (1=connected, 0=disconnected)
- `agglayer_node_clock_health_status` - Overall health indicator (1=healthy, 0.5=starting, 0.25=degraded, 0=unhealthy)
- `agglayer_node_clock_reconnection_attempts_total` - Total number of reconnection attempts
- `agglayer_node_clock_blocks_subscription_lag_total` - Total number of subscription lag events
- `agglayer_node_clock_connection_errors_total` - Total number of connection errors

## Configuration

The clock is configured through the main Agglayer configuration:

```toml
[epoch.block-clock]
epoch-duration = 6        # Number of L1 blocks per epoch
genesis-block = 18000000  # L1 block number to start from

[l1]
ws-node-url = "wss://ethereum-node.example.com"
connect-attempt-timeout = "3s"
```
