use std::env;

use tracing::warn;

/// Default port for given service.
pub trait PortDefaults {
    /// The default port number for this service.
    const DEFAULT: u16;

    /// Environment variable to load the port number from if it's not specified
    /// in the configuration file.
    const ENV_VAR: Option<&str>;
}

/// Port number parametrized by the defaults.
#[derive(Copy, educe::Educe, serde::Serialize, serde::Deserialize)]
#[educe(Debug, Eq, PartialEq, Ord, PartialOrd, Clone)]
#[serde(from = "u16", into = "u16")]
pub struct Port<D> {
    port: u16,
    _defaults: std::marker::PhantomData<fn() -> D>,
}

impl<D> Port<D> {
    pub const fn new(port: u16) -> Self {
        let _defaults = std::marker::PhantomData;
        Self { port, _defaults }
    }

    pub const fn as_u16(&self) -> u16 {
        self.port
    }

    pub fn from_env_var(env_var: &str) -> Option<Self> {
        env::var(env_var)
            .inspect_err(|err| match err {
                env::VarError::NotPresent => (),
                env::VarError::NotUnicode(_) => warn!("Contents of ${env_var} not Unicode"),
            })
            .ok()?
            .parse()
            .inspect_err(|_| warn!("Variable ${env_var} not a valid port number"))
            .ok()
    }
}

impl<D: PortDefaults> Port<D> {
    pub fn from_env() -> Option<Self> {
        Self::from_env_var(D::ENV_VAR?)
    }

    pub fn from_env_or_default() -> Self {
        Self::from_env().unwrap_or(Self::new(D::DEFAULT))
    }
}

impl<D> From<u16> for Port<D> {
    fn from(port: u16) -> Self {
        Self::new(port)
    }
}

impl<D> From<Port<D>> for u16 {
    fn from(value: Port<D>) -> Self {
        value.as_u16()
    }
}

impl<D> std::str::FromStr for Port<D> {
    type Err = <u16 as std::str::FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(Self::new)
    }
}

impl<D: PortDefaults> Default for Port<D> {
    fn default() -> Self {
        Self::from_env_or_default()
    }
}
