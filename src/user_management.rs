use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Mutex;

use crate::schema::users;
use crate::{db, schema};
use diesel::prelude::*;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
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
pub(crate) fn get_users(pool: &State<db::Pool>) -> Json<Vec<String>> {
    let conn = pool.get().expect("db connection failure");

    let results = users::table
        .limit(5)
        .load::<User>(&conn)
        .expect("Error loading users");

    let names = results.into_iter().map(|x| x.username).collect::<Vec<_>>();

    Json(names)
}

fn generate_token() -> String {
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
) -> String {
    let client_id = "513324624986-fn538769dc89nlp075t083h03ihnjldi.apps.googleusercontent.com";
    let parser = jsonwebtoken_google::Parser::new(client_id);
    let claims = parser.parse::<TokenClaims>(&token).await;

    match claims {
        Ok(claims) => {
            let conn = match pool.get() {
                Ok(conn) => conn,
                Err(err) => return err.to_string(),
            };

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
                .unwrap();

            let mut tokens = tokens.sessions.lock().unwrap();
            let token = generate_token();

            tokens.insert(token.clone(), claims.sub);

            token
        }
        Err(err) => err.to_string(),
    }
}

#[post("/check_login", data = "<session>")]
pub(crate) async fn check_login(
    session: String,
    sessions: &State<UserSession>,
    pool: &State<db::Pool>,
) -> String {
    match sessions.sessions.lock().unwrap().get(&session) {
        Some(token) => {
            let conn = match pool.get() {
                Ok(conn) => conn,
                Err(err) => return err.to_string(),
            };

            use schema::users::dsl::*;

            match users
                .filter(sub.eq(token))
                .load::<User>(&conn)
                .unwrap()
                .first()
            {
                Some(user) => {
                    println!("Loaded {:?}", user);

                    "valid".to_string()
                }
                None => "user not found".to_string(),
            }
        }
        None => "invalid".to_string(),
    }
}
