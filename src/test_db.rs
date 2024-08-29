use chrono::{DateTime, Utc};

use crate::db;
use meetings::{Group, User};

#[async_test]
async fn test_db_get_empty_lists() {
    let database_name = format!("test-name-{}", rand::random::<f64>());
    let database_namespace = format!("test-namespace-{}", rand::random::<f64>());

    let dbh = db::get_database(&database_name, &database_namespace).await;

    let events = db::get_events(&dbh).await.unwrap();
    assert!(events.is_empty());

    let audits = db::get_audit(&dbh).await.unwrap();
    assert!(audits.is_empty());

    let groups = db::get_groups(&dbh).await.unwrap();
    assert!(groups.is_empty());
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

    let user_foo = User {
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

    let res = db::add_user(&dbh, &user_foo).await.unwrap();
    assert_eq!(res, ());

    let users = db::get_users(&dbh).await.unwrap();
    assert_eq!(users.len(), 1);
    assert_eq!(users[0].name, user_foo.name);
    assert_eq!(users[0], user_foo);

    let res = db::add_user(&dbh, &user_foo).await;
    assert!(res.is_err());
    let err = res.err().unwrap().to_string();
    assert!(err.contains("There was a problem with the database: Database index `user_email` already contains 'foo@meet-os.com'"));

    let user_peti = User {
        name: String::from("Peti Bar"),
        email: String::from("peti@meet-os.com"),
        ..user_foo.clone()
    };

    let res = db::add_user(&dbh, &user_peti).await;
    assert!(res.is_err());
    let err = res.err().unwrap().to_string();
    assert!(err.contains(
        "There was a problem with the database: Database index `user_uid` already contains 1"
    ));

    let user_peti = User {
        uid: 2,
        name: String::from("Peti Bar"),
        email: String::from("peti@meet-os.com"),
        ..user_foo.clone()
    };
    let res = db::add_user(&dbh, &user_peti).await.unwrap();
    assert_eq!(res, ());

    let users = db::get_users(&dbh).await.unwrap();
    assert_eq!(users.len(), 2);
    // TODO: should we fix the order? Without that these test need to take into account the lack of order
    // assert_eq!(users[0], user_foo);
    // assert_eq!(users[1], user_peti);

    let utc: DateTime<Utc> = Utc::now();

    let rust_maven = Group {
        gid: 1,
        owner: 2,
        name: String::from("Rust Maven"),
        location: String::new(),
        description: String::new(),
        creation_date: utc,
    };
    let res = db::add_group(&dbh, &rust_maven).await.unwrap();
    assert_eq!(res, ());

    let groups = db::get_groups(&dbh).await.unwrap();
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0], rust_maven);
}
