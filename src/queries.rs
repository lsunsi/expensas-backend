pub mod expense;
pub mod session;

use serde::{Deserialize, Serialize};
use sqlx::Type;

pub async fn init(env: &crate::env::Env) -> anyhow::Result<sqlx::PgPool> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect(&env.database_url)
        .await?;

    sqlx::migrate!().run(&pool).await?;
    Ok(pool)
}

#[derive(Debug, Clone, Copy, PartialEq, Type, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Type, Serialize, Deserialize)]
#[sqlx(type_name = "split")]
pub enum Split {
    Proportional,
    Arbitrary,
    Evenly,
}
