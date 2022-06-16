use crate::{env::Env, queries::Person};
use anyhow::Context;
use axum::{
    body::Body,
    extract::{FromRequest, RequestParts},
    http::StatusCode,
};
use hmac::Mac;

pub type Hmac = hmac::Hmac<sha2::Sha256>;

pub fn hmac(env: &Env) -> anyhow::Result<Hmac> {
    Hmac::new_from_slice(env.secret.as_ref())
        .ok()
        .context("Hmac init failed")
}

pub struct SessionAsk(pub i32);
const SESSION_ASK_KEY: &str = "ask";

#[axum::async_trait]
impl FromRequest<Body> for SessionAsk {
    type Rejection = StatusCode;

    async fn from_request(req: &mut RequestParts<Body>) -> Result<Self, Self::Rejection> {
        let mut hmac = req
            .extensions()
            .get::<crate::auth::Hmac>()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?
            .to_owned();

        let (header, sig) = req
            .headers()
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.rsplit_once('/'))
            .and_then(|(h, s)| base64::decode(s).ok().map(|s| (h, s)))
            .ok_or(StatusCode::BAD_REQUEST)?;

        hmac.update(header.as_ref());
        hmac.verify_slice(&sig).map_err(|_| StatusCode::FORBIDDEN)?;

        let id = header
            .split_once('/')
            .filter(|(k, _)| *k == SESSION_ASK_KEY)
            .and_then(|(_, id)| id.parse::<i32>().ok())
            .ok_or(StatusCode::BAD_REQUEST)?;

        Ok(SessionAsk(id))
    }
}

impl SessionAsk {
    pub fn response(self, hmac: &crate::auth::Hmac) -> Result<String, StatusCode> {
        let mut hmac = hmac.to_owned();
        let msg = format!("{SESSION_ASK_KEY}/{}", self.0);

        hmac.update(msg.as_ref());
        let sig = base64::encode(hmac.finalize().into_bytes());

        Ok(format!("{msg}/{sig}"))
    }
}

pub struct Session {
    pub who: Person,
    pub id: i32,
}
const SESSION_KEY: &str = "ses";

#[axum::async_trait]
impl FromRequest<Body> for Session {
    type Rejection = StatusCode;

    async fn from_request(req: &mut RequestParts<Body>) -> Result<Self, Self::Rejection> {
        let mut hmac = req
            .extensions()
            .get::<crate::auth::Hmac>()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?
            .to_owned();

        let (header, sig) = req
            .headers()
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.rsplit_once('/'))
            .and_then(|(h, s)| base64::decode(s).ok().map(|s| (h, s)))
            .ok_or(StatusCode::BAD_REQUEST)?;

        hmac.update(header.as_ref());
        hmac.verify_slice(sig.as_ref())
            .map_err(|_| StatusCode::FORBIDDEN)?;

        let (id, who) = header
            .split_once('/')
            .filter(|(k, _)| *k == SESSION_KEY)
            .and_then(|(_, v)| v.split_once('/'))
            .and_then(|(id, who)| match (id.parse::<i32>(), who) {
                (Ok(id), "ale") => Some((id, Person::Ale)),
                (Ok(id), "lu") => Some((id, Person::Lu)),
                _ => None,
            })
            .ok_or(StatusCode::BAD_REQUEST)?;

        Ok(Session { who, id })
    }
}

impl Session {
    pub fn response(self, hmac: &crate::auth::Hmac) -> Result<String, StatusCode> {
        let mut hmac = hmac.to_owned();
        let msg = format!(
            "{SESSION_KEY}/{}/{}",
            self.id,
            match self.who {
                Person::Ale => "ale",
                Person::Lu => "lu",
            }
        );

        hmac.update(msg.as_ref());
        let sig = base64::encode(hmac.finalize().into_bytes());

        Ok(format!("{msg}/{sig}"))
    }
}
