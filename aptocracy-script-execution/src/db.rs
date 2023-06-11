use diesel::pg::PgConnection;
use diesel::r2d2::{Builder, ConnectionManager, Pool};

pub type PgPool = Pool<ConnectionManager<PgConnection>>;

pub fn init_db() -> PgPool {
    let db_url = std::env::var("DATABASE_URL").expect("Failed to load db url");

    let connection_manager = ConnectionManager::new(db_url);

    let builder = Builder::new().build(connection_manager).unwrap();

    builder
}
