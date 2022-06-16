mod cloth_management;
mod db;
mod error;
mod schema;
mod settings;
mod user_management;

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;
extern crate dotenv;
#[macro_use]
extern crate diesel_migrations;

use db::{run_db_migrations, DbConn};
use rocket::fairing::AdHoc;
use settings::Settings;
use user_management::UserSession;

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
                crate::user_management::login,
                crate::user_management::check_login,
                crate::user_management::check_login_unauthorised,
                crate::cloth_management::create_cloth,
                crate::cloth_management::edit_cloth,
                crate::cloth_management::get_clothes,
                crate::cloth_management::get_cloth,
                crate::cloth_management::delete_cloth,
                crate::cloth_management::get_cloth_image,
                crate::cloth_management::add_wear,
            ],
        )
}
