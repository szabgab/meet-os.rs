#![allow(unused_macros, unused_imports)]

use regex::Regex;
use std::path::PathBuf;

use rocket::http::{ContentType, Status};
use rocket::local::blocking::Client;
use scraper::{Html, Selector};

use crate::test_lib::{extract_cookie, params, read_code_from_email};

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

pub fn register_user_helper(
    client: &Client,
    name: &str,
    email: &str,
    password: &str,
    email_folder: &PathBuf,
) -> String {
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
    let (uid, code) = read_code_from_email(email_folder, &filename);

    let res = client.get(format!("/verify-email/{uid}/{code}")).dispatch();
    assert_eq!(res.status(), Status::Ok);
    let cookie_str = extract_cookie(&res);
    return cookie_str;
}

pub fn add_event_helper(client: &Client, title: &str, date: &str, owner_email: String) {
    let res = client
        .post("/add-event")
        .header(ContentType::Form)
        .body(params!([
            ("gid", "1"),
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

pub fn setup_many_users(client: &Client, email_folder: &PathBuf) {
    let name = "Site Manager";
    let email = "admin@meet-os.com";
    let password = "123456";

    register_user_helper(&client, name, email, password, &email_folder);

    register_user_helper(
        &client,
        "Foo Bar",
        "foo@meet-os.com",
        "123foo",
        &email_folder,
    );

    for ix in 1..3 {
        register_user_helper(
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
    add_event_helper(
        &client,
        "First event",
        "2030-01-01 10:10",
        String::from("foo@meet-os.com"),
    );

    // Make sure the client is not logged in after the setup
    let res = client.get(format!("/logout")).dispatch();
    //assert_eq!(res.status(), Status::Ok);
    rocket::info!("--------------- finished setup_many ----------------")
}
