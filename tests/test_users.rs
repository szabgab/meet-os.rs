use regex::Regex;

use utilities::{
    check_guest_menu, check_html, check_profile_page, check_user_menu, register_user_helper,
    run_external,
};

#[test]
fn profile_without_cookie() {
    run_external(|port| {
        let client = reqwest::blocking::Client::new();
        let url = format!("http://localhost:{port}");

        for path in ["/profile", "/create-group", "/admin"] {
            // Access the profile without a cookie
            let res = client.get(format!("{url}{path}")).send().unwrap();
            assert_eq!(res.status(), 200);
            let html = res.text().unwrap();
            //assert_eq!(html, "");
            check_html(&html, "title", "Not logged in");
            assert!(html.contains("It seems you are not logged in"));
            check_guest_menu(&html);
        }
    });
}

#[test]
fn register_user() {
    run_external(|port| {
        let client = reqwest::blocking::Client::new();
        let url = format!("http://localhost:{port}");
        let res = client
            .post(format!("{url}/register"))
            .form(&[
                ("name", "Foo Bar"),
                ("email", "foo@meet-os.com"),
                ("password", "123456"),
            ])
            .send()
            .unwrap();
        assert_eq!(res.status(), 200);
        //println!("{:#?}", res.headers());
        assert!(res.headers().get("set-cookie").is_none());

        let html = res.text().unwrap();
        check_html(&html, "title", "We sent you an email");
        assert!(html.contains("We sent you an email to <b>foo@meet-os.com</b> Please check your inbox and verify your email address."));
        check_guest_menu(&html);

        let email_file = std::env::var("EMAIL_FILE").unwrap();

        let email_content = std::fs::read_to_string(email_file).unwrap();
        // https://meet-os.com/verify/register/c0514ec6-c51e-4376-ae8e-df82ef79bcef
        let re = Regex::new("http://localhost:8000/verify/register/([a-z0-9-]+)").unwrap();

        println!("email content: {email_content}");
        let code = match re.captures(&email_content) {
            Some(value) => value[1].to_owned(),
            None => panic!("Code not found in email: {email_content}"),
        };

        //assert_eq!(code, res.code);
        println!("code: {code}");

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
        let res = client
            .get(format!("{url}/verify/register/{code}"))
            .send()
            .unwrap();
        assert_eq!(res.status(), 200);
        let cookie = res.headers().get("set-cookie").unwrap().to_str().unwrap();
        println!("cookie: {cookie}");
        assert!(cookie.contains("meet-os="));
        let re = Regex::new("meet-os=([^;]+);").unwrap();
        let cookie_str = match re.captures(cookie) {
            Some(value) => value[1].to_owned(),
            None => panic!("Code not found cookie"),
        };
        println!("cookie_str: {cookie_str}");

        let html = res.text().unwrap();
        check_html(&html, "title", "Thank you for registering");
        assert!(html.contains("Your email was verified."));
        check_user_menu(&html);

        // Access the profile with the cookie
        check_profile_page(&client, &url, &cookie_str, "Foo Bar");
    });
}

#[test]
fn verify_with_non_existent_code() {
    run_external(|port| {
        let client = reqwest::blocking::Client::new();
        let url = format!("http://localhost:{port}/");
        let res = client
            .get(format!("{url}/verify/register/abc"))
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
        let url = format!("http://localhost:{port}/");
        let res = client
            .post(format!("{url}/register"))
            .form(&[
                ("name", "Foo Bar"),
                ("email", "foo@meet-os.com"),
                ("password", "123456"),
            ])
            .send()
            .unwrap();
        assert_eq!(res.status(), 200);
        //println!("{:#?}", res.headers());
        assert!(res.headers().get("set-cookie").is_none());
        let html = res.text().unwrap();
        check_guest_menu(&html);
        check_html(&html, "title", "We sent you an email");
        assert!(html.contains("We sent you an email to <b>foo@meet-os.com</b> Please check your inbox and verify your email address."));

        let res = client
            .post(format!("{url}/register"))
            .form(&[
                ("name", "Foo Bar"),
                ("email", "foo@meet-os.com"),
                ("password", "123456"),
            ])
            .send()
            .unwrap();
        assert_eq!(res.status(), 200);
        //println!("{:#?}", res.headers());
        assert!(res.headers().get("set-cookie").is_none());
        let html = res.text().unwrap();
        check_html(&html, "title", "Registration failed");
        check_guest_menu(&html);
    });
}

