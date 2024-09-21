#![allow(unused_macros, unused_imports)]

use std::path::PathBuf;
use std::process::ExitStatus;

use regex::Regex;
use rocket::http::{ContentType, Status};
use rocket::local::blocking::{Client, LocalResponse};
use scraper::{Html, Selector};

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

pub const OTHER_NAME: &str = "Foo Alpha";
pub const OTHER_EMAIL: &str = "foo-alpha@meet-os.com";
pub const OTHER_PW: &str = "password1";

#[allow(dead_code)]
pub struct TestRunner {
    db_name: String,
    db_namespace: String,
    user_name: String,
    user_pw: String,
    tmp_dir: tempfile::TempDir,
    pub email_folder: PathBuf,
    pub client: Client,
}

impl TestRunner {
    pub fn new() -> Self {
        Self::from("")
    }

    pub fn from(filename: &str) -> Self {
        use rocket::config::Config;

        let tmp_dir = tempfile::tempdir().unwrap();
        println!("tmp_dir: {:?}", tmp_dir);
        let email_folder = tmp_dir.path().join("emails");
        let db_name = format!("test-name-{}", rand::random::<f64>());
        let db_namespace = String::from("test-namespace-for-meet-os");
        let user_name = String::from("root");
        let user_pw = String::from("root");
        println!("namespace: {db_namespace} database: {db_name}");

        if !filename.is_empty() {
            let path = format!("/external/tests/{filename}");

            let result = std::process::Command::new("/usr/bin/docker")
                .arg("exec")
                .arg("surrealdb")
                .arg("/surreal")
                .arg("import")
                .arg("-e")
                .arg("http://localhost:8000")
                .arg("--namespace")
                .arg(&db_namespace)
                .arg("--database")
                .arg(&db_name)
                .arg("--user")
                .arg(&user_name)
                .arg("--pass")
                .arg(&user_pw)
                .arg(&path)
                .output()
                .unwrap();

            println!("result.status: {}", result.status);
            println!("STDOUT: {:?}", std::str::from_utf8(&result.stdout));
            println!("STDERR: {:?}", std::str::from_utf8(&result.stderr));
            assert_eq!(result.status, ExitStatus::default(), "Importing test data");
        }

        let provider = Config::figment()
            .merge(("database_namespace", &db_namespace))
            .merge(("database_name", &db_name))
            .merge(("email", "Folder"))
            .merge(("email_folder", email_folder.to_str().unwrap()))
            .merge(("admins", [ADMIN_EMAIL]));

        let app = super::rocket().configure(provider);
        let client = Client::tracked(app).unwrap();

        Self {
            db_name,
            db_namespace,
            user_name,
            user_pw,
            tmp_dir,
            email_folder,
            client,
        }
    }

    pub fn setup_for_groups(&self) {
        setup_admin(&self.client, &self.email_folder);
        setup_owner(&self.client, &self.email_folder);
        setup_user(&self.client, &self.email_folder);
        create_group_helper(&self.client, "First Group", 2);
        logout(&self.client);
    }
}

impl Drop for TestRunner {
    fn drop(&mut self) {
        let tmp_dir = tempfile::tempdir_in("temp").unwrap();
        let filename = tmp_dir.path().join("remove.sql");
        println!("filename: {filename:?}");
        let dirname = filename
            .ancestors()
            .nth(1)
            .unwrap()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();
        let path_to_file = format!("/external/temp/{dirname}/remove.sql");
        println!("dirname: '{dirname}' '{path_to_file}'");
        let sql = format!("REMOVE DATABASE `{}`;", self.db_name);
        std::fs::write(&filename, sql).unwrap();

        let result = std::process::Command::new("/usr/bin/docker")
            .arg("exec")
            .arg("surrealdb")
            .arg("/surreal")
            .arg("import")
            .arg("-e")
            .arg("http://localhost:8000")
            .arg("--namespace")
            .arg(&self.db_namespace)
            .arg("--database")
            .arg(&self.db_name)
            .arg("--user")
            .arg(&self.user_name)
            .arg("--pass")
            .arg(&self.user_pw)
            .arg(path_to_file)
            .output()
            .unwrap();

        println!("result.status: {}", result.status);
        println!("STDOUT: {:?}", std::str::from_utf8(&result.stdout));
        println!("STDERR: {:?}", std::str::from_utf8(&result.stderr));
        assert_eq!(result.status, ExitStatus::default(), "Importing test data");
    }
}

pub fn clean_emails(email_folder: &std::path::PathBuf) {
    for entry in email_folder.read_dir().unwrap() {
        let entry = entry.unwrap();
        std::fs::remove_file(entry.path()).unwrap();
    }
}

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

macro_rules! check_profile_by_guest {
    ($client: expr) => {{
        let res = $client.get("/profile").dispatch();
        check_not_logged_in!(res);
    }};
}
pub(crate) use check_profile_by_guest;

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

