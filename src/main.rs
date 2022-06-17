mod api;
mod db;
mod error;
mod schema;
mod settings;

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;
extern crate dotenv;
#[macro_use]
extern crate diesel_migrations;

use crate::api::cloth_management::{add_wear, create, delete, edit, get_cloth, list};
use crate::api::user_management::login;
use api::user_management::sessions::UserSession;
use db::{run_db_migrations, DbConn};
use rocket::fairing::AdHoc;
use settings::Settings;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[launch]
async fn rocket() -> _ {
    dotenv::dotenv().ok();

    let settings = Settings::new();

    rocket::build()
        .attach(DbConn::fairing())
        .attach(AdHoc::on_ignite("Run Migrations", run_db_migrations))
        .manage(UserSession::new())
        .manage(settings)
        .mount("/", routes![index])
        .mount(
            "/api/v1/",
            routes![
                index,
                login::login,
                login::check_login,
                login::check_login_unauthorised,
                create::create_cloth,
                edit::edit_cloth,
                list::get_clothes,
                get_cloth::get_cloth,
                delete::delete_cloth,
                get_cloth::get_cloth_image,
                add_wear::add_wear,
            ],
        )
}
