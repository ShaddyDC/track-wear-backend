use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use diesel_migrations::embed_migrations;
use diesel_migrations::run_pending_migrations;

embed_migrations!();

fn run_migrations(connection: &mut PgConnection) {
    run_pending_migrations(connection).expect("Failed migration");
}

pub(crate) type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub fn establish_connection() -> Pool {
    dotenv::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::new(manager).expect("db pool failure");

    let mut conn = pool.get().expect("migration connection failure");
    run_migrations(&mut conn);

    pool
}
