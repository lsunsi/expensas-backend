use super::Db;
use crate::{auth::Session, queries::Person};
use axum::{http::StatusCode, Json};
use futures::TryFutureExt;
use serde::Serialize;

#[derive(Serialize)]
pub struct GetResponse {
    me: Person,
    owed: i64,
    pending_you: i64,
    pending_other: i64,
}

pub async fn get(db: Db, s: Session) -> Result<Json<GetResponse>, StatusCode> {
    let (owed, resolvable) = db
        .begin()
        .and_then(|mut tr| async move {
            let owed = crate::queries::expense::total_owed(&mut tr, s.who).await?;
            let resolvable = crate::queries::expense::resolvable_count(&mut tr, s.who).await?;
            Ok((owed, resolvable))
        })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(GetResponse {
        me: s.who,
        owed,
        pending_you: resolvable.by_you,
        pending_other: resolvable.by_other,
    }))
}
