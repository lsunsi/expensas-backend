use super::Db;
use crate::{
    auth::{Session, SessionAsk},
    queries::{session::SessionState, Person},
};
use axum::{extract::Path, http::StatusCode, Json};
use axum_extra::extract::PrivateCookieJar;

pub async fn ask(
    db: Db,
    cookies: PrivateCookieJar,
    Path(who): Path<Person>,
) -> Result<PrivateCookieJar, StatusCode> {
    match crate::queries::session::ask(&db, who).await {
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        Ok(id) => Ok(cookies.add(SessionAsk(id).into())),
    }
}

pub async fn cancel(cookies: PrivateCookieJar, ask: SessionAsk) -> PrivateCookieJar {
    cookies.remove(ask.into())
}

pub async fn state(
    db: Db,
    SessionAsk(id): SessionAsk,
) -> Result<Json<Option<SessionState>>, StatusCode> {
    match crate::queries::session::state(&db, id).await {
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        Ok(outcome) => Ok(Json(outcome.map(|o| o.1))),
    }
}

pub async fn confirm(db: Db, s: Session, Path(id): Path<i32>) -> StatusCode {
    match crate::queries::session::state(&db, id).await {
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
        Ok(Some((who, SessionState::Confirmable))) if who != s.who => {}
        _ => return StatusCode::BAD_REQUEST,
    };

    match crate::queries::session::confirm(&db, id).await {
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
        Ok(_) => StatusCode::OK,
    }
}

pub async fn refuse(db: Db, s: Session, Path(id): Path<i32>) -> StatusCode {
    match crate::queries::session::state(&db, id).await {
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
        Ok(Some((who, SessionState::Confirmable))) if who != s.who => {}
        _ => return StatusCode::BAD_REQUEST,
    };

    match crate::queries::session::refuse(&db, id).await {
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
        Ok(_) => StatusCode::OK,
    }
}

pub async fn convert(
    db: Db,
    cookies: PrivateCookieJar,
    ask @ SessionAsk(id): SessionAsk,
) -> Result<PrivateCookieJar, StatusCode> {
    let who = match crate::queries::session::state(&db, id).await {
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
        Ok(Some((who, SessionState::Convertable))) => who,
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    match crate::queries::session::convert(&db, id).await {
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        Ok(()) => Ok(cookies.remove(ask.into()).add(Session { who, id }.into())),
    }
}

pub async fn confirmable(db: Db, s: Session) -> Result<Json<Option<i32>>, StatusCode> {
    match crate::queries::session::confirmable(&db, s.who).await {
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        Ok(outcome) => Ok(Json(outcome)),
    }
}

pub async fn drop(cookies: PrivateCookieJar, s: Session) -> PrivateCookieJar {
    cookies.remove(s.into())
}
