use {
    serde::{
        de::{self, Deserializer},
        Deserialize,
    },
    std::net::{IpAddr, Ipv4Addr, SocketAddr},
};

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct ConfigGrpc2Redis {
    pub tracing: ConfigTracing,
    pub grpc: ConfigGrpc,
    pub redis: ConfigRedis,
    pub listen_admin: ConfigListenAdmin,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct ConfigTracing {
    pub json: bool,
}

impl Default for ConfigTracing {
    fn default() -> Self {
        Self { json: true }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct ConfigGrpc {
    pub endpoint: String,
    pub x_token: Option<String>,
}

impl Default for ConfigGrpc {
    fn default() -> Self {
        Self {
            endpoint: "http://127.0.0.1:10000".to_owned(),
            x_token: None,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct ConfigRedis {
    pub endpoint: String,
    pub stream_key: String,
    pub stream_maxlen: u64,
    pub stream_field_key: String,
}

impl Default for ConfigRedis {
    fn default() -> Self {
        Self {
            endpoint: "redis://127.0.0.1:6379/".to_owned(),
            stream_key: "solfees:events".to_owned(),
            stream_maxlen: 15 * 60 * 3 * 4, // ~15min (2.5 slots per sec, 4 events per slot)
            stream_field_key: "message".to_owned(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct ConfigListenAdmin {
    #[serde(deserialize_with = "deserialize_listen")]
    pub bind: SocketAddr,
}

impl Default for ConfigListenAdmin {
    fn default() -> Self {
        Self {
            bind: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8000),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigServer {
    #[serde(default)]
    pub tracing: ConfigTracing,
}

fn deserialize_listen<'de, D>(deserializer: D) -> Result<SocketAddr, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Debug, PartialEq, Eq, Hash, Deserialize)]
    #[serde(untagged)]
    enum Value {
        SocketAddr(SocketAddr),
        Port(u16),
        Env { env: String },
    }

    match Value::deserialize(deserializer)? {
        Value::SocketAddr(addr) => Ok(addr),
        Value::Port(port) => Ok(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), port)),
        Value::Env { env } => std::env::var(env)
            .map_err(|error| format!("{:}", error))
            .and_then(|value| match value.parse() {
                Ok(addr) => Ok(addr),
                Err(error) => match value.parse() {
                    Ok(port) => Ok(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), port)),
                    Err(_) => Err(format!("{:?}", error)),
                },
            })
            .map_err(de::Error::custom),
    }
}
