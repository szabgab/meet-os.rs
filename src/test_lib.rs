#![allow(unused_macros, unused_imports)]

use rocket::http::Status;
use rocket::local::blocking::Client;

use utilities::check_html;

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
        .merge(("email_folder", email_folder.to_str().unwrap()));

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
