use fork::{fork, Fork};
use nix::sys::signal;
use nix::unistd::Pid;
use regex::Regex;
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

fn run_external(func: fn(&str)) {
    let tmp_dir = tempfile::tempdir().unwrap();
    let port = "8001";
    println!("tmp_dir: {:?}", tmp_dir);
    // std::env::set_var("ROCKET_CONFIG", "Debug.toml");
    std::env::set_var("DATABASE_PATH", tmp_dir.path().join("db"));
    std::env::set_var("EMAIL_FILE", tmp_dir.path().join("email.txt"));
    std::env::set_var("ROCKET_PORT", port);
    compile();

    match fork() {
        Ok(Fork::Parent(child)) => {
            println!("Child PID: {}", child);
            std::thread::sleep(std::time::Duration::from_secs(1));

            func(port);

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

#[test]
fn home() {
    run_external(|port| {
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
                        assert_eq!(element.inner_html(), r#"<a href="/group/1">Rust Maven</a>"#);

                        //println!("{}", html)
                    }
                    Err(err) => assert_eq!(err.to_string(), ""),
                };
            }
            Err(err) => {
                assert_eq!(err.to_string(), "");
            }
        };
    });
}

#[test]
fn register_user() {
    run_external(|port| {
        let client = reqwest::blocking::Client::new();
        let res = client
            .post(format!("http://localhost:{port}/register"))
            .form(&[("name", "Foo Bar"), ("email", "foo@meet-os.com")])
            .send()
            .unwrap();
        assert_eq!(res.status(), 200);
        //println!("{:#?}", res.headers());
        assert!(res.headers().get("set-cookie").is_none());

        let html = res.text().unwrap();
        let document = Html::parse_document(&html);
        check_html(&document, "title", "We sent you an email");
        assert!(html.contains(r#"We sent you an email to <b>foo@meet-os.com</b> Please check your inbox and verify your email address."#));

        let email_file = std::env::var("EMAIL_FILE").unwrap();

        let email = std::fs::read_to_string(email_file).unwrap();
        // https://meet-os.com/verify/register/c0514ec6-c51e-4376-ae8e-df82ef79bcef
        let re = Regex::new(r"http://localhost:8000/verify/register/([a-z0-9-]+)").unwrap();

        log::info!("email: {email}");
        let code = match re.captures(&email) {
            Some(value) => value[1].to_owned(),
            None => panic!("Code not found in email"),
        };

        //assert_eq!(code, res.code);
        log::info!("code: {code}");
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Access the profile without a cookie
        let res = client
            .get(format!("http://localhost:{port}/profile"))
            .send()
            .unwrap();
        assert_eq!(res.status(), 200);
        let html = res.text().unwrap();
        //assert_eq!(html, "");
        let document = Html::parse_document(&html);
        check_html(&document, "title", "Missing cookie");
        assert!(html.contains("It seems you are not logged in"));
        std::thread::sleep(std::time::Duration::from_millis(500));

        // TODO. shall we access the database directly and check the data there too?
        //     // Without dropping the client here we get an error on the next line
        //     // value: Db(Tx("IO error: lock hold by current process, acquire time 1705858854 acquiring thread 58266: /tmp/.tmpVlNPFx/db/LOCK: No locks available"))
        //     let res = tokio_test::block_on(get_user_by_email(&db, "foo@meet-os.com"))
        //         .unwrap()
        //         .unwrap();
        //     assert_eq!(res.email, "foo@meet-os.com");
        //     assert_eq!(res.name, "Foo Bar");
        //     assert!(!res.verified);
        //     assert_eq!(res.process, "register");
        //     // date? code?

        //     // Verify the email
        let url = format!("http://localhost:{port}/");
        let res = client
            .get(format!("{url}/verify/register/{code}"))
            .send()
            .unwrap();
        assert_eq!(res.status(), 200);
        let cookie = res.headers().get("set-cookie").unwrap().to_str().unwrap();
        println!("cookie: {cookie}");
        assert!(cookie.contains("meet-os="));
        let re = Regex::new(r"meet-os=([^;]+);").unwrap();
        let cookie_str = match re.captures(&cookie) {
            Some(value) => value[1].to_owned(),
            None => panic!("Code not found cookie"),
        };
        println!("cookie_str: {cookie_str}");

        let html = res.text().unwrap();
        let document = Html::parse_document(&html);
        check_html(&document, "title", "Thank you for registering");
        assert!(html.contains("Your email was verified."));
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Access the profile with the cookie
        let res = client
            .get(format!("{url}/profile"))
            .header("Cookie", format!("meet-os={cookie_str}"))
            .send()
            .unwrap();
        assert_eq!(res.status(), 200);
        let html = res.text().unwrap();
        //assert_eq!(html, "x");
        let document = Html::parse_document(&html);
        check_html(&document, "title", "Profile");
        assert!(html.contains(r#"<h1 class="title is-3">Foo Bar</h1>"#));
    });
}
