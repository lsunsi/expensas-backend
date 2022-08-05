use super::Db;
use crate::{
    auth::Session,
    queries::{Label, Person, Split},
};
use axum::{extract::Path, http::StatusCode, Json};
use futures::TryFutureExt;
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use time::format_description::well_known::{Iso8601, Rfc3339};

#[derive(Serialize)]
pub struct ListResponse {
    id: i32,
    creator: Person,
    payer: Person,
    split: Split,
    label: Label,
    detail: Option<String>,
    date: String,
    paid: i64,
    owed: i64,
    confirmed_at: Option<String>,
    refused_at: Option<String>,
    created_at: String,
}

pub async fn list(db: Db, _s: Session) -> Result<Json<Vec<ListResponse>>, StatusCode> {
    match crate::queries::expense::all(db.deref()).await {
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        Ok(v) => {
            let mut list = vec![];

            for i in v {
                let date = format!(
                    "{:0>4}-{:0>2}-{:0>2}",
                    i.date.year(),
                    i.date.month() as u8,
                    i.date.day()
                );

                let confirmed_at = i
                    .confirmed_at
                    .map(|t| t.format(&Rfc3339))
                    .transpose()
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

                let refused_at = i
                    .refused_at
                    .map(|t| t.format(&Rfc3339))
                    .transpose()
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

                let created_at = i
                    .created_at
                    .format(&Rfc3339)
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

                list.push(ListResponse {
                    id: i.id,
                    creator: i.creator,
                    payer: i.payer,
                    split: i.split,
                    label: i.label,
                    detail: i.detail,
                    paid: i.paid.0,
                    owed: i.owed.0,
                    confirmed_at,
                    refused_at,
                    created_at,
                    date,
                });
            }

            Ok(Json(list))
        }
    }
}

#[derive(Deserialize)]
pub struct SubmitRequest {
    payer: Person,
    split: Split,
    label: Label,
    detail: Option<String>,
    date: String,
    paid: i64,
    owed: Option<i64>,
}

pub async fn submit(db: Db, s: Session, r: Json<SubmitRequest>) -> StatusCode {
    let date = match time::Date::parse(&r.date, &Iso8601::DEFAULT) {
        Err(_) => return StatusCode::BAD_REQUEST,
        Ok(data) => data,
    };

    let owed = match (r.split, r.owed) {
        (Split::Arbitrary, Some(owed)) if owed <= r.paid => owed,
        (Split::Proportional, None) => match r.payer {
            Person::Ale => r.paid / 3,
            Person::Lu => r.paid * 2 / 3,
        },
        (Split::Evenly, None) => r.paid / 2,
        _ => return StatusCode::BAD_REQUEST,
    };

    match crate::queries::expense::submit(
        db.deref(),
        s.who,
        r.payer,
        r.split,
        r.label,
        r.detail.as_deref(),
        date,
        r.paid,
        owed,
    )
    .await
    {
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
