use axum::routing::get;
use sqlx::postgres::PgPoolOptions;

mod env;
mod queries;
mod routes;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let env = env::init().await?;

    let pg = PgPoolOptions::new().connect(&env.database_url).await?;

    let router = axum::Router::new().route("/", get(routes::oiblz));

    axum::Server::bind(&env.rest_socket)
        .serve(router.into_make_service())
        .await?;

    Ok(())
}
