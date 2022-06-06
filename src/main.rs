mod cloth_management;
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
        .manage(pool)
        .manage(UserSession::new())
        .mount("/", routes![index])
        .mount(
            "/api/v1/",
            routes![
                index,
                crate::user_management::login,
                crate::user_management::check_login,
                crate::user_management::check_login_unauthorised,
                crate::cloth_management::create_cloth,
                crate::cloth_management::get_clothes,
                crate::cloth_management::get_cloth,
                crate::cloth_management::delete_cloth,
                crate::cloth_management::get_cloth_image,
            ],
        )
}
