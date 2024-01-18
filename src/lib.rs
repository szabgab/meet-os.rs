use std::env;
use std::path;

use serde::{Deserialize, Serialize};
use surrealdb::engine::local::{Db, RocksDb};
use surrealdb::Surreal;

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    pub email: String,
    pub name: String,
    pub code: String,
    pub verified: bool,
    pub date: String,
}

async fn get_database() -> surrealdb::Result<Surreal<Db>> {
    let database_folder = if let Ok(val) = env::var("DATABASE_PATH") {
        path::PathBuf::from(val)
    } else {
        let current_dir = env::current_dir().unwrap();
        current_dir.join("db")
    };

    let db = Surreal::new::<RocksDb>(database_folder).await?;
    db.use_ns("counter_ns").use_db("counter_db").await?;

    // Maybe do this only when we create the database
    let _response = db
        .query("DEFINE INDEX user_email ON TABLE user COLUMNS email UNIQUE")
        .await?;
    Ok(db)
}

pub async fn add_user(user: &User) -> surrealdb::Result<()> {
    let db = get_database().await?;
    let response = db
        .query(
            "CREATE user SET name=$name, email=$email, date=$date, code=$code, verified=$verified;",
        )
        .bind(("name", &user.name))
        .bind(("email", &user.email))
        .bind(("date", &user.date))
        .bind(("code", &user.code))
        .bind(("verified", user.verified))
        .await?;

    match response.check() {
        Ok(_entries) => {
            //let entries: Vec<User> = entries.take(0)?;
            // fetching the first (and hopefully only) entry
            //if let Some(_entry) = entries.into_iter().next() {
            //println!("{}", entry.count);
            //}

            Ok(())
        }
        Err(err) => {
            //eprintln!("Could not add entry {}", err);
            Err(err)
        }
    }
}
