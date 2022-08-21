use super::Db;
use crate::{auth::Session, queries::Person};
use axum::{extract::Path, http::StatusCode, Json};
use futures::TryFutureExt;
use serde::Deserialize;
use std::ops::Deref;
use time::format_description::well_known::Iso8601;

#[derive(Deserialize)]
pub struct SubmitRequest {
    date: String,
    amount: i64,
}

pub async fn submit(db: Db, s: Session, r: Json<SubmitRequest>) -> StatusCode {
    let date = match time::Date::parse(&r.date, &Iso8601::DEFAULT) {
        Err(_) => return StatusCode::BAD_REQUEST,
        Ok(data) => data,
    };

    let receiver = match s.who {
        Person::Ale => Person::Lu,
        Person::Lu => Person::Ale,
    };

    match crate::queries::transfer::submit(db.deref(), s.who, receiver, date, r.amount).await {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            tracing::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub async fn confirm(db: Db, s: Session, id: Path<i32>) -> StatusCode {
    let res = db.begin().and_then(|mut transaction| async move {
        if !crate::queries::transfer::resolvable(&mut transaction, *id, s.who).await? {
            return Ok(None);
        };

        crate::queries::transfer::confirm(&mut transaction, *id, s.who).await?;
        transaction.commit().map_ok(Some).await
    });

    match res.await {
        Ok(Some(_)) => StatusCode::OK,
        Ok(None) => StatusCode::BAD_REQUEST,
        Err(e) => {
            tracing::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub async fn refuse(db: Db, s: Session, id: Path<i32>) -> StatusCode {
    let res = db.begin().and_then(|mut transaction| async move {
        if !crate::queries::transfer::resolvable(&mut transaction, *id, s.who).await? {
            return Ok(None);
        };

        crate::queries::transfer::refuse(&mut transaction, *id, s.who).await?;
        transaction.commit().map_ok(Some).await
    });

    match res.await {
        Ok(Some(_)) => StatusCode::OK,
        Ok(None) => StatusCode::BAD_REQUEST,
        Err(e) => {
            tracing::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
