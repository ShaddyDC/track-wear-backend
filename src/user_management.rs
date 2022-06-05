use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Mutex;

use crate::error::{ApiErrorResponse, ErrorResponse};
use crate::schema::users;
use crate::{db, schema};
use diesel::prelude::*;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
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

#[get("/users")]
pub(crate) fn get_users(pool: &State<db::Pool>) -> Result<Json<Vec<String>>, ApiErrorResponse> {
    let conn = pool
        .get()
        .map_err(|_| ErrorResponse::new(Status { code: 500 }, "Couldn't connect to database"))?;

    let users = users::table.limit(5).load::<User>(&conn).map_err(|_| {
        ErrorResponse::new(Status { code: 500 }, "Couldn't load users from database")
    })?;

    let names = users.into_iter().map(|x| x.username).collect::<Vec<_>>();

    Ok(Json(names))
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
    pool: &State<db::Pool>,
) -> Result<String, ApiErrorResponse<'static>> {
    let client_id = "513324624986-fn538769dc89nlp075t083h03ihnjldi.apps.googleusercontent.com";
    let parser = jsonwebtoken_google::Parser::new(client_id);
    let claims = parser.parse::<TokenClaims>(&token).await.map_err(|_| {
        ErrorResponse::new(Status { code: 500 }, "Couldn't validate Google account")
    })?;

    let conn = pool
        .get()
        .map_err(|_| ErrorResponse::new(Status { code: 500 }, "Couldn't connect to database"))?;

    let new_user = NewUser {
        sub: claims.sub.clone(),
        email: claims.email,
        username: claims.name,
    };

    use schema::users::dsl::*;

    diesel::insert_into(users)
        .values(&new_user)
        .on_conflict(sub)
        .do_update()
        .set(&new_user)
        .get_result::<User>(&conn)
        .map_err(|_| ErrorResponse::new(Status { code: 500 }, "Couldn't update user"))?;

    let mut sessions = tokens
        .sessions
        .lock()
        .map_err(|_| ErrorResponse::new(Status { code: 500 }, "Couldn't update user session"))?;
    let session_key = generate_session_key();

    sessions.insert(session_key.clone(), claims.sub);

    Ok(session_key)
}

#[post("/check_login", data = "<session>")]
pub(crate) async fn check_login(
    session: String,
    sessions: &State<UserSession>,
    pool: &State<db::Pool>,
) -> Result<Json<UserOut>, ApiErrorResponse<'static>> {
    let sessions = sessions
        .sessions
        .lock()
        .map_err(|_| ErrorResponse::new(Status { code: 500 }, "Couldn't get user sessions"))?;

    let user_id = sessions
        .get(&session)
        .ok_or_else(|| ErrorResponse::new(Status { code: 401 }, "No session found"))?;

    let conn = pool
        .get()
        .map_err(|_| ErrorResponse::new(Status { code: 500 }, "Couldn't connect to database"))?;

    use schema::users::dsl::*;

    let user_vec = users
        .filter(sub.eq(user_id))
        .load::<User>(&conn)
        .map_err(|_| {
            ErrorResponse::new(Status { code: 500 }, "Couldn't load user from database")
        })?;

    let user = user_vec
        .first()
        .ok_or_else(|| ErrorResponse::new(Status { code: 401 }, "User not in database"))?;

    Ok(Json(UserOut {
        id: user.id,
        sub: user.sub.clone(),
        username: user.username.clone(),
        email: user.email.clone(),
    }))
}
