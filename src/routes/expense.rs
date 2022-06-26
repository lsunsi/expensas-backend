use super::Db;
use crate::{
    auth::Session,
    queries::{Person, Split},
};
use axum::{extract::Path, http::StatusCode, Json};
use futures::TryFutureExt;
use serde::{Deserialize, Serialize};
use std::ops::Deref;

#[derive(Serialize)]
pub struct ListResponse {
    creator: Person,
    payer: Person,
    split: Split,
    paid: i64,
    owed: i64,
    confirmed_at: Option<time::OffsetDateTime>,
    refused_at: Option<time::OffsetDateTime>,
    created_at: time::OffsetDateTime,
}

pub async fn list(db: Db, _s: Session) -> Result<Json<Vec<ListResponse>>, StatusCode> {
    match crate::queries::expense::all(db.deref()).await {
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        Ok(v) => Ok(Json(
            v.into_iter()
                .map(|i| ListResponse {
                    creator: i.creator,
                    payer: i.payer,
                    split: i.split,
                    paid: i.paid.0,
                    owed: i.owed.0,
                    confirmed_at: i.confirmed_at,
                    refused_at: i.refused_at,
                    created_at: i.created_at,
                })
                .collect(),
        )),
    }
}

#[derive(Deserialize)]
pub struct SubmitRequest {
    payer: Person,
    split: Split,
    paid: i64,
    owed: Option<i64>,
}

pub async fn submit(db: Db, s: Session, r: Json<SubmitRequest>) -> StatusCode {
    let owed = match (r.split, r.owed) {
        (Split::Arbitrary, Some(owed)) if owed <= r.paid => owed,
        (Split::Proportional, None) => match r.payer {
            Person::Ale => r.paid / 3,
            Person::Lu => r.paid * 2 / 3,
        },
        (Split::Evenly, None) => r.paid / 2,
        _ => return StatusCode::BAD_REQUEST,
    };

    match crate::queries::expense::submit(db.deref(), s.who, r.payer, r.split, r.paid, owed).await {
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
        Ok(_) => StatusCode::OK,
    }
}

pub async fn confirm(db: Db, s: Session, id: Path<i32>) -> StatusCode {
    let res = db.begin().and_then(|mut transaction| async move {
        if !crate::queries::expense::resolvable(&mut transaction, *id, s.who).await? {
            return Ok(None);
        };

        crate::queries::expense::confirm(&mut transaction, *id, s.who).await?;
        transaction.commit().map_ok(Some).await
    });

    match res.await {
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
        Ok(None) => StatusCode::BAD_REQUEST,
        Ok(Some(_)) => StatusCode::OK,
    }
}

pub async fn refuse(db: Db, s: Session, id: Path<i32>) -> StatusCode {
    let res = db.begin().and_then(|mut transaction| async move {
        if !crate::queries::expense::resolvable(&mut transaction, *id, s.who).await? {
            return Ok(None);
        };

        crate::queries::expense::refuse(&mut transaction, *id, s.who).await?;
        transaction.commit().map_ok(Some).await
    });

    match res.await {
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
        Ok(None) => StatusCode::BAD_REQUEST,
        Ok(Some(_)) => StatusCode::OK,
    }
}
