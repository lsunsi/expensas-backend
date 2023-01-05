use super::Db;
use crate::{
    auth::Session,
    queries::{Label, Person, Split},
};
use axum::{extract::Path, http::StatusCode, Json};
use futures::TryFutureExt;
use serde::Deserialize;
use std::ops::Deref;
use time::format_description::well_known::Iso8601;

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
        (Split::Proportional2to1, None) => match r.payer {
            Person::Ale => r.paid / 3,
            Person::Lu => r.paid * 2 / 3,
        },
        (Split::Proportional3to2, None) => match r.payer {
            Person::Ale => r.paid * 2 / 5,
            Person::Lu => r.paid * 3 / 5,
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
        Ok(_) => StatusCode::OK,
        Err(e) => {
            tracing::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
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
        if !crate::queries::expense::resolvable(&mut transaction, *id, s.who).await? {
            return Ok(None);
        };

        crate::queries::expense::refuse(&mut transaction, *id, s.who).await?;
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

pub async fn splitrecc(
    db: Db,
    _s: Session,
    Path((payer, label)): Path<(Person, Label)>,
) -> Result<Json<Option<Split>>, StatusCode> {
    match crate::queries::expense::splitrecc(db.deref(), payer, label).await {
        Ok(sr) => Ok(Json(sr)),
        Err(e) => {
            tracing::error!("{e:?}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
