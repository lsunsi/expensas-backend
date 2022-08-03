use crate::{env::Env, queries::Person};
use axum::{
    body::Body,
    extract::{FromRequest, RequestParts},
    http::StatusCode,
};
use axum_extra::extract::{
    cookie::{Cookie, Key, SameSite},
    PrivateCookieJar,
};
use time::{Duration, OffsetDateTime};

pub fn key(env: &Env) -> Key {
    axum_extra::extract::cookie::Key::from(env.secret.as_bytes())
}

pub struct SessionAsk(pub i32);

#[axum::async_trait]
impl FromRequest<Body> for SessionAsk {
    type Rejection = StatusCode;

    async fn from_request(req: &mut RequestParts<Body>) -> Result<Self, Self::Rejection> {
        let cookies = req
            .extract::<PrivateCookieJar>()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let id = cookies
            .get(COOKIE_SESSION_ASK)
            .and_then(|c| c.value().parse::<i32>().ok())
            .ok_or(StatusCode::BAD_REQUEST)?;

        Ok(SessionAsk(id))
    }
}

impl From<SessionAsk> for Cookie<'static> {
    fn from(s: SessionAsk) -> Self {
        let mut cookie = Cookie::new(COOKIE_SESSION_ASK, s.0.to_string());
        cookie.set_same_site(SameSite::Strict);
        cookie.set_http_only(true);
        cookie.set_path("/");
        cookie
    }
}

pub struct Session {
    pub who: Person,
    pub id: i32,
}

#[axum::async_trait]
impl FromRequest<Body> for Session {
    type Rejection = StatusCode;

    async fn from_request(req: &mut RequestParts<Body>) -> Result<Self, Self::Rejection> {
        let cookies = req
            .extract::<PrivateCookieJar>()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let (id, who) = cookies
            .get(COOKIE_SESSION)
            .as_ref()
            .and_then(|c| c.value().split_once('/'))
            .and_then(|(id, who)| match (id.parse::<i32>(), who) {
                (Ok(id), "ale") => Some((id, Person::Ale)),
                (Ok(id), "lu") => Some((id, Person::Lu)),
                _ => None,
            })
            .ok_or(StatusCode::BAD_REQUEST)?;

        Ok(Session { who, id })
    }
}

impl From<Session> for Cookie<'static> {
    fn from(Session { id, who }: Session) -> Self {
        let who = match who {
            Person::Ale => "ale",
            Person::Lu => "lu",
        };

        let mut cookie = Cookie::new(COOKIE_SESSION, format!("{id}/{who}"));
        cookie.set_expires(OffsetDateTime::now_utc() + Duration::weeks(12));
        cookie.set_same_site(SameSite::Strict);
        cookie.set_http_only(true);
        cookie.set_secure(true);

        cookie.set_path("/");
        cookie
    }
}

const COOKIE_SESSION_ASK: &str = "ask";
const COOKIE_SESSION: &str = "ses";
