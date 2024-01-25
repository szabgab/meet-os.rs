use std::env;

use rocket::fairing::AdHoc;
use surrealdb::engine::local::{Db, RocksDb};
use surrealdb::opt::Resource;
use surrealdb::Surreal;

use crate::User;

/// # Panics
///
/// Panics when it fails to create the database folder or set up the database.
#[must_use]
pub fn init() -> AdHoc {
    AdHoc::on_ignite("Managed Database Connection", |rocket| async {
        let database_folder = env::var("DATABASE_PATH").unwrap_or_else(|_| "./db".to_owned());
        rocket::info!("get_database from folder '{:?}'", database_folder);
        let db = Surreal::new::<RocksDb>(database_folder).await.unwrap();
        rocket::info!("get_database connected");
        db.use_ns("counter_ns").use_db("counter_db").await.unwrap();
        rocket::info!("get_database namespace set");
        // Maybe do this only when we create the database
        db.query("DEFINE INDEX user_email ON TABLE user COLUMNS email UNIQUE")
            .await
            .unwrap()
            .check()
            .unwrap();
        rocket.manage(db)
    })
}

pub async fn add_user(db: &Surreal<Db>, user: &User) -> surrealdb::Result<()> {
    rocket::info!("add user email: '{}' code: '{}'", user.email, user.code);

    db.create(Resource::from("user")).content(user).await?;

    Ok(())
}

pub async fn verify_code(
    db: &Surreal<Db>,
    process: &str,
    code: &str,
) -> surrealdb::Result<Option<User>> {
    rocket::info!("verification code: '{code}' process = '{process}'");
    let verified = true;

    let mut response = db
        .query("UPDATE ONLY user SET verified=$verified, code='' WHERE code=$code AND process=$process;")
        .bind(("verified", verified))
        .bind(("code", code))
        .bind(("process", process))
        .await?;

    let entry: Option<User> = response.take(0)?;

    if let Some(entry) = entry.as_ref() {
        rocket::info!(
            "verification ok '{}', '{}', '{}'",
            entry.name,
            entry.email,
            entry.process
        );
    }

    Ok(entry)
}

pub async fn get_user_by_email(db: &Surreal<Db>, email: &str) -> surrealdb::Result<Option<User>> {
    rocket::info!("get_user_by_email: '{email}'");
    rocket::info!("has db");
    let mut response = db
        .query("SELECT * FROM user WHERE email=$email;")
        .bind(("email", email))
        .await?;

    let entry: Option<User> = response.take(0)?;

    if let Some(entry) = entry.as_ref() {
        rocket::info!("************* {}, {}", entry.name, entry.email);
    }

    Ok(entry)
}

pub async fn add_login_code_to_user(
    db: &Surreal<Db>,
    email: &str,
    process: &str,
    code: &str,
) -> surrealdb::Result<Option<User>> {
    rocket::info!("add_login_code_to_user: '{email}', '{process}', '{code}'");

    rocket::info!("has db");
    let mut response = db
        .query("UPDATE user SET code=$code, process=$process WHERE email=$email;")
        .bind(("email", email))
        .bind(("process", process))
        .bind(("code", code))
        .await?;

    let entry: Option<User> = response.take(0)?;

    if let Some(entry) = entry.as_ref() {
        rocket::info!("entry: '{}' '{}'", entry.email, entry.process);
    }

    Ok(entry)
}
