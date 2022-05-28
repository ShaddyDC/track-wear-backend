mod db;
mod schema;

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;
extern crate dotenv;
#[macro_use]
extern crate diesel_migrations;

use self::diesel::prelude::*;
use db::establish_connection;
use rocket::serde::json::Json;
use rocket::State;
use schema::users;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/users")]
fn get_users(pool: &State<db::Pool>) -> Json<Vec<String>> {
    let conn = pool.get().expect("migration connection failure");

    let results = users::table
        .limit(5)
        .load::<User>(&conn)
        .expect("Error loading users");

    let names = results.into_iter().map(|x| x.username).collect::<Vec<_>>();

    Json(names)
}

#[derive(Queryable)]
pub struct User {
    pub id: i32,
    pub username: String,
}

#[launch]
fn rocket() -> _ {
    dotenv::dotenv().ok();

    let pool = establish_connection();

    rocket::build()
        .manage(pool)
        .mount("/", routes![index, get_users])
}
