use serde::Serialize;
use std::fmt::Debug;

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

pub(crate) struct UserLoggedIn(pub(crate) UserOut);
