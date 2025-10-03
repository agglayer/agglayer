use std::{
    env,
    net::{Ipv4Addr, SocketAddrV4},
};

use tracing::warn;

/// Default port and IP address for given service.
pub trait AddrDefaults {
    /// The default IP address to listen on.
    const HOST: Ipv4Addr;

    /// The default port number for this service.
    const PORT: u16;

    /// Environment variable to load the port number from if it's not specified
    /// in the configuration file.
    const PORT_ENV_VAR: Option<&str>;
}

/// Port number parametrized by the defaults.
#[derive(educe::Educe, serde::Serialize, serde::Deserialize)]
#[educe(Debug, Eq, PartialEq, Ord, PartialOrd, Clone)]
#[serde(
    from = "AddrConfigOptions",
    into = "AddrConfigOptions",
    bound(deserialize = "D: AddrDefaults")
)]
pub struct AddrConfig<D> {
    addr: SocketAddrV4,
    _defaults: std::marker::PhantomData<fn() -> D>,
}

impl<D> AddrConfig<D> {
    pub const fn new(addr: SocketAddrV4) -> Self {
        let _defaults = std::marker::PhantomData;
        Self { addr, _defaults }
    }

    pub const fn port(&self) -> u16 {
        self.addr.port()
    }

    pub const fn ip(&self) -> &Ipv4Addr {
        self.addr.ip()
    }

    pub const fn addr(&self) -> &SocketAddrV4 {
        &self.addr
    }

    pub const fn into_addr(self) -> SocketAddrV4 {
        self.addr
    }

    pub fn to_addr(&self) -> SocketAddrV4 {
        self.addr.clone().into()
    }

    fn port_from_env_var(env_var: &str) -> Option<u16> {
        env::var(env_var)
            .inspect_err(|err| match err {
                env::VarError::NotPresent => (),
                env::VarError::NotUnicode(_) => warn!("Contents of ${env_var} are not Unicode"),
            })
            .ok()?
            .parse()
            .inspect_err(|_| warn!("Variable ${env_var} is not a valid port number"))
            .ok()
    }
}

impl<D: AddrDefaults> AddrConfig<D> {
    fn port_from_env() -> Option<u16> {
        Self::port_from_env_var(D::PORT_ENV_VAR?)
    }

    fn port_from_env_or_default() -> u16 {
        Self::port_from_env().unwrap_or(D::PORT)
    }

    fn from_optional_fields_with_defaults(host: Option<Ipv4Addr>, port: Option<u16>) -> Self {
        let ip = host.unwrap_or(D::HOST);
        let port = port.unwrap_or_else(Self::port_from_env_or_default);
        Self::new(SocketAddrV4::new(ip, port))
    }

    fn from_addr_config_options(config_opts: AddrConfigOptions) -> Self {
        match config_opts {
            AddrConfigOptions::Full(addr) => Self::new(addr),
            AddrConfigOptions::Parts { host, port } => {
                Self::from_optional_fields_with_defaults(host, port)
            }
        }
    }
}

impl<D> std::str::FromStr for AddrConfig<D> {
    type Err = <SocketAddrV4 as std::str::FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(Self::new)
    }
}

impl<D> From<SocketAddrV4> for AddrConfig<D> {
    fn from(addr: SocketAddrV4) -> Self {
        Self::new(addr)
    }
}

impl<D: AddrDefaults> Default for AddrConfig<D> {
    fn default() -> Self {
        Self::from_optional_fields_with_defaults(None, None)
    }
}

/// Defines how the config options look in the config file.
#[derive(educe::Educe, Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[educe(Default)]
#[serde(untagged)]
enum AddrConfigOptions {
    Full(SocketAddrV4),

    #[educe(Default)]
    Parts {
        #[serde(default)]
        host: Option<Ipv4Addr>,
        #[serde(default)]
        port: Option<u16>,
    },
}

impl<D> From<AddrConfig<D>> for AddrConfigOptions {
    fn from(addr_config: AddrConfig<D>) -> Self {
        Self::Full(addr_config.into_addr())
    }
}

impl<D: AddrDefaults> From<AddrConfigOptions> for AddrConfig<D> {
    fn from(config_opts: AddrConfigOptions) -> Self {
        AddrConfig::from_addr_config_options(config_opts)
    }
}
