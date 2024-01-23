use std::env;
use std::path;

use surrealdb::engine::local::{Db, RocksDb};
use surrealdb::Surreal;

use crate::User;

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
    rocket::info!("add user email: '{}' code: '{}'", user.email, user.code);
    let db = get_database().await?;
    let response = db
        .query(
            "CREATE user SET name=$name, email=$email, date=$date, process=$process, code=$code, verified=$verified;",
        )
        .bind(("name", &user.name))
        .bind(("email", &user.email))
        .bind(("date", &user.date))
        .bind(("process", &user.process))
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

pub async fn verify_code(process: &str, code: &str) -> surrealdb::Result<Option<User>> {
    rocket::info!("verification code: '{code}' process = '{process}'");
    let db = get_database().await?;
    let verified = true;
    let response = db
        .query("UPDATE ONLY user SET verified=$verified, code='' WHERE code=$code AND process=$process;")
        .bind(("verified", verified))
        .bind(("code", code))
        .bind(("process", process))
        .await?;

    match response.check() {
        Ok(mut entries) => {
            let entries: Vec<User> = entries.take(0)?;
            match entries.first() {
                Some(entry) => {
                    rocket::info!(
                        "verification ok '{}', '{}', '{}'",
                        entry.name,
                        entry.email,
                        entry.process
                    );
                    Ok(Some(entry.clone()))
                }
                None => Ok(None),
            }
        }
        Err(err) => Err(err),
    }
}

pub async fn get_user_by_email(email: &str) -> surrealdb::Result<Option<User>> {
    rocket::info!("get_user_by_email: '{email}'");
    let db = get_database().await?;
    rocket::info!("has db");
    let response = db
        .query("SELECT * FROM user WHERE email=$email;")
        .bind(("email", email))
        .await?;

    match response.check() {
        Ok(mut entries) => {
            let entries: Vec<User> = entries.take(0)?;
            match entries.first() {
                Some(entry) => {
                    rocket::info!("************* {}, {}", entry.name, entry.email);
                    Ok(Some(entry.clone()))
                }
                None => Ok(None),
            }
        }
        Err(err) => Err(err),
    }
}
