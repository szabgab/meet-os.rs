use fork::{fork, Fork};
use nix::sys::signal;
use nix::unistd::Pid;
use std::{net::TcpListener, path::PathBuf};
use std::os::unix::process::CommandExt;
use std::process::Command;
use scraper::{Html, Selector};
use regex::Regex;

fn compile() {
    let _result = Command::new("cargo")
        .args(["build"])
        .output()
        .expect("command failed to start");
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

pub fn check_html_list(html: &str, tag: &str, text: Vec<&str>) {
    let document = Html::parse_document(html);
    let selector = Selector::parse(tag).unwrap();

    let element = document.select(&selector).next().unwrap();
    assert_eq!(element.inner_html(), text[0]);
    for ix in 1..text.len() {
        let element = document.select(&selector).nth(ix).unwrap();
        assert_eq!(element.inner_html(), text[ix]);
    }
}

pub fn run_external(func: fn(&str, std::path::PathBuf)) {
    let tmp_dir = tempfile::tempdir().unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port().to_string();
    drop(listener);
    println!("port: {port}");
    println!("tmp_dir: {:?}", tmp_dir);
    let email_folder = tmp_dir.path().join("emails");

    let rocket_toml = std::fs::read_to_string("Rocket.skeleton.toml").unwrap();
    let db_name = format!("test-name-{}", rand::random::<f64>());
    let db_namespace = format!("test-namespace-{}", rand::random::<f64>());
    let rocket_toml = rocket_toml.replace("meet-os-local-db", &db_name);
    let rocket_toml = rocket_toml.replace("meet-os-local-ns", &db_namespace);
    let rocket_toml = rocket_toml.replace("Sendgrid | Folder", "Folder");
    let rocket_toml = rocket_toml.replace("/path/to/email_folder", email_folder.to_str().unwrap());
    let rocket_toml = rocket_toml.replace("8001",&port);
    println!("{rocket_toml}");

    let rocket_toml_path = tmp_dir.path().join("Rocket.toml");
    std::fs::write(&rocket_toml_path, rocket_toml).unwrap();

    compile();

    match fork() {
        Ok(Fork::Parent(child)) => {
            println!("Child PID: {}", child);
            std::thread::sleep(std::time::Duration::from_secs(1));

            func(&port, email_folder);

            signal::kill(Pid::from_raw(child), signal::Signal::SIGTERM).unwrap();

            println!("end of tests, shutting down the server")
        }
        Ok(Fork::Child) => {
            println!("Starting the web server in the child process");
            let _result = Command::new("./target/debug/meetings")
                .env("ROCKET_CONFIG", rocket_toml_path)
                .exec();
        }
        Err(_) => println!("Fork failed"),
    }
}


pub fn check_profile_page(client: &reqwest::blocking::Client, url: &str, cookie_str: &str, h1: &str) {
    let res = client
        .get(format!("{url}/profile"))
        .header("Cookie", format!("meet-os={cookie_str}"))
        .send()
        .unwrap();
    assert_eq!(res.status(), 200);
    let html = res.text().unwrap();

    if h1.is_empty() {
        check_html(&html, "title", "Not logged in");
        assert!(html.contains("It seems you are not logged in"));
    } else {
        check_html(&html, "title", "Profile");
        check_html(&html, "h1", h1);
    }
}

pub fn register_user_helper(client: &reqwest::blocking::Client, url: &str, name: &str, email: &str, password: &str, email_folder: &PathBuf) -> String {
    let res = client
    .post(format!("{url}/register"))
    .form(&[("name", name), ("email", email), ("password", password)])
    .send()
    .unwrap();
    assert_eq!(res.status(), 200);

    let dir = email_folder
    .read_dir()
    .expect("read_dir call failed")
    .flatten()
    .collect::<Vec<_>>();
    println!("dir: {}", dir.len());

     // -2 because after the email with the code we also send a notification to the admin.
    let filename = format!("{}.txt", dir.len()-2);
    let (uid, code) = read_code_from_email(email_folder, &filename);

    let res = client
    .get(format!("{url}/verify-email/{uid}/{code}"))
    .send()
    .unwrap();
    assert_eq!(res.status(), 200);
    let cookie_str = extract_cookie(&res);
    return cookie_str;
}

pub fn login_helper(client: &reqwest::blocking::Client, url: &str, email: &str, password: &str) -> String {
    let res = client
    .post(format!("{url}/login"))
    .form(&[("email", email), ("password", password)])
    .send()
    .unwrap();
    assert_eq!(res.status(), 200);

    let cookie_str = extract_cookie(&res);
    cookie_str
}


pub fn read_code_from_email(email_folder: &std::path::PathBuf, filename: &str) -> (usize, String) {
    let email_file = email_folder.join(filename);
    let email_content = std::fs::read_to_string(email_file).unwrap();
    // https://meet-os.com/verify-email/3/c0514ec6-c51e-4376-ae8e-df82ef79bcef
    let re = Regex::new("http://localhost:[0-9]+/verify-email/([0-9]+)/([a-z0-9-]+)").unwrap();

    //println!("email content: {email_content}");
    let (uid, code) = match re.captures(&email_content) {
        Some(value) => (
            value[1].parse::<usize>().unwrap(),
            value[2].to_owned()
        ),
        None => panic!("Code not find in email: {email_content}"),
    };
    println!("extract uid: {uid} code: {code} from email");

    (uid, code)
}

pub fn extract_cookie(res: &reqwest::blocking::Response) -> String {
    let cookie = res.headers().get("set-cookie").unwrap().to_str().unwrap();
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
