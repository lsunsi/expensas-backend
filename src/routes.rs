use crate::{
    auth::{Session, SessionAsk},
    queries::{Person, SessionState},
};
use axum::{extract::Path, http::StatusCode, Json};
use axum_extra::extract::PrivateCookieJar;

type Db = axum::Extension<sqlx::PgPool>;

pub async fn oiblz() -> &'static str {
    "oiblz"
}

pub async fn post_session_ask(
    db: Db,
    cookies: PrivateCookieJar,
    Path(who): Path<Person>,
) -> Result<PrivateCookieJar, StatusCode> {
    match crate::queries::create_session(&db, who).await {
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        Ok(id) => Ok(cookies.add(SessionAsk(id).into())),
    }
}

pub async fn post_session_cancel(cookies: PrivateCookieJar, ask: SessionAsk) -> PrivateCookieJar {
    cookies.remove(ask.into())
}

pub async fn get_session_state(
    db: Db,
    SessionAsk(id): SessionAsk,
) -> Result<Json<Option<SessionState>>, StatusCode> {
    match crate::queries::session_state(&db, id).await {
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        Ok(outcome) => Ok(Json(outcome.map(|o| o.1))),
    }
}

pub async fn post_session_confirm(db: Db, s: Session, Path(id): Path<i32>) -> StatusCode {
    match crate::queries::session_state(&db, id).await {
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
        Ok(Some((who, SessionState::Confirmable))) if who != s.who => {}
        _ => return StatusCode::BAD_REQUEST,
    };

    match crate::queries::confirm_session(&db, id).await {
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
        Ok(_) => StatusCode::OK,
    }
}

pub async fn post_session_convert(
    db: Db,
    cookies: PrivateCookieJar,
    ask @ SessionAsk(id): SessionAsk,
) -> Result<PrivateCookieJar, StatusCode> {
    let who = match crate::queries::session_state(&db, id).await {
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
        Ok(Some((who, SessionState::Convertable))) => who,
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    match crate::queries::convert_session(&db, id).await {
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        Ok(()) => Ok(cookies.remove(ask.into()).add(Session { who, id }.into())),
    }
}

pub async fn get_session_confirmable(db: Db, s: Session) -> Result<Json<Option<i32>>, StatusCode> {
    match crate::queries::confirmable_session(&db, s.who).await {
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        Ok(outcome) => Ok(Json(outcome)),
    }
}

pub async fn post_session_drop(cookies: PrivateCookieJar, s: Session) -> PrivateCookieJar {
    cookies.remove(s.into())
}
