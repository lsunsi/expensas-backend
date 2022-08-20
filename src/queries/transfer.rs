use super::Person;
use sqlx::{postgres::types::PgMoney, Executor, Postgres};
use time::Date;

pub struct Transfer {
    pub id: i32,
    pub sender: Person,
    pub receiver: Person,
    pub date: time::Date,
    pub amount: PgMoney,
    pub confirmed_at: Option<time::OffsetDateTime>,
    pub refused_at: Option<time::OffsetDateTime>,
    pub created_at: time::OffsetDateTime,
}

pub async fn all(db: impl Executor<'_, Database = Postgres>) -> sqlx::Result<Vec<Transfer>> {
    sqlx::query_as!(
        Transfer,
        r#"
        SELECT
            id,
            sender as "sender: Person",
            receiver as "receiver: Person",
            date,
            amount,
            confirmed_at,
            refused_at,
            created_at
        FROM transfers
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
            FROM transfers
            WHERE id = $1 AND receiver = $2
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
    sender: Person,
    receiver: Person,
    date: Date,
    amount: i64,
) -> sqlx::Result<i32> {
    sqlx::query_scalar!(
        "
        INSERT INTO transfers (sender, receiver, date, amount, created_at)
        VALUES ($1, $2, $3, $4, NOW())
        RETURNING id
        ",
        sender as Person,
        receiver as Person,
        date,
        PgMoney(amount)
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
        UPDATE transfers
        SET confirmed_at = NOW()
        WHERE id = $1
            AND confirmed_at IS NULL
            AND refused_at IS NULL
            AND receiver = $2
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
        UPDATE transfers
        SET refused_at = NOW()
        WHERE id = $1
            AND confirmed_at IS NULL
            AND refused_at IS NULL
            AND receiver = $2
        RETURNING id
        ",
        id,
        by as Person
    )
    .fetch_one(db)
    .await
    .map(|_| ())
}
