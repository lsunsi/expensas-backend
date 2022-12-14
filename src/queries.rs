pub mod expense;
pub mod session;
pub mod summary;
pub mod transfer;

use serde::{Deserialize, Serialize};
use sqlx::Type;

pub async fn init(env: &crate::env::Env) -> anyhow::Result<sqlx::PgPool> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect(&env.database_url)
        .await?;

    sqlx::migrate!().run(&pool).await?;
    Ok(pool)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Type, Serialize, Deserialize)]
#[sqlx(type_name = "person")]
pub enum Person {
    Ale,
    Lu,
}

#[derive(Debug, Clone, Copy, Type, Serialize, Deserialize)]
#[sqlx(type_name = "split")]
pub enum Split {
    Proportional2to1,
    Proportional3to2,
    Arbitrary,
    Evenly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Type, Serialize, Deserialize)]
#[sqlx(type_name = "label")]
pub enum Label {
    Market,
    Delivery,
    Transport,
    Leisure,
    Water,
    Internet,
    Gas,
    Housing,
    Electricity,
    Furnitance,
}
