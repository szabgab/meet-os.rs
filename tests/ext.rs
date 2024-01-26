use regex::Regex;

use utilities::{check_html, check_html_list, run_external};

#[test]
fn home() {
    run_external(|port| {
        match reqwest::blocking::get(format!("http://localhost:{port}/")) {
            Ok(res) => {
                assert_eq!(res.status(), 200);
                match res.text() {
                    Ok(html) => {
                        check_html(&html, "title", "Meet-OS");
                        check_html(&html, "h1", "Welcome to the Rust meeting server");
                        check_html_list(
                            &html,
                            "li",
                            vec![
                                r#"<a href="/event/1">Web development with Rocket</a>"#,
                                r#"<a href="/group/1">Rust Maven</a>"#,
                            ],
                        );
                        check_html_list(&html, "h2", vec!["Events", "Groups"]);

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
        check_html(&html, "title", "We sent you an email");
        assert!(html.contains(r#"We sent you an email to <b>foo@meet-os.com</b> Please check your inbox and verify your email address."#));

        let email_file = std::env::var("EMAIL_FILE").unwrap();

        let email_content = std::fs::read_to_string(email_file).unwrap();
        // https://meet-os.com/verify/register/c0514ec6-c51e-4376-ae8e-df82ef79bcef
        let re = Regex::new(r"http://localhost:8000/verify/register/([a-z0-9-]+)").unwrap();

        println!("email content: {email_content}");
        let code = match re.captures(&email_content) {
            Some(value) => value[1].to_owned(),
            None => panic!("Code not found in email: {email_content}"),
        };

        //assert_eq!(code, res.code);
        println!("code: {code}");
        //        std::thread::sleep(std::time::Duration::from_millis(500));

        // Access the profile without a cookie
        let res = client
            .get(format!("http://localhost:{port}/profile"))
            .send()
            .unwrap();
        assert_eq!(res.status(), 200);
        let html = res.text().unwrap();
        //assert_eq!(html, "");
        check_html(&html, "title", "Missing cookie");
        assert!(html.contains("It seems you are not logged in"));
        //        std::thread::sleep(std::time::Duration::from_millis(500));

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
        check_html(&html, "title", "Thank you for registering");
        assert!(html.contains("Your email was verified."));
        //        std::thread::sleep(std::time::Duration::from_millis(500));

        // Access the profile with the cookie
        let res = client
            .get(format!("{url}/profile"))
            .header("Cookie", format!("meet-os={cookie_str}"))
            .send()
            .unwrap();
        assert_eq!(res.status(), 200);
        let html = res.text().unwrap();
        //assert_eq!(html, "x");
        check_html(&html, "title", "Profile");
        check_html(&html, "h1", "Foo Bar");
    });
}

#[test]
fn verify_with_non_existent_code() {
    run_external(|port| {
        let client = reqwest::blocking::Client::new();
        let res = client
            .get(format!("http://localhost:{port}/verify/register/abc"))
            .send()
            .unwrap();
        assert_eq!(res.status(), 200);
        let html = res.text().unwrap();
        //assert_eq!(html, "");
        //assert!(html.contains("<title>Thank you for registering</title>"));
        assert!(html.contains("Invalid code <b>abc</b>"));
    });
}

#[test]
fn duplicate_email() {
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
        check_html(&html, "title", "We sent you an email");
        assert!(html.contains(r#"We sent you an email to <b>foo@meet-os.com</b> Please check your inbox and verify your email address."#));
        //        std::thread::sleep(std::time::Duration::from_millis(500));

        let res = client
            .post(format!("http://localhost:{port}/register"))
            .form(&[("name", "Foo Bar"), ("email", "foo@meet-os.com")])
            .send()
            .unwrap();
        assert_eq!(res.status(), 200);
        //println!("{:#?}", res.headers());
        assert!(res.headers().get("set-cookie").is_none());
        let html = res.text().unwrap();
        check_html(&html, "title", "Registration failed");
    });
}

#[test]
fn login() {
    run_external(|port| {
        // register new user
        let client = reqwest::blocking::Client::new();
        let res = client
            .post(format!("http://localhost:{port}/register"))
            .form(&[("name", "Foo Bar"), ("email", "foo@meet-os.com")])
            .send()
            .unwrap();
        assert_eq!(res.status(), 200);
        //println!("{:#?}", res.headers());
        assert!(res.headers().get("set-cookie").is_none());
        //        std::thread::sleep(std::time::Duration::from_millis(500));

        let res = client
            .post(format!("http://localhost:{port}/login"))
            .form(&[("email", "foo@meet-os.com")])
            .send()
            .unwrap();
        assert_eq!(res.status(), 200);
        assert!(res.headers().get("set-cookie").is_none());
        let html = res.text().unwrap();
        check_html(&html, "title", "We sent you an email");
        assert!(html.contains("We sent you an email to <b>foo@meet-os.com</b>"));

        // TODO: get the user from the database and check if there is a code and if the process is "login"

        // get the email and extract the code from the link
        let email_file = std::env::var("EMAIL_FILE").unwrap();
        let email = std::fs::read_to_string(email_file).unwrap();
        // https://meet-os.com/verify/login/c0514ec6-c51e-4376-ae8e-df82ef79bcef
        let re = Regex::new(r"http://localhost:8000/verify/login/([a-z0-9-]+)").unwrap();

        log::info!("email: {email}");
        let code = match re.captures(&email) {
            Some(value) => value[1].to_owned(),
            None => panic!("Code not found in email"),
        };
        println!("code: {code}");
        //assert_eq!(code, res.code);
        //        std::thread::sleep(std::time::Duration::from_millis(500));

        // "Click" on the link an verify the email
        let res = client
            .get(format!("http://localhost:{port}/verify/login/{code}"))
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
        //assert_eq!(html, "x");
        check_html(&html, "title", "Welcome back");
        assert!(html.contains(r#"<a href="/profile">profile</a>"#));
        //        std::thread::sleep(std::time::Duration::from_millis(500));

        // Access the profile with the cookie
        let response = client
            .get(format!("http://localhost:{port}/profile"))
            .header("Cookie", format!("meet-os={cookie_str}"))
            .send()
            .unwrap();
        assert_eq!(response.status(), 200);
        let html = response.text().unwrap();
        //assert_eq!(html, "x");
        check_html(&html, "title", "Profile");
        check_html(&html, "h1", "Foo Bar");
    });
}

#[test]
fn register_with_bad_email_address() {
    run_external(|port| {
        // register new user
        let client = reqwest::blocking::Client::new();
        let res = client
            .post(format!("http://localhost:{port}/register"))
            .form(&[("name", "Foo Bar"), ("email", "meet-os.com")])
            .send()
            .unwrap();
        assert_eq!(res.status(), 200); // TODO should this stay 200 OK?

        let html = res.text().unwrap();
        // TODO make these tests parse the HTML and verify the extracted title tag!
        //assert_eq!(html, "");
        check_html(&html, "title", "Invalid email address");
        assert!(html.contains("Invalid email address <b>meet-os.com</b> Please try again"));
    });
}

#[test]
fn event_page() {
    run_external(|port| {
        let client = reqwest::blocking::Client::new();
        let res = client
            .get(format!("http://localhost:{port}/event/1"))
            .send()
            .unwrap();
        assert_eq!(res.status(), 200);
        let html = res.text().unwrap();
        check_html(&html, "title", "Web development with Rocket");
        check_html(&html, "h1", "Web development with Rocket");
        assert!(html.contains(r#"Organized by <a href="/group/1">Rust Maven</a>."#));
        assert!(html.contains(r#"<div><b>Date</b>: 2024-02-04T17:00:00 UTC</div>"#));
        assert!(html.contains(r#"<div><b>Location</b>: Virtual</div>"#));
    });
}

#[test]
fn group_page() {
    run_external(|port| {
        let client = reqwest::blocking::Client::new();
        let res = client
            .get(format!("http://localhost:{port}/group/1"))
            .send()
            .unwrap();
        assert_eq!(res.status(), 200);
        let html = res.text().unwrap();

        check_html(&html, "title", "Rust Maven");
        check_html(&html, "h1", "Rust Maven");
        assert!(html.contains(
            r#"<li><a href="/event/1">2024-02-04T17:00:00 - Web development with Rocket</a></li>"#
        ));
        assert!(html.contains(r#"<div><b>Location</b>: Virtual</div>"#));
    });
}

// TODO try to login with an email address that was not registered
