mod auth;
mod env;
mod queries;
mod routes;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let env = env::init().await?;
    let db = queries::init(&env).await?;

    routes::init(db, env).await?;
    Ok(())
}
