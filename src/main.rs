use axum::{
    http::{Request, Response},
    routing::{get, post},
};
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
use tracing::Span;

mod auth;
mod env;
mod queries;
mod routes;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let env = env::init().await?;
    let key = auth::key(&env);

    tracing_subscriber::fmt::init();

    let db = PgPoolOptions::new().connect(&env.database_url).await?;
    sqlx::migrate!().run(&db).await?;

    queries::create_session(&db, queries::Person::Ale).await?;

    let router = axum::Router::new()
        .route("/", get(routes::oiblz))
        .route("/session/ask/:who", post(routes::post_session_ask))
        .route("/session/cancel", post(routes::post_session_cancel))
        .route("/session/state", get(routes::get_session_state))
        .route("/session/confirm/:id", post(routes::post_session_confirm))
        .route("/session/refuse/:id", post(routes::post_session_refuse))
        .route("/session/convert", post(routes::post_session_convert))
        .route("/session/confirmable", get(routes::get_session_confirmable))
        .route("/session/drop", post(routes::post_session_drop))
        .layer(axum::Extension(key))
        .layer(axum::Extension(db))
        .layer(
            tower_http::trace::TraceLayer::new_for_http()
                .make_span_with(|req: &Request<_>| {
                    tracing::info_span!(
                        "request",
                        http.method = %req.method(),
                        http.target = %req.uri(),
                        http.status_code = tracing::field::Empty,
                        latency = tracing::field::Empty,
                    )
                })
                .on_response(|resp: &Response<_>, latency: Duration, span: &Span| {
                    span.record("http.status_code", &tracing::field::display(resp.status()));
                    span.record("latency", &tracing::field::debug(latency));
                    tracing::info!("!")
                }),
        );

    axum::Server::bind(&env.rest_socket)
        .serve(router.into_make_service())
        .await?;

    Ok(())
}