#[test]
fn login_user() {
    run_external(|port| {
        let client = reqwest::blocking::Client::new();
        let url = format!("http://localhost:{port}/");

        let _cookie_str =
            register_user_helper(&client, &url, "Foo Bar", "foo@meet-os.com", "123456");
        //println!("cookie: {cookie_str}");
        //check_profile_page(&client, &url, &cookie_str, "Peti Bar");

        let res = client
            .post(format!("{url}/login"))
            .form(&[("email", "foo@meet-os.com"), ("password", "123456")])
            .send()
            .unwrap();
        assert_eq!(res.status(), 200);

        let cookie = res.headers().get("set-cookie").unwrap().to_str().unwrap();
        println!("cookie: {cookie}");
        assert!(cookie.contains("meet-os="));
        let re = Regex::new("meet-os=([^;]+);").unwrap();
        let cookie_str = match re.captures(cookie) {
            Some(value) => value[1].to_owned(),
            None => panic!("Code not found cookie"),
        };
        println!("cookie_str: {cookie_str}");

        let html = res.text().unwrap();
        //assert_eq!(html, "x");
        check_html(&html, "title", "Welcome back");
        check_user_menu(&html);

        // // Access the profile with the cookie
        check_profile_page(&client, &url, &cookie_str, "Foo Bar");

        let res = client.get(format!("{url}/logout")).send().unwrap();
        assert_eq!(res.status(), 200);
        let html = res.text().unwrap();
        check_html(&html, "title", "Logged out");
        check_html(&html, "h1", "Logged out");
        //check_guest_menu(&html);

        // TODO as the login information is only saved in the client-side cookie, if someone has the cookie they can
        // use it even the user has clicked on /logout and we have asked the browser to remove the cookie.
        // If we want to make sure that the user cannot access the system any more we'll have to manage the login information
        // on the server side.
        //check_profile_page(&client, &url, &cookie_str, "");
    });
}

// #[test]
// fn login() {
//     run_external(|port| {
//         let client = reqwest::blocking::Client::new();
//         let url = format!("http://localhost:{port}/");

//         let _cookie_str = register_user_helper(&client, &url, "Foo Bar", "foo@meet-os.com", "123456");
//         //println!("cookie: {cookie_str}");
//         //check_profile_page(&client, &url, &cookie_str, "Peti Bar");

//         let res = client
//             .post(format!("{url}/login"))
//             .form(&[("email", "foo@meet-os.com")])
//             .send()
//             .unwrap();
//         assert_eq!(res.status(), 200);
//         assert!(res.headers().get("set-cookie").is_none());
//         let html = res.text().unwrap();
//         check_html(&html, "title", "We sent you an email");
//         assert!(html.contains("We sent you an email to <b>foo@meet-os.com</b>"));

//         // TODO: get the user from the database and check if there is a code and if the process is "login"

//         // get the email and extract the code from the link
//         let email_file = std::env::var("EMAIL_FILE").unwrap();
//         let email = std::fs::read_to_string(email_file).unwrap();
//         // https://meet-os.com/verify/login/c0514ec6-c51e-4376-ae8e-df82ef79bcef
//         let re = Regex::new("http://localhost:8000/verify/login/([a-z0-9-]+)").unwrap();

//         log::info!("email: {email}");
//         let code = match re.captures(&email) {
//             Some(value) => value[1].to_owned(),
//             None => panic!("Code not found in email"),
//         };
//         println!("code: {code}");
//         //assert_eq!(code, res.code);

//         // "Click" on the link an verify the email
//         let res = client
//             .get(format!("{url}/verify/login/{code}"))
//             .send()
//             .unwrap();
//         assert_eq!(res.status(), 200);
//         let cookie = res.headers().get("set-cookie").unwrap().to_str().unwrap();
//         println!("cookie: {cookie}");
//         assert!(cookie.contains("meet-os="));
//         let re = Regex::new("meet-os=([^;]+);").unwrap();
//         let cookie_str = match re.captures(cookie) {
//             Some(value) => value[1].to_owned(),
//             None => panic!("Code not found cookie"),
//         };
//         println!("cookie_str: {cookie_str}");

//         let html = res.text().unwrap();
//         //assert_eq!(html, "x");
//         check_html(&html, "title", "Welcome back");
//         check_user_menu(&html);

