mod cors;
mod db;
mod error;
mod schema;
mod user_management;

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;
extern crate dotenv;
#[macro_use]
extern crate diesel_migrations;

use cors::CORS;
use db::establish_connection;
use user_management::UserSession;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[launch]
async fn rocket() -> _ {
    dotenv::dotenv().ok();

    let pool = establish_connection();

    rocket::build()
        .attach(CORS)
        .manage(pool)
        .manage(UserSession::new())
        .mount("/", routes![index])
        .mount(
            "/api/v1/",
            routes![
                index,
                crate::user_management::get_users,
                crate::user_management::login,
                crate::user_management::check_login,
                crate::user_management::check_login_unauthorised,
            ],
        )
}
