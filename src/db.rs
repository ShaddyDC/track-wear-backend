use diesel_migrations::embed_migrations;
use rocket::{Build, Rocket};
use rocket_sync_db_pools::{database, diesel};

#[database("track_wear")]
pub(crate) struct DbConn(diesel::PgConnection);

embed_migrations!();

pub(crate) async fn run_db_migrations(rocket: Rocket<Build>) -> Rocket<Build> {
    let conn = DbConn::get_one(&rocket).await.expect("database connection");
    conn.run(|c| embedded_migrations::run(c))
        .await
        .expect("can run migrations");

    rocket
}
