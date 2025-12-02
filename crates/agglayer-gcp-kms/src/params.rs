//! The [`KMS`] struct provides functionality to create a GCP KMS signer from a
//! configuration. This struct is used to initialize and configure a Google
//! Cloud KMS signer.

use std::{fmt::Display, str::FromStr};

use agglayer_config::GcpKmsConfig;
use tracing::warn;

pub use crate::error::Error;

const GOOGLE_PROJECT_ID: &str = "GOOGLE_PROJECT_ID";
const GOOGLE_LOCATION: &str = "GOOGLE_LOCATION";
const GOOGLE_KEYRING: &str = "GOOGLE_KEYRING";
const GOOGLE_KEY_NAME_LEGACY: &str = "GOOGLE_KEY_NAME";
const GOOGLE_KEY_VERSION_LEGACY: &str = "GOOGLE_KEY_VERSION";
const GOOGLE_KEY_NAME_PP_SETTLEMENT: &str = "GOOGLE_KEY_NAME_PP_SETTLEMENT";
const GOOGLE_KEY_VERSION_PP_SETTLEMENT: &str = "GOOGLE_KEY_VERSION_PP_SETTLEMENT";
const GOOGLE_KEY_NAME_TX_SETTLEMENT: &str = "GOOGLE_KEY_NAME_TX_SETTLEMENT";
const GOOGLE_KEY_VERSION_TX_SETTLEMENT: &str = "GOOGLE_KEY_VERSION_TX_SETTLEMENT";

#[derive(Debug)]
pub struct KMSParameters {
    pub(crate) project_id: String,
    pub(crate) location: String,
    pub(crate) keyring_name: String,
    pub(crate) key_name_pp_settlement: String,
    pub(crate) key_version_pp_settlement: u64,
    pub(crate) key_name_tx_settlement: String,
    pub(crate) key_version_tx_settlement: u64,
}

// Helper function to get a value from environment variables or configuration
// with fallbacks.
// The function returns the value in the following order of precedence:
// 1. The primary environment variable.
// 2. The fallback environment variable (if provided).
// 3. The primary configuration value.
// 4. The fallback configuration value (if provided).
// If none are found, it returns an error.
fn from_env_or_conf<T>(
    env_key: &str,
    env_key_fallback: Option<&str>,
    config_value: &Option<T>,
    config_value_fallback: &Option<T>,
) -> Result<T, Error>
where
    T: FromStr + Clone + Display,
{
    if let Ok(value) = std::env::var(env_key) {
        return value
            .parse()
            .map_err(|_| Error::KmsConfig(env_key.to_string()));
    } else if let Some(env_key_fallback) = env_key_fallback {
        if let Ok(value) = std::env::var(env_key_fallback) {
            warn!("Fallback KMS env:{env_key}=>env:{env_key_fallback}");
            return value
                .parse()
                .map_err(|_| Error::KmsConfig(env_key_fallback.to_string()));
        }
    }

    if let Some(config_value) = config_value {
        return Ok(config_value.clone());
    } else if let Some(config_value_fallback) = config_value_fallback {
        warn!("Fallback KMS config env:{env_key}=>config:{config_value_fallback}");
        return Ok(config_value_fallback.clone());
    }

    Err(Error::KmsConfig(env_key.to_string()))
}

impl TryFrom<&GcpKmsConfig> for KMSParameters {
    type Error = Error;
    fn try_from(config: &GcpKmsConfig) -> Result<Self, Self::Error> {
        // Get configuration values from environment variables or config
        let project_id = from_env_or_conf(GOOGLE_PROJECT_ID, None, &config.project_id, &None)?;
        let location = from_env_or_conf(GOOGLE_LOCATION, None, &config.location, &None)?;
        let keyring_name = from_env_or_conf(GOOGLE_KEYRING, None, &config.keyring, &None)?;
        let key_name_pp_settlement = from_env_or_conf(
            GOOGLE_KEY_NAME_PP_SETTLEMENT,
            Some(GOOGLE_KEY_NAME_LEGACY),
            &config.pp_settlement_key_name,
            &config.key_name,
        )?;
        let key_version_pp_settlement = from_env_or_conf(
            GOOGLE_KEY_VERSION_PP_SETTLEMENT,
            Some(GOOGLE_KEY_VERSION_LEGACY),
            &config.pp_settlement_key_version,
            &config.key_version,
        )?;
        let key_name_tx_settlement = from_env_or_conf(
            GOOGLE_KEY_NAME_TX_SETTLEMENT,
            Some(GOOGLE_KEY_NAME_LEGACY),
            &config.tx_settlement_key_name,
            &config.key_name,
        )?;
        let key_version_tx_settlement = from_env_or_conf(
            GOOGLE_KEY_VERSION_TX_SETTLEMENT,
            Some(GOOGLE_KEY_VERSION_LEGACY),
            &config.tx_settlement_key_version,
            &config.key_version,
        )?;

        Ok(Self {
            project_id,
            location,
            keyring_name,
            key_name_pp_settlement,
            key_version_pp_settlement,
            key_name_tx_settlement,
            key_version_tx_settlement,
        })
    }
}

#[cfg(test)]
mod test {
    use agglayer_config::GcpKmsConfig;
    use serial_test::serial;

