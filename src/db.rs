#![allow(clippy::std_instead_of_core)]

use std::env;

use rocket::fairing::AdHoc;
use surrealdb::engine::remote::ws::Client;
use surrealdb::engine::remote::ws::Ws;
use surrealdb::opt::Resource;
use surrealdb::Surreal;

use crate::{Counter, Event, Group, User};

/// # Panics
///
/// Panics when it fails to create the database folder or set up the database.
#[must_use]
pub fn fairing() -> AdHoc {
    // TODO handle errors here properly by using AdHoc::try_on_ignite instead of AdHoc::on_ignite.
    AdHoc::on_ignite("Managed Database Connection", |rocket| async {
        let db = get_database().await;
        rocket.manage(db)
    })
}

/// # Panics
///
/// Panics when it fails to create the database folder or set up the database.
pub async fn get_database() -> Surreal<Client> {
    let address = "127.0.0.1:8001";
    let db = Surreal::new::<Ws>(address).await.unwrap();
    let db_namespace =
        env::var("DATABASE_NAMESPACE").unwrap_or_else(|_| String::from("meet-os-ns"));
    let db_name = env::var("DATABASE_NAME").unwrap_or_else(|_| String::from("meet-os-ns"));
    db.use_ns(&db_namespace).use_db(&db_name).await.unwrap();
    // TODO: do this only when we create the database
    db.query("DEFINE INDEX user_email ON TABLE user COLUMNS email UNIQUE")
        .await
        .unwrap()
        .check()
        .unwrap();
    db
}

pub async fn add_user(db: &Surreal<Client>, user: &User) -> surrealdb::Result<()> {
    rocket::info!("add user email: '{}' code: '{}'", user.email, user.code);

    db.create(Resource::from("user")).content(user).await?;

    Ok(())
}

pub async fn add_group(db: &Surreal<Client>, group: &Group) -> surrealdb::Result<()> {
    rocket::info!("add group: '{}'", group.name);

    db.create(Resource::from("group")).content(group).await?;

    Ok(())
}

pub async fn verify_code(
    db: &Surreal<Client>,
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

pub async fn get_user_by_id(db: &Surreal<Client>, uid: usize) -> surrealdb::Result<Option<User>> {
    rocket::info!("get_user_by_id: '{uid}'");

    let mut response = db
        .query("SELECT * FROM user WHERE uid=$uid;")
        .bind(("uid", uid))
        .await?;

    let entry: Option<User> = response.take(0)?;

    if let Some(entry) = entry.as_ref() {
        rocket::info!("Foud user {}, {}", entry.name, entry.email);
    }

    Ok(entry)
}

pub async fn get_user_by_email(
    db: &Surreal<Client>,
    email: &str,
) -> surrealdb::Result<Option<User>> {
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
    db: &Surreal<Client>,
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

#[must_use]
pub async fn get_events_by_group_id(db: &Surreal<Client>, gid: usize) -> Vec<Event> {
    match get_events(db).await {
        Ok(events) => events
            .into_iter()
            .filter(|event| event.group_id == gid)
            .collect(),
        Err(_) => vec![],
    }
}

pub async fn get_users(db: &Surreal<Client>) -> surrealdb::Result<Vec<User>> {
    rocket::info!("get_users");
    let mut response = db.query("SELECT * FROM user;").await?;
    let entries: Vec<User> = response.take(0)?;
    for ent in &entries {
        rocket::info!("user name {}", ent.name);
    }
    Ok(entries)
}

pub async fn get_groups(db: &Surreal<Client>) -> surrealdb::Result<Vec<Group>> {
    rocket::info!("get_groups");
    let mut response = db.query("SELECT * FROM group;").await?;
    let entries: Vec<Group> = response.take(0)?;
    for ent in &entries {
        rocket::info!("group name {}", ent.name);
    }
    Ok(entries)
}

pub async fn get_group_by_gid(
    db: &Surreal<Client>,
    gid: usize,
) -> surrealdb::Result<Option<Group>> {
    rocket::info!("get_group_by_gid: '{gid}'");
    let mut response = db
        .query("SELECT * FROM group WHERE gid=$gid;")
        .bind(("gid", gid))
        .await?;

    let entry: Option<Group> = response.take(0)?;

    if let Some(entry) = entry.as_ref() {
        rocket::info!("Group name: {}", entry.name);
    }

    Ok(entry)
}

pub async fn get_events(db: &Surreal<Client>) -> surrealdb::Result<Vec<Event>> {
    rocket::info!("get_events");
    let mut response = db.query("SELECT * FROM group;").await?;
    let entries: Vec<Event> = response.take(0)?;
    for ent in &entries {
        rocket::info!("event name {}", ent.title);
    }
    Ok(entries)
}

/// # Panics
///
/// Panics when there is an error
pub async fn increment(db: &Surreal<Client>, name: &str) -> surrealdb::Result<usize> {
    // TODO: do this only when creatig the database
    let _response = db
        .query("DEFINE INDEX counter_name ON TABLE counter COLUMNS name UNIQUE")
        .await?;

    #[allow(clippy::separated_literal_suffix)]
    let response = db
        .query(
            "
            INSERT INTO counter (name, count)
                VALUES ($name, $count) ON DUPLICATE KEY UPDATE count += 1;
        ",
        )
        .bind(("name", name))
        .bind(("count", 1_i32))
        .await?;

    let mut entries = response.check()?;
    let entries: Vec<Counter> = entries.take(0)?;
    // fetching the first (and hopefully only) entry
    let entry = entries.into_iter().next().unwrap();
    let id: usize = entry.count.try_into().unwrap();

    Ok(id)
}

pub async fn get_event_by_eid(
    db: &Surreal<Client>,
    eid: usize,
) -> surrealdb::Result<Option<Event>> {
    rocket::info!("get_event_by_eid: '{eid}'");
    let mut response = db
        .query("SELECT * FROM event WHERE eid=$eid;")
        .bind(("eid", eid))
        .await?;

    let entry: Option<Event> = response.take(0)?;

    if let Some(entry) = entry.as_ref() {
        rocket::info!("Event title: {}", entry.title);
    }

    Ok(entry)
}
