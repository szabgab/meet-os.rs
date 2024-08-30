#![allow(unused_macros, unused_imports)]

use regex::Regex;
use std::path::PathBuf;

use rocket::http::{ContentType, Status};
use rocket::local::blocking::Client;

use utilities::{check_html, read_code_from_email};

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
