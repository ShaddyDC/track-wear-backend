use std::fmt::Debug;
use std::time::SystemTime;

use crate::api::user_management::models::{User, UserLoggedIn, UserOut};
use crate::api::user_management::sessions::UserSession;
use crate::db::DbConn;
use crate::error::ErrorResponse;
use crate::schema;
use crate::schema::users;
use crate::settings::Settings;
use diesel::prelude::*;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rocket::http::CookieJar;
use rocket::http::{Cookie, Status};
use rocket::serde::json::Json;
use rocket::State;
use serde::{Deserialize, Serialize};

#[derive(Insertable, AsChangeset)]
#[table_name = "users"]
pub struct NewUser {
    pub sub: String,
    pub username: String,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TokenClaims {
    pub sub: String,
    pub email: String,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub(super) struct SessionCookie {
    pub(super) session_key: String,
    pub(super) creation_time: SystemTime,
}

#[get("/check_login")]
pub(crate) async fn check_login(user: UserLoggedIn) -> Json<UserOut> {
    Json(user.0)
}

#[get("/check_login", rank = 2)]
pub(crate) async fn check_login_unauthorised() -> ErrorResponse {
    ErrorResponse::new(Status { code: 401 }, "Login required".to_string())
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

    let session_key = generate_session_key();

    tokens
        .sessions
        .lock()
        .map_err(|_| {
            ErrorResponse::new(
                Status { code: 500 },
                "Couldn't update user session".to_string(),
            )
        })?
        .insert(session_key.clone(), claims.sub);

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

fn generate_session_key() -> String {
    const LEN: usize = 32;

    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(LEN)
        .map(char::from)
        .collect()
}
