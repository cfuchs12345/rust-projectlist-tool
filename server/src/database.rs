use sea_orm::{Database, DatabaseConnection, DbErr};
use sea_orm_migration::prelude::*;
use std::env;

use sqlx::{sqlite::SqliteConnectOptions, ConnectOptions};
use std::str::FromStr;

pub async fn connect() -> Result<DatabaseConnection, DbErr> {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    
    create_db_file(&db_url);

    let opts = SqliteConnectOptions::from_str(&db_url);
    opts.expect("invalid options").create_if_missing(true).connect();

    // establish connection to database and apply migrations
    // -> create post table if not exists
    let con = Database::connect(&db_url).await;

    con
}

fn create_db_file(db_url: &str) {
    let filename = db_url.strip_prefix("sqlite:").expect("invalid db url").strip_suffix("?mode=rwc").expect("invalid db url");

    let path = std::path::Path::new(&filename);

    if !path.exists() {
        std::fs::File::create(path).unwrap();
    }
}

pub(crate) async fn migrate(conn: &DatabaseConnection) {
    migration::Migrator::up(conn, None).await.unwrap();
}