    struct SetEnvGuard {
        key: String,
        original_value: Option<String>,
    }
    impl SetEnvGuard {
        fn new(key: &str, value: Option<&str>) -> Self {
            let original_value = std::env::var(key).ok();
            if let Some(value) = value {
                unsafe {
                    std::env::set_var(key, value);
                }
            } else {
                unsafe {
                    std::env::remove_var(key);
                }
            }
            Self {
                key: key.to_string(),
                original_value,
            }
        }
    }
    impl Drop for SetEnvGuard {
        fn drop(&mut self) {
            if let Some(original_value) = &self.original_value {
                unsafe {
                    std::env::set_var(&self.key, original_value);
                }
            } else {
                unsafe {
                    std::env::remove_var(&self.key);
                }
            }
        }
    }

    fn set_env(unique: bool, distinct: bool) -> Vec<SetEnvGuard> {
        let enabled = unique || distinct;
        vec![
            SetEnvGuard::new("GOOGLE_PROJECT_ID", enabled.then_some("env_project_id")),
            SetEnvGuard::new("GOOGLE_LOCATION", enabled.then_some("env_location")),
            SetEnvGuard::new("GOOGLE_KEYRING", enabled.then_some("env_keyring")),
            SetEnvGuard::new(
                "GOOGLE_KEY_NAME_PP_SETTLEMENT",
                distinct.then_some("env_key_name_pp"),
            ),
            SetEnvGuard::new("GOOGLE_KEY_VERSION_PP_SETTLEMENT", distinct.then_some("1")),
            SetEnvGuard::new(
                "GOOGLE_KEY_NAME_TX_SETTLEMENT",
                distinct.then_some("env_key_name_tx"),
            ),
            SetEnvGuard::new("GOOGLE_KEY_VERSION_TX_SETTLEMENT", distinct.then_some("2")),
            SetEnvGuard::new("GOOGLE_KEY_NAME", unique.then_some("env_key_name")),
            SetEnvGuard::new("GOOGLE_KEY_VERSION", unique.then_some("3")),
        ]
    }
    fn set_config(unique: bool, distinct: bool) -> GcpKmsConfig {
        let enabled = unique || distinct;
        GcpKmsConfig {
            project_id: enabled.then_some("conf_project_id".to_string()),
            location: enabled.then_some("conf_location".to_string()),
            keyring: enabled.then_some("conf_keyring".to_string()),
            pp_settlement_key_name: distinct.then_some("conf_key_name_pp".to_string()),
            pp_settlement_key_version: distinct.then_some(1),
            tx_settlement_key_name: distinct.then_some("conf_key_name_tx".to_string()),
            tx_settlement_key_version: distinct.then_some(2),
            key_name: unique.then_some("conf_key_name".to_string()),
            key_version: unique.then_some(3),
        }
    }

    #[test]
    #[serial]
    fn test_disctinct_env_vars() {
        let _raii = set_env(false, true);
        let config = set_config(true, true);
        let params = crate::KMSParameters::try_from(&config).unwrap();
        assert_eq!(params.project_id, "env_project_id");
        assert_eq!(params.location, "env_location");
        assert_eq!(params.keyring_name, "env_keyring");
        assert_eq!(params.key_name_pp_settlement, "env_key_name_pp");
        assert_eq!(params.key_version_pp_settlement, 1);
        assert_eq!(params.key_name_tx_settlement, "env_key_name_tx");
        assert_eq!(params.key_version_tx_settlement, 2);
    }

    #[test]
    #[serial]
    fn test_unique_env_vars() {
        let _raii = set_env(true, false);
        let config = set_config(true, true);
        let params = crate::KMSParameters::try_from(&config).unwrap();
        assert_eq!(params.project_id, "env_project_id");
        assert_eq!(params.location, "env_location");
        assert_eq!(params.keyring_name, "env_keyring");
        assert_eq!(params.key_name_pp_settlement, "env_key_name");
        assert_eq!(params.key_version_pp_settlement, 3);
        assert_eq!(params.key_name_tx_settlement, "env_key_name");
        assert_eq!(params.key_version_tx_settlement, 3);
    }

    #[test]
    #[serial]
    fn test_distinct_config_vars() {
        let _raii = set_env(false, false);
        let config = set_config(false, true);
        let params = crate::KMSParameters::try_from(&config).unwrap();
        assert_eq!(params.project_id, "conf_project_id");
        assert_eq!(params.location, "conf_location");
        assert_eq!(params.keyring_name, "conf_keyring");
        assert_eq!(params.key_name_pp_settlement, "conf_key_name_pp");
        assert_eq!(params.key_version_pp_settlement, 1);
        assert_eq!(params.key_name_tx_settlement, "conf_key_name_tx");
        assert_eq!(params.key_version_tx_settlement, 2);
    }
    #[test]
    #[serial]
    fn test_unique_config_vars() {
        let _raii = set_env(false, false);
        let config = set_config(true, false);
        let params = crate::KMSParameters::try_from(&config).unwrap();
        assert_eq!(params.project_id, "conf_project_id");
        assert_eq!(params.location, "conf_location");
        assert_eq!(params.keyring_name, "conf_keyring");
        assert_eq!(params.key_name_pp_settlement, "conf_key_name");
        assert_eq!(params.key_version_pp_settlement, 3);
        assert_eq!(params.key_name_tx_settlement, "conf_key_name");
        assert_eq!(params.key_version_tx_settlement, 3);
    }
}
