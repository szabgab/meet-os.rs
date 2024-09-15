use std::net::TcpListener;
use std::os::unix::process::CommandExt;
use std::process::Command;

use fork::{fork, Fork};
use nix::sys::signal;
use nix::unistd::Pid;
use scraper::{Html, Selector};

#[test]
fn check_empty_home() {
    // We only keep this test to showcase the external testing we used for a while.
    // tarpaulin - does not like it so we only run it when the user sets RUN_EXTERNAL=1
    // e.g. in the CI
    match std::env::var("RUN_EXTERNAL") {
        Ok(run) => {
            if run == "" {
                println!("RUN_EXTERNAL='{run}' environment variable is not set. Skipping.");
                return;
            }
        }
        Err(_) => {
            println!("RUN_EXTERNAL environment variable is not set. Skipping.");
            return;
        }
    }
    run_external(|port, _email_folder| {
        let url = format!("http://localhost:{port}");
        let res = reqwest::blocking::get(format!("{url}/")).unwrap();
        assert_eq!(res.status(), 200);

        let html = res.text().unwrap();
        check_html(&html, "title", "Meet-OS");
        check_html(&html, "h1", "Welcome to the Meet-OS meeting server");
        assert!(!html.contains("<h2>Events</h2>"));
        assert!(!html.contains("<h2>Groups</h2>"));
        check_guest_menu(&html);
    });
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
    let db_namespace = "test-namespace-for-meet-os";
    let rocket_toml = rocket_toml.replace("meet-os-local-db", &db_name);
    let rocket_toml = rocket_toml.replace("meet-os-local-ns", &db_namespace);
    let rocket_toml = rocket_toml.replace("Sendgrid | Folder", "Folder");
    let rocket_toml = rocket_toml.replace("/path/to/email_folder", email_folder.to_str().unwrap());
    let rocket_toml = rocket_toml.replace("8001", &port);
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

fn compile() {
    let _result = Command::new("cargo")
        .args(["build"])
        .output()
        .expect("command failed to start");
}

pub fn check_html(html: &str, tag: &str, text: &str) {
    let document = Html::parse_document(html);
    let selector = Selector::parse(tag).unwrap();
    assert_eq!(
        document.select(&selector).next().unwrap().inner_html(),
        text
    );
}

pub fn check_guest_menu(html: &str) {
    assert!(!html.contains(r#"<a href="/admin" class="navbar-item">Admin</a>"#));

    assert!(html.contains(r#"<a href="/register" class="navbar-item">Register</a>"#));
    assert!(html.contains(r#"<a href="/login" class="navbar-item">Login</a>"#));

    assert!(!html.contains(r#"<a href="/profile" class="navbar-item">Profile"#));
    assert!(!html.contains(r#"<a href="/logout" class="navbar-item">Logout</a>"#));
}
