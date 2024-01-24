use fork::{fork, Fork};
use nix::sys::signal;
use nix::unistd::Pid;
use scraper::{Html, Selector};
use std::os::unix::process::CommandExt;
use std::process::Command;

#[test]
fn external() {
    let tmp_dir = tempfile::tempdir().unwrap();
    println!("tmp_dir: {:?}", tmp_dir);
    std::env::set_var("DATABASE_PATH", tmp_dir.path().join("db"));
    std::env::set_var("EMAIL_FILE", tmp_dir.path().join("email.txt"));
    std::env::set_var("ROCKET_PORT", "8001");

    let _result = Command::new("cargo")
        .args(["build"])
        .output()
        .expect("command failed to start");
    println!("---- after compile");

    match fork() {
        Ok(Fork::Parent(child)) => {
            println!("Child PID: {}", child);
            std::thread::sleep(std::time::Duration::from_secs(1));
            match reqwest::blocking::get("http://localhost:8001/") {
                Ok(res) => {
                    println!("status: {:?}", res.status());
                    //println!("server: {:?}", &res.headers()["server"]);
                    match res.text() {
                        Ok(html) => {
                            let document = Html::parse_document(&html);
                            let selector = Selector::parse("title").unwrap();
                            for element in document.select(&selector) {
                                assert_eq!(element.inner_html(), "Meet-OS")
                            }

                            let selector = Selector::parse("h1").unwrap();
                            for element in document.select(&selector) {
                                assert_eq!(
                                    element.inner_html(),
                                    "Welcome to the Rust meeting server"
                                )
                            }

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
                    println!("Error {}", err);
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
    // println!("======================");
    //     let res = match reqwest::blocking::get("http://localhost:8000/") {
    //         Ok(res) => res,
    //         Err(err) => {
    //             println!("Error {}", err);
    //             std::process::exit(1);
    //         }
    //     };

    // let client = Client::tracked(super::rocket()).unwrap();
    // let response = client.get("/privacy").dispatch();

    // assert_eq!(response.status(), Status::Ok);
    // let body = response.into_string().unwrap();
    // assert!(body.contains(r#"<title>Privacy Policy</title>"#));
}
