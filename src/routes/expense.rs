use super::Db;
use crate::{
    auth::Session,
    queries::{Person, Split},
};
use axum::{extract::Path, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

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
    match crate::queries::expense::all(&db).await {
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

    match crate::queries::expense::submit(&db, s.who, r.payer, r.split, r.paid, owed).await {
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
        Ok(_) => StatusCode::OK,
    }
}

pub async fn confirm(db: Db, s: Session, id: Path<i32>) -> StatusCode {
    match crate::queries::expense::resolvable(&db, *id, s.who).await {
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
        Ok(false) => return StatusCode::BAD_REQUEST,
        Ok(true) => {}
    };

    match crate::queries::expense::confirm(&db, *id, s.who).await {
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
        Ok(_) => StatusCode::OK,
    }
}

pub async fn refuse(db: Db, s: Session, id: Path<i32>) -> StatusCode {
    match crate::queries::expense::resolvable(&db, *id, s.who).await {
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
        Ok(false) => return StatusCode::BAD_REQUEST,
        Ok(true) => {}
    };

    match crate::queries::expense::refuse(&db, *id, s.who).await {
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
        Ok(_) => StatusCode::OK,
    }
}
