use fork::{fork, Fork};
use nix::sys::signal;
use nix::unistd::Pid;
use std::net::TcpListener;
use std::os::unix::process::CommandExt;
use std::process::Command;
use scraper::{Html, Selector};


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

pub fn run_external(func: fn(&str)) {
    let tmp_dir = tempfile::tempdir().unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port().to_string();
    drop(listener);
    println!("port: {port}");
    println!("tmp_dir: {:?}", tmp_dir);
    std::env::set_var("ROCKET_CONFIG", "Debug.toml");
    std::env::set_var("DATABASE_PATH", tmp_dir.path().join("db"));
    std::env::set_var("EMAIL_FILE", tmp_dir.path().join("email.txt"));
    std::env::set_var("ROCKET_PORT", &port);
    compile();

    match fork() {
        Ok(Fork::Parent(child)) => {
            println!("Child PID: {}", child);
            std::thread::sleep(std::time::Duration::from_secs(1));

            func(&port);

            signal::kill(Pid::from_raw(child), signal::Signal::SIGTERM).unwrap();

            println!("end of tests, shutting down the server")
        }
        Ok(Fork::Child) => {
            println!("Starting the web server in the child process");
            let _result = Command::new("./target/debug/meetings").exec();
        }
        Err(_) => println!("Fork failed"),
    }
}
