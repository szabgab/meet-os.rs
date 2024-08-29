use chrono::{DateTime, Utc};

use crate::db;
use meetings::User;

#[async_test]
async fn test_db_get_empty_lists() {
    let database_name = format!("test-name-{}", rand::random::<f64>());
    let database_namespace = format!("test-namespace-{}", rand::random::<f64>());

    let dbh = db::get_database(&database_name, &database_namespace).await;

    let events = db::get_events(&dbh).await.unwrap();
    assert!(events.is_empty());

    let audits = db::get_audit(&dbh).await.unwrap();
    assert!(audits.is_empty());
}

#[async_test]
async fn test_db_get_none() {
    let database_name = format!("test-name-{}", rand::random::<f64>());
    let database_namespace = format!("test-namespace-{}", rand::random::<f64>());

    let dbh = db::get_database(&database_name, &database_namespace).await;

    let event = db::get_event_by_eid(&dbh, 1).await.unwrap();
    assert!(event.is_none());

    let user = db::get_user_by_email(&dbh, "bad_email").await.unwrap();
    assert!(user.is_none());

    let user = db::get_user_by_id(&dbh, 23).await.unwrap();
    assert!(user.is_none());

    let user = db::get_user_by_code(&dbh, "register", "hello")
        .await
        .unwrap();
    assert!(user.is_none());
}

#[async_test]
async fn test_db_user() {
    let database_name = format!("test-name-{}", rand::random::<f64>());
    let database_namespace = format!("test-namespace-{}", rand::random::<f64>());

    let dbh = db::get_database(&database_name, &database_namespace).await;

    let utc: DateTime<Utc> = Utc::now();

    let user = User {
        uid: 1,
        name: String::from("Foo Bar"),
        email: String::from("foo@meet-os.com"),
        password: String::from("should be hashed password"),
        code: String::from("generated code"),
        process: String::from("register"),
        verified: false,
        registration_date: utc,
        verification_date: None,
        github: None,
        gitlab: None,
        linkedin: None,
        about: None,
    };

    let res = db::add_user(&dbh, &user).await.unwrap();
    assert_eq!(res, ());
}