macro_rules! check_guest_menu {
    ($html: expr) => {{
        assert!(!$html.contains(r#"<a href="/admin" class="navbar-item">Admin</a>"#));

        assert!($html.contains(r#"<a href="/register" class="navbar-item">Register</a>"#));
        assert!($html.contains(r#"<a href="/login" class="navbar-item">Login</a>"#));

        assert!(!$html.contains(r#"<a href="/profile" class="navbar-item">Profile"#));
        assert!(!$html.contains(r#"<a href="/logout" class="navbar-item">Logout</a>"#));
    }};
}
pub(crate) use check_guest_menu;

macro_rules! check_logged_in_menu {
    ($html: expr) => {{
        assert!(!$html.contains(r#"<a href="/register" class="navbar-item">Register</a>"#));
        assert!(!$html.contains(r#"<a href="/login" class="navbar-item">Login</a>"#));

        assert!($html.contains(r#"<a href="/profile" class="navbar-item">Profile"#));
        assert!($html.contains(r#"<a href="/logout" class="navbar-item">Logout</a>"#));
    }};
}
pub(crate) use check_logged_in_menu;

macro_rules! check_admin_menu {
    ($html: expr) => {
        use crate::test_lib::check_logged_in_menu;
        check_logged_in_menu!($html);
        assert!($html.contains(r#"<a href="/admin" class="navbar-item">Admin</a>"#));
    };
}
pub(crate) use check_admin_menu;

macro_rules! check_user_menu {
    ($html: expr) => {{
        use crate::test_lib::check_logged_in_menu;
        check_logged_in_menu!($html);
        assert!(!$html.contains(r#"<a href="/admin" class="navbar-item">Admin</a>"#));
    }};
}
pub(crate) use check_user_menu;

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

macro_rules! check_message {
    ($html: expr, $title: expr, $message: expr) => {{
        check_html!($html, "title", $title);
        check_html!($html, "h1", $title);
        check_html!($html, "#message", $message);
    }};
}
pub(crate) use check_message;

macro_rules! check_not_logged_in {
    ($res: expr) => {{
        assert_eq!($res.status(), Status::Unauthorized);
        let html = $res.into_string().unwrap();
        check_message!(&html, "Not logged in", "You are not logged in");
        check_guest_menu!(&html);
    }};
}
pub(crate) use check_not_logged_in;

macro_rules! check_unauthorized {
    ($res: expr) => {{
        assert_eq!($res.status(), Status::Forbidden);
        let html = $res.into_string().unwrap();
        check_message!(
            &html,
            "Unauthorized",
            "You don't have the rights to access this page."
        )
    }};
}
pub(crate) use check_unauthorized;

macro_rules! check_unprocessable {
    ($res: expr) => {{
        assert_eq!($res.status(), Status::UnprocessableEntity);
        let html = $res.into_string().unwrap();
        check_message!(
            &html,
            "422 Unprocessable Entity",
            "The request was well-formed but was unable to be followed due to semantic errors."
        );
    }};
}
pub(crate) use check_unprocessable;

macro_rules! check_not_the_owner {
    ($res: expr) => {{
        assert_eq!($res.status(), Status::Ok);
        let html = $res.into_string().unwrap();
        check_message!(
            &html,
            "Not the owner",
            r#"You are not the owner of the group <b>1</b>"#
        );
    }};
}
pub(crate) use check_not_the_owner;

macro_rules! check_only_guest {
    ($res: expr) => {{
        assert_eq!($res.status(), Status::Ok);
        let html = $res.into_string().unwrap();
        check_message!(&html, "Logged in", r#"Logged in users cannot access this page. Please, <a href="/logout">logout</a> and try again!"#);
        check_user_menu!(&html);
    }};
}
pub(crate) use check_only_guest;

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

pub fn register_user_helper(client: &Client, name: &str, email: &str, password: &str) {
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
    register_user_helper(client, name, email, password);

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
    register_user_helper(&client, UNVERIFIED_NAME, UNVERIFIED_EMAIL, UNVERIFIED_PW);
}

pub fn setup_many_users(client: &Client, email_folder: &PathBuf) {
    setup_admin(client, email_folder);
    setup_owner(client, email_folder);
    setup_user(client, email_folder);

    register_and_verify_user(&client, OTHER_NAME, OTHER_EMAIL, OTHER_PW, &email_folder);

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

// pub fn login_user(client: &Client) {
//     login_helper(client, USER_EMAIL, USER_PW);
// }

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

pub fn setup_all(client: &Client, email_folder: &PathBuf) {
    setup_many_users(client, email_folder);

    create_group_helper(&client, "First Group", 2);
    create_group_helper(&client, "Second Group", 2);
    create_group_helper(&client, "Third Group", 3);
    setup_event(client, 1);
    setup_event(client, 2);
    setup_event(client, 3);

    // Make sure the client is not logged in after the setup
    let res = client.get(format!("/logout")).dispatch();
    // The setup_many_users logged the user out already so the above might return an error
    // That's why we don't check if it is Status::Ok
    //assert_eq!(res.status(), Status::Ok);
    rocket::info!("--------------- finished setup_all ----------------")
}

pub fn setup_for_events(client: &Client, email_folder: &PathBuf) {
    setup_admin(&client, &email_folder);
    setup_owner(&client, &email_folder);
    setup_user(&client, &email_folder);
    create_group_helper(&client, "First Group", 2);
    setup_event(&client, 1);
    logout(&client);
}
