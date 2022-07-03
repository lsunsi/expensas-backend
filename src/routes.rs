mod expense;
mod session;

use axum::{
    http::{Request, Response},
    routing::{get, post},
};
use std::time::Duration;
use tracing::Span;

type Db = axum::Extension<sqlx::PgPool>;

pub async fn init(db: sqlx::PgPool, env: crate::env::Env) -> anyhow::Result<()> {
    let router = axum::Router::new()
        .route("/", get(|| async { "oiblz" }))
        .route("/session/ask/:who", post(session::ask))
        .route("/session/cancel", post(session::cancel))
        .route("/session/state", get(session::state))
        .route("/session/confirm/:id", post(session::confirm))
        .route("/session/refuse/:id", post(session::refuse))
        .route("/session/convert", post(session::convert))
        .route("/session/confirmable", get(session::confirmable))
        .route("/session/drop", post(session::drop))
        .route("/expense/list", get(expense::list))
        .route("/expense/submit", post(expense::submit))
        .route("/expense/confirm/:id", post(expense::confirm))
        .route("/expense/refuse/:id", post(expense::refuse))
        .layer(axum::Extension(crate::auth::key(&env)))
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
