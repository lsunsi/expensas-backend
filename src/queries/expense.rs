use super::{Label, Person, Split};
use sqlx::{postgres::types::PgMoney, Executor, Postgres};
use time::Date;

pub struct Expense {
    pub id: i32,
    pub creator: Person,
    pub payer: Person,
    pub split: Split,
    pub label: Label,
    pub detail: Option<String>,
    pub date: time::Date,
    pub paid: PgMoney,
    pub owed: PgMoney,
    pub confirmed_at: Option<time::OffsetDateTime>,
    pub refused_at: Option<time::OffsetDateTime>,
    pub created_at: time::OffsetDateTime,
}

pub async fn all(db: impl Executor<'_, Database = Postgres>) -> sqlx::Result<Vec<Expense>> {
    sqlx::query_as!(
        Expense,
        r#"
        SELECT
            id,
            creator as "creator: Person",
            payer as "payer: Person",
            split as "split: Split",
            label as "label: Label",
            detail,
            date,
            paid,
            owed,
            confirmed_at,
            refused_at,
            created_at
        FROM expenses
        ORDER BY date DESC, created_at DESC
        "#
    )
    .fetch_all(db)
    .await
}

pub async fn resolvable(
    db: impl Executor<'_, Database = Postgres>,
    id: i32,
    by: Person,
) -> sqlx::Result<bool> {
    sqlx::query_scalar!(
        r#"
        SELECT EXISTS (
            SELECT (refused_at IS NULL AND confirmed_at IS NULL)
            FROM expenses
            WHERE id = $1 AND creator != $2
        ) as "resolvable!"
        "#,
        id,
        by as Person
    )
    .fetch_one(db)
    .await
}

pub async fn submit(
    db: impl Executor<'_, Database = Postgres>,
    creator: Person,
    payer: Person,
    split: Split,
    label: Label,
    detail: Option<&str>,
    date: Date,
    paid: i64,
    owed: i64,
) -> sqlx::Result<i32> {
    sqlx::query_scalar!(
        "
        INSERT INTO expenses (creator, payer, split, label, detail, date, paid, owed, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW())
        RETURNING id
        ",
        creator as Person,
        payer as Person,
        split as Split,
        label as Label,
        detail,
        date,
        PgMoney(paid),
        PgMoney(owed)
    )
    .fetch_one(db)
    .await
}

pub async fn confirm(
    db: impl Executor<'_, Database = Postgres>,
    id: i32,
    by: Person,
) -> sqlx::Result<()> {
    sqlx::query_scalar!(
        "
        UPDATE expenses
        SET confirmed_at = NOW()
        WHERE id = $1
            AND confirmed_at IS NULL
            AND refused_at IS NULL
            AND creator != $2
        RETURNING id
        ",
        id,
        by as Person,
    )
    .fetch_one(db)
    .await
    .map(|_| ())
}

pub async fn refuse(
    db: impl Executor<'_, Database = Postgres>,
    id: i32,
    by: Person,
) -> sqlx::Result<()> {
    sqlx::query_scalar!(
        "
        UPDATE expenses
        SET refused_at = NOW()
        WHERE id = $1
            AND confirmed_at IS NULL
            AND refused_at IS NULL
            AND creator != $2
        RETURNING id
        ",
        id,
        by as Person
    )
    .fetch_one(db)
    .await
    .map(|_| ())
}

pub async fn total_owed(
    db: impl Executor<'_, Database = Postgres>,
    who: Person,
) -> sqlx::Result<i64> {
    sqlx::query!(
        "
        SELECT
            SUM(CASE payer WHEN $1 THEN owed ELSE 0::money END) as owed,
            SUM(CASE payer WHEN $1 THEN 0::money ELSE owed END) as owes
        FROM expenses
        WHERE confirmed_at IS NOT NULL
        ",
        who as Person
    )
    .fetch_one(db)
    .await
    .map(|r| {
        r.owed
            .zip(r.owes)
            .map(|(owed, owes)| owed.0 - owes.0)
            .unwrap_or_default()
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
        FROM expenses
        WHERE confirmed_at IS NULL
            AND refused_at IS NULL
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
