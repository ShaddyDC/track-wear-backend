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

use crate::api::item_management::{add_use, create, delete, edit, get_item, list};
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
                create::create_item,
                edit::edit_item,
                list::get_items,
                get_item::get_item,
                delete::delete_item,
                get_item::get_item_image,
                add_use::add_use,
            ],
        )
}
