#![allow(unused_macros, unused_imports)]

use regex::Regex;
use std::path::PathBuf;

use rocket::http::{ContentType, Status};
use rocket::local::blocking::{Client, LocalResponse};
use scraper::{Html, Selector};

macro_rules! params {
    ($params:expr) => {
        $params
            .into_iter()
            .map(|pair| format!("{}={}", pair.0, pair.1))
            .collect::<Vec<_>>()
            .join("&")
    };
}
pub(crate) use params;

use crate::test_helpers::ADMIN_EMAIL;

pub fn run_inprocess(func: fn(std::path::PathBuf, Client)) {
    use rocket::config::Config;

    let tmp_dir = tempfile::tempdir().unwrap();
    println!("tmp_dir: {:?}", tmp_dir);
    let email_folder = tmp_dir.path().join("emails");
    let db_name = format!("test-name-{}", rand::random::<f64>());
    let db_namespace = format!("test-namespace-{}", rand::random::<f64>());

    let provider = Config::figment()
        .merge(("database_namespace", &db_namespace))
        .merge(("database_name", &db_name))
        .merge(("email", "Folder"))
        .merge(("email_folder", email_folder.to_str().unwrap()))
        .merge(("admins", [ADMIN_EMAIL]));

    let app = super::rocket().configure(provider);
    let client = Client::tracked(app).unwrap();

    func(email_folder, client);
}

pub fn check_profile_by_guest(client: &Client) {
    let res = client.get("/profile").dispatch();
    check_not_logged_in(res);
}

macro_rules! check_profile_by_user {
    ($client: expr, $email: expr, $h1: expr) => {{
        let res = $client
            .get("/profile")
            .private_cookie(("meet-os", $email.to_owned()))
            .dispatch();

        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();

        check_html!(&html, "title", "Profile");
        check_html!(&html, "h1", $h1);
    }};
}
pub(crate) use check_profile_by_user;

pub fn check_guest_menu(html: &str) {
    assert!(!html.contains(r#"<a href="/admin" class="navbar-item">Admin</a>"#));

    assert!(html.contains(r#"<a href="/register" class="navbar-item">Register</a>"#));
    assert!(html.contains(r#"<a href="/login" class="navbar-item">Login</a>"#));

    assert!(!html.contains(r#"<a href="/profile" class="navbar-item">Profile"#));
    assert!(!html.contains(r#"<a href="/logout" class="navbar-item">Logout</a>"#));
}

fn check_logged_in_menu(html: &str) {
    assert!(!html.contains(r#"<a href="/register" class="navbar-item">Register</a>"#));
    assert!(!html.contains(r#"<a href="/login" class="navbar-item">Login</a>"#));

    assert!(html.contains(r#"<a href="/profile" class="navbar-item">Profile"#));
    assert!(html.contains(r#"<a href="/logout" class="navbar-item">Logout</a>"#));
}

pub fn check_admin_menu(html: &str) {
    check_logged_in_menu(html);
    assert!(html.contains(r#"<a href="/admin" class="navbar-item">Admin</a>"#));
}

pub fn check_user_menu(html: &str) {
    check_logged_in_menu(html);
    assert!(!html.contains(r#"<a href="/admin" class="navbar-item">Admin</a>"#));
}

macro_rules! check_html {
    ($html: expr, $selectors: expr, $text: expr) => {{
        let document = scraper::Html::parse_document($html);
        let selector = scraper::Selector::parse($selectors).unwrap();
        assert_eq!(
            &document.select(&selector).next().unwrap().inner_html(),
            $text
        );
    }};
}
pub(crate) use check_html;

pub fn check_not_logged_in(res: LocalResponse) {
    assert_eq!(res.status(), Status::Unauthorized);
    let html = res.into_string().unwrap();
    check_html!(&html, "title", "Not logged in");
    check_html!(&html, "h1", "Not logged in");
    check_html!(&html, "#message", "You are not logged in");
    check_guest_menu(&html);
}

pub fn check_unauthorized(res: LocalResponse) {
    assert_eq!(res.status(), Status::Forbidden);
    let html = res.into_string().unwrap();
    check_html!(&html, "title", "Unauthorized");
    check_html!(&html, "h1", "Unauthorized");
    check_html!(
        &html,
        "#message",
        "You don't have the rights to access this page."
    );
}

pub fn check_unprocessable(res: LocalResponse) {
    assert_eq!(res.status(), Status::UnprocessableEntity);
    let html = res.into_string().unwrap();
    check_html!(&html, "title", "422 Unprocessable Entity");
    check_html!(&html, "h1", "422 Unprocessable Entity");
    assert!(html.contains(
        "The request was well-formed but was unable to be followed due to semantic errors."
    ));
}

pub fn check_not_the_owner(res: LocalResponse) {
    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    check_html!(&html, "title", "Not the owner");
    check_html!(&html, "h1", "Not the owner");
    check_html!(
        &html,
        "#message",
        r#"You are not the owner of the group <b>1</b>"#
    );
}

// check_html!_list(
//     &html,
//     "li",
//     vec![
//         r#"<a href="/event/1">Web development with Rocket</a>"#,
//         r#"<a href="/group/1">Rust Maven</a>"#,
//     ],
// );

// pub fn check_html!_list(html: &str, tag: &str, text: Vec<&str>) {
//     let document = Html::parse_document(html);
//     let selector = Selector::parse(tag).unwrap();

//     let element = document.select(&selector).next().unwrap();
//     assert_eq!(element.inner_html(), text[0]);
//     for ix in 1..text.len() {
//         let element = document.select(&selector).nth(ix).unwrap();
//         assert_eq!(element.inner_html(), text[ix]);
//     }
// }

pub fn read_code_from_email(
    email_folder: &std::path::PathBuf,
    filename: &str,
    prefix: &str,
) -> (usize, String) {
    let email_file = email_folder.join(filename);
    let email_content = std::fs::read_to_string(email_file).unwrap();
    // https://meet-os.com/verify-email/3/c0514ec6-c51e-4376-ae8e-df82ef79bcef
    let regex_string = format!("http://localhost:[0-9]+/{prefix}/([0-9]+)/([a-z0-9-]+)");
    let re = Regex::new(&regex_string).unwrap();

    //println!("email content: {email_content}");
    let (uid, code) = match re.captures(&email_content) {
        Some(value) => (value[1].parse::<usize>().unwrap(), value[2].to_owned()),
        None => panic!("Code not find in email: {email_content}"),
    };
    println!("extract uid: {uid} code: {code} from email");

    (uid, code)
}
