mod expense;
mod list;
mod session;
mod summary;
mod transfer;

use axum::{
    extract::FromRef,
    http::{Method, Request, Response},
    routing::{get, post},
};
use std::time::Duration;
use tracing::Span;

type Db = axum::extract::State<sqlx::PgPool>;

#[derive(Clone, FromRef)]
pub(crate) struct State {
    key: axum_extra::extract::cookie::Key,
    db: sqlx::PgPool,
}

pub async fn init(db: sqlx::PgPool, env: crate::env::Env) -> anyhow::Result<()> {
    let key = crate::auth::key(&env);

    let cors = tower_http::cors::CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(env.allow_origin.clone())
        .allow_credentials(true);

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
        .route("/expense/submit", post(expense::submit))
        .route("/expense/confirm/:id", post(expense::confirm))
        .route("/expense/refuse/:id", post(expense::refuse))
        .route("/expense/splitrecc/:p/:l", get(expense::splitrecc))
        .route("/transfer/submit", post(transfer::submit))
        .route("/transfer/confirm/:id", post(transfer::confirm))
        .route("/transfer/refuse/:id", post(transfer::refuse))
        .route("/summary", get(summary::get))
        .route("/list", post(list::generate))
        .layer(cors)
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
        )
        .with_state(State { db, key });

    axum::Server::bind(&env.rest_socket)
        .serve(router.into_make_service())
        .await?;

    Ok(())
}
