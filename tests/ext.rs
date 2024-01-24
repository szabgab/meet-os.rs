use fork::{fork, Fork};
use nix::sys::signal;
use nix::unistd::Pid;
use scraper::{Html, Selector};
use std::os::unix::process::CommandExt;
use std::process::Command;

fn compile() {
    let _result = Command::new("cargo")
        .args(["build"])
        .output()
        .expect("command failed to start");
}

fn check_html(document: &Html, tag: &str, text: &str) {
    let selector = Selector::parse(tag).unwrap();
    assert_eq!(
        document.select(&selector).next().unwrap().inner_html(),
        text
    );
}

#[test]
fn external() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let port = "8001";
    println!("tmp_dir: {:?}", tmp_dir);
    std::env::set_var("DATABASE_PATH", tmp_dir.path().join("db"));
    std::env::set_var("EMAIL_FILE", tmp_dir.path().join("email.txt"));
    std::env::set_var("ROCKET_PORT", port);
    compile();

    match fork() {
        Ok(Fork::Parent(child)) => {
            println!("Child PID: {}", child);
            std::thread::sleep(std::time::Duration::from_secs(1));
            match reqwest::blocking::get(format!("http://localhost:{port}/")) {
                Ok(res) => {
                    assert_eq!(res.status(), 200);
                    match res.text() {
                        Ok(html) => {
                            let document = Html::parse_document(&html);
                            check_html(&document, "title", "Meet-OS");
                            check_html(&document, "h1", "Welcome to the Rust meeting server");

                            let selector = Selector::parse("li").unwrap();
                            let element = document.select(&selector).next().unwrap();
                            assert_eq!(
                                element.inner_html(),
                                r#"<a href="/event/1">Web development with Rocket</a>"#
                            );
                            let element = document.select(&selector).nth(1).unwrap();
                            assert_eq!(
                                element.inner_html(),
                                r#"<a href="/group/1">Rust Maven</a>"#
                            );

                            //println!("{}", html)
                        }
                        Err(err) => assert_eq!(err.to_string(), ""),
                    };
                }
                Err(err) => {
                    assert_eq!(err.to_string(), "");
                }
            };

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
