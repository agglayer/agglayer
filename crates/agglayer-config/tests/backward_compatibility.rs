use std::{io::Read, net::SocketAddr, path::PathBuf, str::FromStr, time::Duration};

use agglayer_config::{
    epoch::TimeClockConfig,
    log::{LogFormat, LogOutput},
    AuthConfig, Config, Epoch, PrivateKey,
};
use ethers::types::H160;
use pretty_assertions::assert_eq;
use url::Url;

fn backward_compatibility_config(file: &str) -> PathBuf {
    env!("CARGO_MANIFEST_DIR")
        .parse::<PathBuf>()
        .unwrap()
        .join("./tests/")
        .join(file)
}

#[test]
fn all_config() {
    let mut reader = String::new();
    let mut file =
        std::fs::File::open(backward_compatibility_config("backward_compatibility.toml")).unwrap();
    file.read_to_string(&mut reader).unwrap();

    let config: Config = toml::from_str(&reader).unwrap();

    let mut base_config = Config::default();

    base_config
        .full_node_rpcs
        .insert(1, Url::parse("http://zkevm-node:8123").unwrap());
    // Update log
    base_config.log.format = LogFormat::Json;

    // Update rpc
    base_config.outbound.rpc.settle.max_retries = 10;
    base_config.outbound.rpc.settle.retry_interval = Duration::from_secs(10);
    base_config.outbound.rpc.settle.confirmations = 10;

    // Update L1
    base_config.l1.chain_id = 1338;

    // Update telemetry
    base_config.telemetry.addr = SocketAddr::V4(std::net::SocketAddrV4::new(
        std::net::Ipv4Addr::new(0, 0, 0, 0),
        3030,
    ));

    base_config.epoch = Epoch::TimeClock(TimeClockConfig {
        epoch_duration: Duration::from_secs(10),
    });
    base_config.shutdown.runtime_timeout = Duration::from_secs(10);
    base_config
        .certificate_orchestrator
        .input_backpressure_buffer_size = 10000;

    base_config.auth = AuthConfig::Local(agglayer_config::LocalConfig {
        private_keys: vec![PrivateKey {
            path: PathBuf::from_str("/pk/agglayer.keystore").unwrap(),
            password: "testonly".into(),
        }],
    });
    assert_eq!(config, base_config);
}

#[test]
fn kurtosis() {
    let mut reader = String::new();
    let mut file = std::fs::File::open(backward_compatibility_config(
        "backward_compatibility-kurtosis.toml",
    ))
    .unwrap();
    file.read_to_string(&mut reader).unwrap();

    let config: Config = toml::from_str(&reader).unwrap();

    let mut base_config = Config::default();

    base_config
        .full_node_rpcs
        .insert(1, Url::parse("http://zkevm-node:8123").unwrap());

    base_config.log.format = LogFormat::Json;
    base_config.log.outputs = vec![LogOutput::Stderr];

    base_config.rpc.port = 8080;

    base_config.l1.node_url =
        Url::parse("https://rpc-cdk-validium-cardona-03-zkevm.polygondev.tools").unwrap();
    base_config.l1.rollup_manager_contract =
        H160::from_str("0x32d33D5137a7cFFb54c5Bf8371172bcEc5f310ff").unwrap();
    base_config.l1.chain_id = 0;

    base_config.telemetry.addr = SocketAddr::V4(std::net::SocketAddrV4::new(
        std::net::Ipv4Addr::new(0, 0, 0, 0),
        8080,
    ));

    base_config.auth = AuthConfig::Local(agglayer_config::LocalConfig {
        private_keys: vec![PrivateKey {
            path: PathBuf::from_str("/etc/zkevm/agglayer.keystore").unwrap(),
            password: "test".into(),
        }],
    });

    assert_eq!(config, base_config);
}
