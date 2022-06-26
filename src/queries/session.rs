use super::Person;
use sqlx::PgPool;

pub async fn ask(db: &PgPool, person: Person) -> sqlx::Result<i32> {
    sqlx::query_scalar!(
        "
		INSERT INTO sessions (who, created_at, confirmed_at)
		VALUES ($1, NOW(), (SELECT CASE EXISTS (SELECT 1 FROM sessions) WHEN true THEN null ELSE NOW() END))
		RETURNING id
		",
        person as Person
    )
    .fetch_one(db)
    .await
}

pub async fn confirm(db: &PgPool, id: i32) -> sqlx::Result<()> {
    sqlx::query_scalar!(
        "
		UPDATE sessions
		SET confirmed_at = NOW()
		WHERE id = $1
            AND confirmed_at IS NULL
            AND refused_at IS NULL
		RETURNING id
		",
        id
    )
    .fetch_one(db)
    .await
    .map(|_| ())
}

pub async fn refuse(db: &PgPool, id: i32) -> sqlx::Result<()> {
    sqlx::query_scalar!(
        "
        UPDATE sessions
        SET refused_at = NOW()
        WHERE id = $1
            AND confirmed_at IS NULL
            AND refused_at IS NULL
        RETURNING id
        ",
        id
    )
    .fetch_one(db)
    .await
    .map(|_| ())
}

pub async fn convert(db: &PgPool, id: i32) -> sqlx::Result<()> {
    sqlx::query_scalar!(
        "
		UPDATE sessions
		SET converted_at = NOW()
		WHERE id = $1
            AND confirmed_at IS NOT NULL
            AND converted_at IS NULL
		RETURNING id
		",
        id
    )
    .fetch_one(db)
    .await
    .map(|_| ())
}

#[derive(serde::Serialize)]
pub enum SessionState {
    Confirmable,
    Convertable,
    Converted,
    Refused,
    Stale,
}

pub async fn state(db: &PgPool, id: i32) -> sqlx::Result<Option<(Person, SessionState)>> {
    sqlx::query!(
        r#"
		SELECT
            s1.who as "who: Person",
            s1.confirmed_at,
            s1.converted_at,
            s1.refused_at,
            s2.id IS NOT NULL as "stale!"
		FROM sessions s1
        LEFT JOIN sessions s2 ON s2.who = s1.who and s2.created_at > s1.created_at
		WHERE s1.id = $1
		"#,
        id
    )
    .fetch_optional(db)
    .await
    .map(|r| {
        r.map(|r| {
            (
                r.who,
                if r.refused_at.is_some() {
                    SessionState::Refused
                } else if r.stale {
                    SessionState::Stale
                } else if r.confirmed_at.is_none() {
                    SessionState::Confirmable
                } else if r.converted_at.is_none() {
                    SessionState::Convertable
                } else {
                    SessionState::Converted
                },
            )
        })
    })
}

pub async fn confirmable(db: &PgPool, by: Person) -> sqlx::Result<Option<i32>> {
    sqlx::query_scalar!(
        r#"
        SELECT s1.id
        FROM sessions s1
        LEFT JOIN sessions s2 ON s2.who = s1.who and s2.created_at > s1.created_at
        WHERE s1.who != $1
            AND s1.confirmed_at IS NULL
            AND s1.refused_at IS NULL
            AND s2.id IS NULL
        LIMIT 1
        "#,
        by as Person
    )
    .fetch_optional(db)
    .await
}
