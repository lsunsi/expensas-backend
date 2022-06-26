use super::{Person, Split};
use sqlx::{postgres::types::PgMoney, PgPool};

pub struct Expense {
    pub creator: Person,
    pub payer: Person,
    pub split: Split,
    pub paid: PgMoney,
    pub owed: PgMoney,
    pub confirmed_at: Option<time::OffsetDateTime>,
    pub refused_at: Option<time::OffsetDateTime>,
    pub created_at: time::OffsetDateTime,
}

pub async fn all(db: &PgPool) -> sqlx::Result<Vec<Expense>> {
    sqlx::query_as!(
        Expense,
        r#"
        SELECT
            creator as "creator: Person",
            payer as "payer: Person",
            split as "split: Split",
            paid,
            owed,
            confirmed_at,
            refused_at,
            created_at
        FROM expenses
        "#
    )
    .fetch_all(db)
    .await
}

pub async fn resolvable(db: &PgPool, id: i32, by: Person) -> sqlx::Result<bool> {
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
    db: &PgPool,
    creator: Person,
    payer: Person,
    split: Split,
    paid: i64,
    owed: i64,
) -> sqlx::Result<i32> {
    sqlx::query_scalar!(
        "
        INSERT INTO expenses (creator, payer, split, paid, owed, created_at)
        VALUES ($1, $2, $3, $4, $5, NOW())
        RETURNING id
        ",
        creator as Person,
        payer as Person,
        split as Split,
        PgMoney(paid),
        PgMoney(owed)
    )
    .fetch_one(db)
    .await
}

pub async fn confirm(db: &PgPool, id: i32, by: Person) -> sqlx::Result<()> {
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

pub async fn refuse(db: &PgPool, id: i32, by: Person) -> sqlx::Result<()> {
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