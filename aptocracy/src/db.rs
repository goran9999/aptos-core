use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};

pub type PgPool = Pool<ConnectionManager<PgConnection>>;

pub fn init_db() -> PgPool {
    let database_url =
        std::env::var("DATABASE_URL").expect("Failed to load database url env variable");

    let manager = ConnectionManager::new(database_url);

    let db_pool = Pool::builder().build(manager).unwrap();

    db_pool
}
