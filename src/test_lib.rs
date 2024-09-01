#![allow(unused_macros, unused_imports)]

use regex::Regex;
use std::path::PathBuf;

use rocket::http::{ContentType, Status};
use rocket::local::blocking::Client;
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
        .merge(("admins", ["admin@meet-os.com"]));

    let app = super::rocket().configure(provider);
    let client = Client::tracked(app).unwrap();

    func(email_folder, client);
}

pub fn check_profile_page_in_process(client: &Client, email: &str, h1: &str) {
    let res = client
        .get("/profile")
        .private_cookie(("meet-os", email.to_owned()))
        .dispatch();

    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();

    if h1.is_empty() {
        check_html(&html, "title", "Not logged in");
        assert!(html.contains("It seems you are not logged in"));
    } else {
        check_html(&html, "title", "Profile");
        check_html(&html, "h1", h1);
    }
}

pub fn extract_cookie(res: &rocket::local::blocking::LocalResponse) -> String {
    let cookie = res.headers().get_one("set-cookie").unwrap();
    println!("cookie: {cookie}");
    assert!(cookie.contains("meet-os="));
    let re = Regex::new("meet-os=([^;]+);").unwrap();
    let cookie_str = match re.captures(cookie) {
        Some(value) => value[1].to_owned(),
        None => panic!("Code not found cookie"),
    };

    println!("cookie_str: {cookie_str}");

    cookie_str
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

pub fn setup_many(client: &Client, email_folder: &PathBuf) {
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

pub fn check_html(html: &str, tag: &str, text: &str) {
    let document = Html::parse_document(html);
    let selector = Selector::parse(tag).unwrap();
    assert_eq!(
        document.select(&selector).next().unwrap().inner_html(),
        text
    );
}

// check_html_list(
//     &html,
//     "li",
//     vec![
//         r#"<a href="/event/1">Web development with Rocket</a>"#,
//         r#"<a href="/group/1">Rust Maven</a>"#,
//     ],
// );

// pub fn check_html_list(html: &str, tag: &str, text: Vec<&str>) {
//     let document = Html::parse_document(html);
//     let selector = Selector::parse(tag).unwrap();

//     let element = document.select(&selector).next().unwrap();
//     assert_eq!(element.inner_html(), text[0]);
//     for ix in 1..text.len() {
//         let element = document.select(&selector).nth(ix).unwrap();
//         assert_eq!(element.inner_html(), text[ix]);
//     }
// }

pub fn read_code_from_email(email_folder: &std::path::PathBuf, filename: &str) -> (usize, String) {
    let email_file = email_folder.join(filename);
    let email_content = std::fs::read_to_string(email_file).unwrap();
    // https://meet-os.com/verify-email/3/c0514ec6-c51e-4376-ae8e-df82ef79bcef
    let re = Regex::new("http://localhost:[0-9]+/verify-email/([0-9]+)/([a-z0-9-]+)").unwrap();

    //println!("email content: {email_content}");
    let (uid, code) = match re.captures(&email_content) {
        Some(value) => (value[1].parse::<usize>().unwrap(), value[2].to_owned()),
        None => panic!("Code not find in email: {email_content}"),
    };
    println!("extract uid: {uid} code: {code} from email");

    (uid, code)
}

pub fn read_code_from_email_any(
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
