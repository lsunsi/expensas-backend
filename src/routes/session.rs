use super::Db;
use crate::{
    auth::{Session, SessionAsk},
    queries::{session::SessionState, Person},
};
use axum::{extract::Path, http::StatusCode, Json};
use axum_extra::extract::PrivateCookieJar;
use futures::TryFutureExt;
use std::ops::Deref;

pub async fn ask(
    db: Db,
    cookies: PrivateCookieJar,
    Path(who): Path<Person>,
) -> Result<PrivateCookieJar, StatusCode> {
    match crate::queries::session::ask(db.deref(), who).await {
        Ok(id) => Ok(cookies.add(SessionAsk(id).into())),
        Err(e) => {
            tracing::error!("{e:?}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn cancel(cookies: PrivateCookieJar, ask: SessionAsk) -> PrivateCookieJar {
    cookies.remove(ask.into())
}

pub async fn state(
    db: Db,
    SessionAsk(id): SessionAsk,
) -> Result<Json<Option<SessionState>>, StatusCode> {
    match crate::queries::session::state(db.deref(), id).await {
        Ok(outcome) => Ok(Json(outcome.map(|o| o.1))),
        Err(e) => {
            tracing::error!("{e:?}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn confirm(db: Db, s: Session, Path(id): Path<i32>) -> StatusCode {
    let res = db.begin().and_then(|mut transaction| async move {
        match crate::queries::session::state(&mut transaction, id).await? {
            Some((who, SessionState::Confirmable)) if who != s.who => {}
            _ => return Ok(None),
        };

        crate::queries::session::confirm(&mut transaction, id).await?;
        transaction.commit().await.map(Some)
    });

    match res.await {
        Ok(Some(())) => StatusCode::OK,
        Ok(None) => StatusCode::BAD_REQUEST,
        Err(e) => {
            tracing::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub async fn refuse(db: Db, s: Session, Path(id): Path<i32>) -> StatusCode {
    let res = db.begin().and_then(|mut transaction| async move {
        match crate::queries::session::state(&mut transaction, id).await? {
            Some((who, SessionState::Confirmable)) if who != s.who => {}
            _ => return Ok(None),
        };

        crate::queries::session::refuse(&mut transaction, id).await?;
        transaction.commit().await.map(Some)
    });

    match res.await {
        Ok(Some(())) => StatusCode::OK,
        Ok(None) => StatusCode::BAD_REQUEST,
        Err(e) => {
            tracing::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub async fn convert(
    db: Db,
    cookies: PrivateCookieJar,
    ask @ SessionAsk(id): SessionAsk,
) -> Result<PrivateCookieJar, StatusCode> {
    let res = db.begin().and_then(|mut transaction| async move {
        let who = match crate::queries::session::state(&mut transaction, id).await? {
            Some((who, SessionState::Convertable)) => who,
            _ => return Ok(None),
        };

        crate::queries::session::convert(&mut transaction, id).await?;
        transaction.commit().await.map(|()| Some(who))
    });

    match res.await {
        Ok(Some(who)) => Ok(cookies.remove(ask.into()).add(Session { who, id }.into())),
        Ok(None) => Err(StatusCode::BAD_REQUEST),
        Err(e) => {
            tracing::error!("{e:?}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn confirmable(db: Db, s: Session) -> Result<Json<Option<i32>>, StatusCode> {
    match crate::queries::session::confirmable(db.deref(), s.who).await {
        Ok(outcome) => Ok(Json(outcome)),
        Err(e) => {
            tracing::error!("{e:?}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn drop(cookies: PrivateCookieJar, s: Session) -> PrivateCookieJar {
    cookies.remove(s.into())
}
