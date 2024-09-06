#![allow(unused_macros, unused_imports)]

use regex::Regex;
use std::path::PathBuf;

use rocket::http::{ContentType, Status};
use rocket::local::blocking::Client;
use scraper::{Html, Selector};

use crate::test_lib::{params, read_code_from_email};

pub const OWNER_EMAIL: &str = "owner@meet-os.com";
pub const OWNER_PW: &str = "123foo";
pub const OWNER_NAME: &str = "Don Quijote de la Mancha";

pub const USER_EMAIL: &str = "user@meet-os.com";
pub const USER_PW: &str = "password1";
pub const USER_NAME: &str = "Sancho Panza";

pub const UNVERIFIED_EMAIL: &str = "unverified@meet-os.com";
pub const UNVERIFIED_PW: &str = "qwerty";
pub const UNVERIFIED_NAME: &str = "Halfway Through";

pub const ADMIN_EMAIL: &str = "admin@meet-os.com";
pub const ADMIN_PW: &str = "123456";
pub const ADMIN_NAME: &str = "Site Manager";

pub fn register_user(client: &Client, name: &str, email: &str, password: &str) {
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
}

fn verify_email(email_folder: &PathBuf, client: &Client) {
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

pub fn register_and_verify_user(
    client: &Client,
    name: &str,
    email: &str,
    password: &str,
    email_folder: &PathBuf,
) {
    register_user(client, name, email, password);

    verify_email(email_folder, client);
}

pub fn setup_admin(client: &Client, email_folder: &PathBuf) {
    register_and_verify_user(&client, ADMIN_NAME, ADMIN_EMAIL, ADMIN_PW, &email_folder);
}

pub fn setup_owner(client: &Client, email_folder: &PathBuf) {
    register_and_verify_user(&client, OWNER_NAME, OWNER_EMAIL, OWNER_PW, &email_folder);
}

pub fn setup_user(client: &Client, email_folder: &PathBuf) {
    register_and_verify_user(&client, USER_NAME, USER_EMAIL, USER_PW, &email_folder);
}

pub fn setup_unverified_user(client: &Client, email_folder: &PathBuf) {
    register_user(&client, UNVERIFIED_NAME, UNVERIFIED_EMAIL, UNVERIFIED_PW);
}

pub fn setup_many_users(client: &Client, email_folder: &PathBuf) {
    setup_admin(client, email_folder);
    setup_owner(client, email_folder);
    setup_user(client, email_folder);

    for ix in 2..3 {
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

pub fn logout(client: &Client) {
    client.get(format!("/logout")).dispatch();
}

pub fn login_admin(client: &Client) {
    login_helper(client, ADMIN_EMAIL, ADMIN_PW);
}

pub fn login_owner(client: &Client) {
    login_helper(client, OWNER_EMAIL, OWNER_PW);
}

fn login_helper(client: &Client, email: &str, password: &str) {
    let res = client
        .post("/login")
        .header(ContentType::Form)
        .body(params!([("email", email), ("password", password)]))
        .dispatch();
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

pub fn create_group_helper(client: &Client, name: &str, owner: usize) {
    let res = client
        .post("/admin/create-group")
        .header(ContentType::Form)
        .body(params!([
            ("name", name),
            ("location", ""),
            ("description", "",),
            ("owner", &owner.to_string()),
        ]))
        .private_cookie(("meet-os", ADMIN_EMAIL))
        .dispatch();

    assert_eq!(res.status(), Status::Ok);
}

pub fn setup_event(client: &Client, eid: usize) {
    match eid {
        1 => add_event_helper(
            &client,
            "First event",
            "2030-01-01 10:10",
            "1",
            String::from(OWNER_EMAIL),
        ),
        2 => add_event_helper(
            &client,
            "Second event",
            "2030-01-02 10:10",
            "1",
            String::from(OWNER_EMAIL),
        ),
        3 => add_event_helper(
            &client,
            "Third event",
            "2030-01-03 10:10",
            "2",
            String::from(OWNER_EMAIL),
        ),

        _ => panic!("no such eid",),
    }
}

pub fn setup_many(client: &Client, email_folder: &PathBuf) {
    setup_many_users(client, email_folder);

    create_group_helper(&client, "First Group", 2);
    create_group_helper(&client, "Second Group", 2);
    create_group_helper(&client, "Third Group", 3);
    setup_event(client, 1);
    setup_event(client, 2);
    setup_event(client, 3);

    // Make sure the client is not logged in after the setup
    let res = client.get(format!("/logout")).dispatch();
    //assert_eq!(res.status(), Status::Ok);
    rocket::info!("--------------- finished setup_many ----------------")
}

pub fn setup_for_events(client: &Client, email_folder: &PathBuf) {
    setup_admin(&client, &email_folder);
    setup_owner(&client, &email_folder);
    setup_user(&client, &email_folder);
    create_group_helper(&client, "First Group", 2);
    setup_event(&client, 1);
    logout(&client);
}
