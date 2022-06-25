pub mod session;

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

pub async fn init(env: &crate::env::Env) -> anyhow::Result<sqlx::PgPool> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect(&env.database_url)
        .await?;

    sqlx::migrate!().run(&pool).await?;
    Ok(pool)
}
