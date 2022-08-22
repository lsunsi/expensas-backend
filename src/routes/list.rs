use super::Db;
use crate::{
    auth::Session,
    queries::{Label, Person, Split},
};
use axum::{http::StatusCode, Json};
use futures::TryFutureExt;
use itertools::Itertools;
use serde::Serialize;

#[derive(Serialize)]
struct Expense {
    id: i32,
    yours: bool,
    payer: Person,
    split: Split,
    label: Label,
    detail: Option<String>,
    date: String,
    paid: i64,
    spent: i64,
    confirmed: bool,
    refused: bool,
}

#[derive(Serialize)]
struct Transfer {
    id: i32,
    yours: bool,
    date: String,
    amount: i64,
    confirmed: bool,
    refused: bool,
}

#[derive(Serialize)]
#[serde(tag = "t", content = "c")]
enum Item {
    Transfer(Transfer),
    Expense(Expense),
}

#[derive(Serialize)]
struct Month {
    n: u8,
    spent_me: u64,
    spent_we: u64,
    items: Vec<Item>,
}

#[derive(Serialize)]
pub struct Response {
    pendings: Vec<Item>,
    months: Vec<Month>,
}

pub async fn get(db: Db, s: Session) -> Result<Json<Response>, StatusCode> {
    let (expenses, transfers) = db
        .begin()
        .and_then(|mut transaction| async move {
            Ok((
                crate::queries::expense::all(&mut transaction).await?,
                crate::queries::transfer::all(&mut transaction).await?,
            ))
        })
        .await
        .map_err(|e| {
            tracing::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let expenses = expenses.into_iter().map(|e| {
        let spent = if e.payer == s.who {
            e.paid.0 - e.owed.0
        } else {
            e.owed.0
        };

        (
            e.date,
            e.created_at,
            spent,
            e.paid.0,
            e.confirmed_at.is_none() && e.refused_at.is_none(),
            e.confirmed_at.is_some(),
            Item::Expense(Expense {
                id: e.id,
                yours: e.creator == s.who,
                payer: e.payer,
                split: e.split,
                label: e.label,
                detail: e.detail,
                date: date_to_string(e.date),
                paid: e.paid.0,
                spent,
                confirmed: e.confirmed_at.is_some(),
                refused: e.refused_at.is_some(),
            }),
        )
    });

    let transfers = transfers.into_iter().map(|t| {
        (
            t.date,
            t.created_at,
            0,
            0,
            t.confirmed_at.is_none() && t.refused_at.is_none(),
            t.confirmed_at.is_some(),
            Item::Transfer(Transfer {
                id: t.id,
                yours: t.sender == s.who,
                date: date_to_string(t.date),
                amount: t.amount.0,
                confirmed: t.confirmed_at.is_some(),
                refused: t.refused_at.is_some(),
            }),
        )
    });

    let groups = expenses
        .chain(transfers)
        .sorted_by_key(|a| (a.0, a.1))
        .rev()
        .group_by(|a| (a.0.month()));

    let mut pendings = Vec::new();
    let mut months = Vec::new();

    for (month, group) in &groups {
        let mut items = Vec::new();
        let mut spent_me = 0;
        let mut spent_we = 0;

        for (_, _, spent, paid, pending, confirmed, item) in group {
            if pending {
                pendings.push(item);
                continue;
            }

            items.push(item);

            if confirmed {
                spent_me += spent as u64;
                spent_we += paid as u64;
            }
        }

        months.push(Month {
            n: month as u8,
            spent_me,
            spent_we,
            items,
        })
    }

    Ok(Json(Response { pendings, months }))
}

fn date_to_string(date: time::Date) -> String {
    format!(
        "{:0>4}-{:0>2}-{:0>2}",
        date.year(),
        date.month() as u8,
        date.day()
    )
}
