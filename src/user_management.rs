use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Mutex;
use std::time::{Duration, SystemTime};

use crate::db::DbConn;
use crate::error::{ApiError, ErrorResponse};
use crate::schema;
use crate::schema::users;
use crate::settings::Settings;
use diesel::prelude::*;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rocket::http::CookieJar;
use rocket::http::{Cookie, Status};
use rocket::outcome::{try_outcome, IntoOutcome};
use rocket::request::{self, FromRequest, Outcome};
use rocket::serde::json::Json;
use rocket::{Request, State};
use serde::{Deserialize, Serialize};

#[derive(Queryable, Debug)]
pub struct User {
    pub id: i32,
    pub sub: String,
    pub username: String,
    pub email: String,
}

#[derive(Serialize)]
pub struct UserOut {
    pub id: i32,
    pub sub: String,
    pub username: String,
    pub email: String,
}

#[derive(Insertable, AsChangeset)]
#[table_name = "users"]
pub struct NewUser {
    pub sub: String,
    pub username: String,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub email: String,
    pub name: String,
}

pub(crate) struct UserSession {
    sessions: Mutex<HashMap<String, String>>,
}

impl UserSession {
    pub(crate) fn new() -> UserSession {
        UserSession {
            sessions: Mutex::new(HashMap::<String, String>::new()),
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserOut {
    type Error = ApiError;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let session_cookie = try_outcome!(req
            .cookies()
            .get_private("session")
            .ok_or_else(|| ApiError::new("No session set".to_string()))
            .or_forward(()));

        let session_cookie_string = session_cookie.value();

        let session_cookie_value =
            try_outcome!(serde_json::from_str::<SessionCookie>(session_cookie_string)
                .map_err(|_| ApiError::new("Couldn't parse session".to_string()))
                .or_forward(()));

        let session_age = try_outcome!(session_cookie_value
            .creation_time
            .elapsed()
            .map_err(|_| ApiError::new("Couldn't determine session age".to_string()))
            .or_forward(()));

        let max_age = Duration::from_secs(60 * 60 * 24 * 30);
        if session_age > max_age {
            return request::Outcome::Failure((
                Status { code: 401 },
                ApiError::new("Session too old".to_string()),
            ));
        }

        let sessions = try_outcome!(req.guard::<&State<UserSession>>().await.map_failure(|_| {
            (
                Status { code: 500 },
                ApiError::new("Couldn't get UserSession".to_string()),
            )
        }));
        let user_id = {
            let sessions = try_outcome!(sessions
                .sessions
                .lock()
                .map_err(|_| ApiError::new("Couldn't get user sessions".to_string()))
                .or_forward(()));

            try_outcome!(sessions
                .get(&session_cookie_value.session_key)
                .ok_or_else(|| ApiError::new("No session found".to_string()))
                .or_forward(()))
            .clone()
        };

        let conn = try_outcome!(req.guard::<DbConn>().await.map_failure(|_| {
            (
                Status { code: 500 },
                ApiError::new("Couldn't get database connection".to_string()),
            )
        }));

        use schema::users::dsl::*;

        let user_vec = try_outcome!(
            conn.run(|c| users
                .filter(sub.eq(user_id))
                .load::<User>(c)
                .map_err(|_| ApiError::new("Couldn't load user from database".to_string()))
                .or_forward(()))
                .await
        );

        let user = try_outcome!(user_vec
            .first()
            .ok_or_else(|| { ApiError::new("User not in database".to_string()) })
            .or_forward(()));

        Outcome::Success(UserOut {
            id: user.id,
            sub: user.sub.clone(),
            username: user.username.clone(),
            email: user.email.clone(),
        })
    }
}
#[derive(Serialize, Deserialize)]
struct SessionCookie {
    session_key: String,
    creation_time: SystemTime,
}

fn generate_session_key() -> String {
    const LEN: usize = 32;

    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(LEN)
        .map(char::from)
        .collect()
}

#[post("/login", data = "<token>")]
pub(crate) async fn login(
    token: String,
    tokens: &State<UserSession>,
    conn: DbConn,
    cookies: &CookieJar<'_>,
    settings: &State<Settings>,
) -> Result<&'static str, ErrorResponse> {
    let parser = jsonwebtoken_google::Parser::new(&settings.google_client_id);
    let claims = parser.parse::<TokenClaims>(&token).await.map_err(|_| {
        ErrorResponse::new(
            Status { code: 500 },
            "Couldn't validate Google account".to_string(),
        )
    })?;

    let new_user = NewUser {
        sub: claims.sub.clone(),
        email: claims.email,
        username: claims.name,
    };

    use schema::users::dsl::*;

    conn.run(move |c| {
        diesel::insert_into(users)
            .values(&new_user)
            .on_conflict(sub)
            .do_update()
            .set(&new_user)
            .get_result::<User>(c)
            .map_err(|_| {
                ErrorResponse::new(Status { code: 500 }, "Couldn't update user".to_string())
            })
    })
    .await?;

    let mut sessions = tokens.sessions.lock().map_err(|_| {
        ErrorResponse::new(
            Status { code: 500 },
            "Couldn't update user session".to_string(),
        )
    })?;
    let session_key = generate_session_key();

    sessions.insert(session_key.clone(), claims.sub);

    let cookie = SessionCookie {
        session_key,
        creation_time: SystemTime::now(),
    };

    let cookie_string = serde_json::to_string(&cookie).map_err(|err| {
        ErrorResponse::new(
            Status { code: 500 },
            format!("Couldn't create session cookie {}", err),
        )
    })?;

    cookies.add_private(Cookie::new("session", cookie_string));

    Ok("Success")
}

#[get("/check_login")]
pub(crate) async fn check_login(user: UserOut) -> Json<UserOut> {
    Json(user)
}

#[get("/check_login", rank = 2)]
pub(crate) async fn check_login_unauthorised() -> ErrorResponse {
    ErrorResponse::new(Status { code: 401 }, "Login required".to_string())
}
