use anyhow::Context;
use axum::http::HeaderValue;
use std::{error::Error, net::SocketAddr, str::FromStr};

pub struct Env {
    pub allow_origin: HeaderValue,
    pub database_url: String,
    pub rest_socket: SocketAddr,
    pub secret: String,
}

#[cfg(not(debug_assertions))]
pub async fn init() -> anyhow::Result<Env> {
    Ok(Env {
        allow_origin: read("ALLOW_ORIGIN")?,
        database_url: read("DATABASE_URL")?,
        rest_socket: read("REST_SOCKET")?,
        secret: read("SECRET")?,
    })
}

#[cfg(not(debug_assertions))]
fn read<T, E>(key: &str) -> anyhow::Result<T>
where
    T: FromStr<Err = E>,
    E: Error + Send + Sync + 'static,
{
    std::env::var(key)
        .with_context(|| format!("Key missing: {key}"))?
        .parse()
        .with_context(|| format!("Key unparsable: {key}"))
}

#[cfg(debug_assertions)]
pub async fn init() -> anyhow::Result<Env> {
    let file = tokio::fs::read_to_string(".env")
        .await
        .context("No env file found")?;

    let mut map = file
        .split('\n')
        .filter_map(|s| s.split_once('='))
        .collect::<std::collections::HashMap<&str, &str>>();

    Ok(Env {
        allow_origin: read(&mut map, "ALLOW_ORIGIN")?,
        database_url: read(&mut map, "DATABASE_URL")?,
        rest_socket: read(&mut map, "REST_SOCKET")?,
        secret: read(&mut map, "SECRET")?,
    })
}

#[cfg(debug_assertions)]
fn read<T, E>(map: &mut std::collections::HashMap<&str, &str>, key: &str) -> anyhow::Result<T>
where
    T: FromStr<Err = E>,
    E: Error + Send + Sync + 'static,
{
    map.get(key)
        .with_context(|| format!("Key missing: {key}"))?
        .parse()
        .with_context(|| format!("Key unparsable: {key}"))
}
