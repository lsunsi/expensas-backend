use super::Person;
use sqlx::{Executor, Postgres};

pub struct TotalOwed {
    pub definitely: i64,
    pub maybe: i64,
}

pub async fn total_owed(
    db: impl Executor<'_, Database = Postgres>,
    by: Person,
) -> sqlx::Result<TotalOwed> {
    sqlx::query!(
        "
        SELECT
            SUM(CASE WHEN payer = $1 AND confirmed THEN owed ELSE 0::money END) as owed_def,
            SUM(CASE WHEN payer = $1 AND NOT confirmed THEN owed ELSE 0::money END) as owed_maybe,
            SUM(CASE WHEN payer != $1 AND confirmed THEN owed ELSE 0::money END) as owes_def,
            SUM(CASE WHEN payer != $1 AND NOT confirmed THEN owed ELSE 0::money END) as owes_maybe
        FROM (
	        SELECT owed, payer, confirmed_at IS NOT NULL as confirmed
	        FROM expenses
	        WHERE refused_at IS NULL
	        UNION ALL
	        SELECT amount, sender, confirmed_at IS NOT NULL as confirmed
	        FROM transfers
	        WHERE refused_at IS NULL
	    ) _
        ",
        by as Person
    )
    .fetch_one(db)
    .await
    .map(|r| TotalOwed {
        definitely: r.owed_def.map(|a| a.0).unwrap_or(0) - r.owes_def.map(|a| a.0).unwrap_or(0),
        maybe: r.owed_maybe.map(|a| a.0).unwrap_or(0) - r.owes_maybe.map(|a| a.0).unwrap_or(0),
    })
}

pub struct ResolvableCount {
    pub by_other: i64,
    pub by_you: i64,
}

pub async fn resolvable_count(
    db: impl Executor<'_, Database = Postgres>,
    me: Person,
) -> sqlx::Result<ResolvableCount> {
    sqlx::query!(
        "
        SELECT
            SUM(CASE creator WHEN $1 THEN 1 ELSE 0 END) as by_other,
            SUM(CASE creator WHEN $1 THEN 0 ELSE 1 END) as by_you
        FROM (
        	SELECT creator FROM expenses WHERE confirmed_at IS NULL AND refused_at IS NULL
        	UNION ALL
        	SELECT sender FROM transfers WHERE confirmed_at IS NULL AND refused_at IS NULL
        ) _
        ",
        me as Person
    )
    .fetch_one(db)
    .await
    .map(|r| ResolvableCount {
        by_other: r.by_other.unwrap_or_default(),
        by_you: r.by_you.unwrap_or_default(),
    })
}
