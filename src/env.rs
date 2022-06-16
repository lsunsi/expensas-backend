use anyhow::Context;
use std::{collections::HashMap, error::Error, net::SocketAddr, str::FromStr};

pub struct Env {
    pub rest_socket: SocketAddr,
    pub database_url: String,
    pub secret: String,
}

pub async fn init() -> anyhow::Result<Env> {
    let file = tokio::fs::read_to_string(ENV_PATH)
        .await
        .context("No env file found")?;

    let mut map = file
        .split('\n')
        .filter_map(|s| s.split_once('='))
        .collect::<HashMap<&str, &str>>();

    Ok(Env {
        rest_socket: read(&mut map, "REST_SOCKET")?,
        database_url: read(&mut map, "DATABASE_URL")?,
        secret: read(&mut map, "SECRET")?,
    })
}

fn read<T, E>(map: &mut HashMap<&str, &str>, key: &str) -> anyhow::Result<T>
where
    T: FromStr<Err = E>,
    E: Error + Send + Sync + 'static,
{
    map.get(key)
        .with_context(|| format!("Key missing: {key}"))?
        .parse()
        .with_context(|| format!("Key unparsable: {key}"))
}

const ENV_PATH: &str = ".env";
