#![allow(unused_macros, unused_imports)]

use regex::Regex;
use std::path::PathBuf;

use rocket::http::{ContentType, Status};
use rocket::local::blocking::Client;
use scraper::{Html, Selector};

use crate::test_lib::{params, read_code_from_email};

pub const FOO_EMAIL: &str = "foo@meet-os.com";
pub const FOO1_EMAIL: &str = "foo1@meet-os.com";

pub fn create_group_helper(client: &Client, name: &str, owner: usize) {
    let admin_email = "admin@meet-os.com";
    let res = client
        .post("/admin/create-group")
        .header(ContentType::Form)
        .body(params!([
            ("name", name),
            ("location", ""),
            ("description", "",),
            ("owner", &owner.to_string()),
        ]))
        .private_cookie(("meet-os", admin_email))
        .dispatch();

    assert_eq!(res.status(), Status::Ok);
}

pub fn login_helper(client: &Client, email: &str, password: &str) {
    let res = client
        .post("/login")
        .header(ContentType::Form)
        .body(params!([("email", email), ("password", password)]))
        .dispatch();
    assert_eq!(res.status(), Status::Ok);
}

pub fn register_and_verify_user(
    client: &Client,
    name: &str,
    email: &str,
    password: &str,
    email_folder: &PathBuf,
) {
    let res = client
        .post(format!("/register"))
        .header(ContentType::Form)
        .body(params!([
            ("name", name),
            ("email", email),
            ("password", password)
        ]))
        .dispatch();
    assert_eq!(res.status(), Status::Ok);

    let dir = email_folder
        .read_dir()
        .expect("read_dir call failed")
        .flatten()
        .collect::<Vec<_>>();
    println!("dir: {}", dir.len());

    // -2 because after the email with the code we also send a notification to the admin.
    let filename = format!("{}.txt", dir.len() - 2);
    let (uid, code) = read_code_from_email(email_folder, &filename, "verify-email");

    let res = client.get(format!("/verify-email/{uid}/{code}")).dispatch();
    assert_eq!(res.status(), Status::Ok);
}

pub fn add_event_helper(client: &Client, title: &str, date: &str, gid: &str, owner_email: String) {
    let res = client
        .post("/add-event")
        .header(ContentType::Form)
        .body(params!([
            ("gid", gid),
            ("offset", "-180"),
            ("title", title),
            ("location", "Virtual"),
            ("description", ""),
            ("date", date),
        ]))
        .private_cookie(("meet-os", owner_email))
        .dispatch();
    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    assert!(html.contains("Event added"));
    //rocket::info!("{html}");
}

pub fn setup_admin(client: &Client, email_folder: &PathBuf) {
    let name = "Site Manager";
    let email = "admin@meet-os.com";
    let password = "123456";

    register_and_verify_user(&client, name, email, password, &email_folder);
}

pub fn setup_foo(client: &Client, email_folder: &PathBuf) {
    register_and_verify_user(&client, "Foo Bar", FOO_EMAIL, "123foo", &email_folder);
}

pub fn setup_foo1(client: &Client, email_folder: &PathBuf) {
    register_and_verify_user(&client, "Foo 1", FOO1_EMAIL, "password1", &email_folder);
}

pub fn setup_many_users(client: &Client, email_folder: &PathBuf) {
    setup_admin(client, email_folder);
    setup_foo(client, email_folder);

    for ix in 1..3 {
        register_and_verify_user(
            &client,
            format!("Foo {ix}").as_str(),
            format!("foo{ix}@meet-os.com").as_str(),
            format!("password{ix}").as_str(),
            &email_folder,
        );
    }

    // Make sure the client is not logged in after the setup
    let res = client.get(format!("/logout")).dispatch();
    //assert_eq!(res.status(), Status::Ok);
    rocket::info!("--------------- finished setup_many_users ----------------")
}

pub fn setup_many(client: &Client, email_folder: &PathBuf) {
    setup_many_users(client, email_folder);

    create_group_helper(&client, "First Group", 2);
    create_group_helper(&client, "Second Group", 2);
    create_group_helper(&client, "Third Group", 3);
    add_event_helper(
        &client,
        "First event",
        "2030-01-01 10:10",
        "1",
        String::from(FOO_EMAIL),
    );

    add_event_helper(
        &client,
        "Second event",
        "2030-01-02 10:10",
        "1",
        String::from(FOO_EMAIL),
    );

    add_event_helper(
        &client,
        "Third event",
        "2030-01-03 10:10",
        "2",
        String::from(FOO_EMAIL),
    );

    // Make sure the client is not logged in after the setup
    let res = client.get(format!("/logout")).dispatch();
    //assert_eq!(res.status(), Status::Ok);
    rocket::info!("--------------- finished setup_many ----------------")
}