//         // Access the profile with the cookie
//         check_profile_page(&client, &url, &cookie_str, "Foo Bar");

//         let res = client.get(format!("{url}/logout")).send().unwrap();
//         assert_eq!(res.status(), 200);
//         let html = res.text().unwrap();
//         check_html(&html, "title", "Logged out");
//         check_html(&html, "h1", "Logged out");

//         // TODO as the login information is only saved in the client-side cookie, if someone has the cookie they can
//         // use it even the user has clicked on /logout and we have asked the browser to remove the cookie.
//         // If we want to make sure that the user cannot access the system any more we'll have to manage the login information
//         // on the server side.
//         //check_profile_page(&client, &url, &cookie_str, "");
//     });
// }

#[test]
fn register_with_bad_email_address() {
    run_external(|port| {
        // register new user
        let client = reqwest::blocking::Client::new();
        let url = format!("http://localhost:{port}/");
        let res = client
            .post(format!("{url}/register"))
            .form(&[
                ("name", "Foo Bar"),
                ("email", "meet-os.com"),
                ("password", "123456"),
            ])
            .send()
            .unwrap();
        assert_eq!(res.status(), 200); // TODO should this stay 200 OK?

        let html = res.text().unwrap();
        // TODO make these tests parse the HTML and verify the extracted title tag!
        //assert_eq!(html, "");
        check_html(&html, "title", "Invalid email address");
        assert!(html.contains("Invalid email address <b>meet-os.com</b> Please try again"));
        check_guest_menu(&html);
    });
}

// #[test]
// fn event_page() {
//     run_external(|port| {
//         let client = reqwest::blocking::Client::new();
//         let url = format!("http://localhost:{port}/");
//         let res = client.get(format!("{url}/event/1")).send().unwrap();
//         assert_eq!(res.status(), 200);
//         let html = res.text().unwrap();
//         check_html(&html, "title", "Web development with Rocket");
//         check_html(&html, "h1", "Web development with Rocket");
//         assert!(html.contains(r#"Organized by <a href="/group/1">Rust Maven</a>."#));
//         assert!(html.contains("<div><b>Date</b>: 2024-02-04T17:00:00 UTC</div>"));
//         assert!(html.contains("<div><b>Location</b>: Virtual</div>"));
//     });
// }

// #[test]
// fn group_page() {
//     run_external(|port| {
//         let client = reqwest::blocking::Client::new();
//         let url = format!("http://localhost:{port}/");
//         let res = client.get(format!("{url}/group/1")).send().unwrap();
//         assert_eq!(res.status(), 200);
//         let html = res.text().unwrap();

//         check_html(&html, "title", "Rust Maven");
//         check_html(&html, "h1", "Rust Maven");
//         assert!(html.contains(
//             r#"<li><a href="/event/1">2024-02-04T17:00:00 - Web development with Rocket</a></li>"#
//         ));
//         assert!(html.contains("<div><b>Location</b>: Virtual</div>"));
//     });
// }

// TODO try to login with an email address that was not registered

#[test]
fn login_with_unregistered_email() {
    run_external(|port| {
        let client = reqwest::blocking::Client::new();
        let url = format!("http://localhost:{port}/");

        let res = client
            .post(format!("{url}/login"))
            .form(&[("email", "other@meet-os.com"), ("password", "123456")])
            .send()
            .unwrap();
        assert_eq!(res.status(), 200);
        assert!(res.headers().get("set-cookie").is_none());
        let html = res.text().unwrap();
        check_html(&html, "title", "No such user");
        assert!(html.contains("No user with address <b>other@meet-os.com</b>"));
        check_guest_menu(&html);
    });
}

#[test]
fn login_with_bad_password() {
    run_external(|port| {
        let client = reqwest::blocking::Client::new();
        let url = format!("http://localhost:{port}/");

        let _cookie_str =
            register_user_helper(&client, &url, "Foo Bar", "foo@meet-os.com", "123456");

        let res = client
            .post(format!("{url}/login"))
            .form(&[("email", "foo@meet-os.com"), ("password", "123457")])
            .send()
            .unwrap();
        assert_eq!(res.status(), 200);
        assert!(res.headers().get("set-cookie").is_none());

        let html = res.text().unwrap();
        check_html(&html, "title", "Invalid password");
        check_html(&html, "h1", "Invalid password");
        check_guest_menu(&html);
    });
}
