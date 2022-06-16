use sqlx::PgPool;

#[derive(Debug, PartialEq, sqlx::Type, serde::Deserialize)]
#[sqlx(type_name = "person")]
pub enum Person {
    Ale,
    Lu,
}

impl std::ops::Not for Person {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Person::Ale => Person::Lu,
            Person::Lu => Person::Ale,
        }
    }
}

pub async fn create_session(db: &PgPool, person: Person) -> sqlx::Result<i32> {
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

pub async fn confirm_session(db: &PgPool, id: i32) -> sqlx::Result<()> {
    sqlx::query_scalar!(
        "
		UPDATE sessions
		SET confirmed_at = NOW()
		WHERE id = $1 AND confirmed_at IS NULL
		RETURNING id
		",
        id
    )
    .fetch_one(db)
    .await
    .map(|_| ())
}

pub async fn convert_session(db: &PgPool, id: i32) -> sqlx::Result<()> {
    sqlx::query_scalar!(
        "
		UPDATE sessions
		SET converted_at = NOW()
		WHERE id = $1 AND confirmed_at IS NOT NULL AND converted_at IS NULL
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
    Stale,
}

pub async fn session_state(db: &PgPool, id: i32) -> sqlx::Result<Option<(Person, SessionState)>> {
    sqlx::query!(
        r#"
		SELECT
            s1.who as "who: Person",
            s1.confirmed_at,
            s1.converted_at,
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
                if r.stale {
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

pub async fn confirmable_session(db: &PgPool, by: Person) -> sqlx::Result<Option<i32>> {
    sqlx::query_scalar!(
        r#"
        SELECT id
        FROM sessions
        WHERE confirmed_at IS NULL AND who = $1
        ORDER BY created_at DESC
        LIMIT 1
        "#,
        !by as Person
    )
    .fetch_optional(db)
    .await
}
