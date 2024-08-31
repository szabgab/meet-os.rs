use crate::test_lib::{
    check_profile_page_in_process, extract_cookie, params, register_user_helper, run_inprocess,
};
use rocket::http::{ContentType, Status};
use utilities::{
    check_admin_menu, check_guest_menu, check_html, check_user_menu, read_code_from_email,
};

#[test]
fn try_page_without_cookie() {
    run_inprocess(|email_folder, client| {
        for path in ["/profile", "/admin/create-group?uid=1", "/admin"] {
            // Access the profile without a cookie
            let res = client.get(path).dispatch();
            assert_eq!(res.status(), Status::Unauthorized, "{path}");
            let html = res.into_string().unwrap();
            //assert_eq!(html, "");
            check_html(&html, "title", "Not logged in");
            assert!(html.contains("You are not logged in"));
            check_guest_menu(&html);
        }
    });
}

#[test]
fn register_user() {
    run_inprocess(|email_folder, client| {
        let email = "foo@meet-os.com";
        let res = client
            .post(format!("/register"))
            .header(ContentType::Form)
            .body(params!([
                ("name", "Foo Bar"),
                ("email", email),
                ("password", "123456"),
            ]))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        assert!(res.headers().get_one("set-cookie").is_none());

        let html = res.into_string().unwrap();
        check_html(&html, "title", "We sent you an email");
        assert!(html.contains("We sent you an email to <b>foo@meet-os.com</b> Please check your inbox and verify your email address."));
        check_guest_menu(&html);

        let (uid, code) = read_code_from_email(&email_folder, "0.txt");

        // Verify the email
        let res = client.get(format!("/verify-email/{uid}/{code}")).dispatch();
        assert_eq!(res.status(), Status::Ok);

        let cookie_str = extract_cookie(&res);

        let html = res.into_string().unwrap();
        check_html(&html, "title", "Thank you for registering");
        assert!(html.contains("Your email was verified."));
        check_user_menu(&html);

        // Access the profile with the cookie
        check_profile_page_in_process(&client, email, "Foo Bar");
    });
}

#[test]
fn verify_with_non_existent_id() {
    run_inprocess(|email_folder, client| {
        let res = client.get(format!("/verify-email/1/abc")).dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        //assert_eq!(html, "");
        check_html(&html, "title", "Invalid id");
        assert!(html.contains("Invalid id <b>1</b>"));
    });
}

#[test]
fn verify_with_bad_code() {
    run_inprocess(|email_folder, client| {
        let res = client
            .post(format!("/register"))
            .header(ContentType::Form)
            .body(params!([
                ("name", "Foo Bar"),
                ("email", "foo@meet-os.com"),
                ("password", "123456"),
            ]))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);

        let res = client.get(format!("/verify-email/1/abc")).dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        //assert_eq!(html, "");
        check_html(&html, "title", "Invalid code");
        assert!(html.contains("Invalid code <b>abc</b>"));
    });
}

#[test]
fn duplicate_email() {
    run_inprocess(|email_folder, client| {
        let res = client
            .post(format!("/register"))
            .header(ContentType::Form)
            .body(params!([
                ("name", "Foo Bar"),
                ("email", "foo@meet-os.com"),
                ("password", "123456"),
            ]))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        //println!("{:#?}", res.headers());
        assert!(res.headers().get_one("set-cookie").is_none());
        let html = res.into_string().unwrap();
        check_guest_menu(&html);
        check_html(&html, "title", "We sent you an email");
        assert!(html.contains("We sent you an email to <b>foo@meet-os.com</b> Please check your inbox and verify your email address."));

        let res = client
            .post(format!("/register"))
            .header(ContentType::Form)
            .body(params!([
                ("name", "Foo Bar"),
                ("email", "foo@meet-os.com"),
                ("password", "123456"),
            ]))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        //println!("{:#?}", res.headers());
        assert!(res.headers().get_one("set-cookie").is_none());
        let html = res.into_string().unwrap();
        check_html(&html, "title", "Registration failed");
        check_guest_menu(&html);
    });
}

#[test]
fn login_regular_user() {
    run_inprocess(|email_folder, client| {
        let foo_mail = "foo@meet-os.com";
        let _cookie_str =
            register_user_helper(&client, "Foo Bar", foo_mail, "123456", &email_folder);

        check_profile_page_in_process(&client, &foo_mail, "Foo Bar");

        let res = client
            .post("/login")
            .header(ContentType::Form)
            .body(params!([("email", foo_mail), ("password", "123456")]))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);

        let cookie_str = extract_cookie(&res);
        let html = res.into_string().unwrap();

        //assert_eq!(html, "");
        check_html(&html, "title", "Welcome back");
        check_user_menu(&html);

        // Access the profile with the cookie
        check_profile_page_in_process(&client, &foo_mail, "Foo Bar");

        // TODO: logout requires a logged in user
        //let res = client.get("/logout").dispatch();
        // assert_eq!(res.status(), Status::Unauthorized);
        //let html = res.into_string().unwrap();
        // check_html(&html, "title", "Not logged in");
        // assert!(html.contains("You are not logged in"));

        // logout
        let res = client
            .get("/logout")
            .private_cookie(("meed-os", foo_mail))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();

        check_html(&html, "title", "Logged out");
        check_html(&html, "h1", "Logged out");
        check_guest_menu(&html);

        // TODO as the login information is only saved in the client-side cookie, if someone has the cookie they can
        // use it even the user has clicked on /logout and we have asked the browser to remove the cookie.
        // If we want to make sure that the user cannot access the system any more we'll have to manage the login information
        // on the server side.
        //check_profile_page_in_process(&client, &url, &cookie_str, "");
    });
}

#[test]
fn login_admin_user() {
    run_inprocess(|email_folder, client| {
        let name = "Site Manager";
        let email = "admin@meet-os.com";
        let password = "123456";

        let _cookie_str = register_user_helper(&client, name, email, password, &email_folder);

        let res = client
            .post("/login")
            .header(ContentType::Form)
            .body(params!([("email", email), ("password", password)]))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);

        let cookie_str = extract_cookie(&res);
        let html = res.into_string().unwrap();

        //assert_eq!(html, "x");
        check_html(&html, "title", "Welcome back");
        check_admin_menu(&html);

        // // Access the profile with the cookie
        check_profile_page_in_process(&client, &email, name);

        let res = client
            .get("/logout")
            .private_cookie(("meed-os", email))
            .dispatch();

        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        check_html(&html, "title", "Logged out");
        check_html(&html, "h1", "Logged out");

        // TODO as the login information is only saved in the client-side cookie, if someone has the cookie they can
        // use it even the user has clicked on /logout and we have asked the browser to remove the cookie.
        // If we want to make sure that the user cannot access the system any more we'll have to manage the login information
        // on the server side.
        //check_profile_page_in_process(&client, &url, &cookie_str, "");
    });
}

// #[test]
// fn login() {
//    run_inprocess(|email_folder, client| {

//         let _cookie_str = register_user_helper(&client, "Foo Bar", "foo@meet-os.com", "123456");
//         //println!("cookie: {cookie_str}");
//         //check_profile_page_in_process(&client, &url, &cookie_str, "Peti Bar");

//         let res = client
//             .post("/login")
// .header(ContentType::Form)

//             .form(&[("email", "foo@meet-os.com")])
//             .dispatch();
// assert_eq!(res.status(), Status::Ok);
// let html = res.into_string().unwrap();
//         assert!(res.headers().get("set-cookie").is_none());
//         check_html(&html, "title", "We sent you an email");
//         assert!(html.contains("We sent you an email to <b>foo@meet-os.com</b>"));

//         // TODO: get the user from the database and check if there is a code and if the process is "login"

//         // get the email and extract the code from the link
//         let email_file = format!("{email_folder}/0.txt");
//         let email = std::fs::read_to_string(email_file).unwrap();
//         // https://meet-os.com/verify/login/c0514ec6-c51e-4376-ae8e-df82ef79bcef
//         let re = Regex::new("http://localhost:[0-9]+/verify/login/([a-z0-9-]+)").unwrap();

//         log::info!("email: {email}");
//         let code = match re.captures(&email) {
//             Some(value) => value[1].to_owned(),
//             None => panic!("Code not found in email"),
//         };
//         println!("code: {code}");
//         //assert_eq!(code, res.code);

//         // "Click" on the link an verify the email
//         let res = client
//             .get("/verify/login/{code}")
//             .dispach();
// assert_eq!(res.status(), Status::Ok);
// let html = res.into_string().unwrap();
//         let cookie_str = extract_cookie(&res);
//         //assert_eq!(html, "x");
//         check_html(&html, "title", "Welcome back");
//         check_user_menu(&html);

//         // Access the profile with the cookie
//         check_profile_page_in_process(&client, &url, &cookie_str, "Foo Bar");

//         let res = client.get("/logout").dispatch();
// assert_eq!(res.status(), Status::Ok);
// let html = res.into_string().unwrap();
//         check_html(&html, "title", "Logged out");
//         check_html(&html, "h1", "Logged out");

//         // TODO as the login information is only saved in the client-side cookie, if someone has the cookie they can
//         // use it even the user has clicked on /logout and we have asked the browser to remove the cookie.
//         // If we want to make sure that the user cannot access the system any more we'll have to manage the login information
//         // on the server side.
//         //check_profile_page_in_process(&client, &url, &cookie_str, "");
//     });
// }

#[test]
fn register_with_bad_email_address() {
    run_inprocess(|email_folder, client| {
        // register new user
        let res = client
            .post("/register")
            .header(ContentType::Form)
            .body(params!([
                ("name", "Foo Bar"),
                ("email", "meet-os.com"),
                ("password", "123456"),
            ]))
            .dispatch();
        // TODO should this stay 200 OK?
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();

        // TODO make these tests parse the HTML and verify the extracted title tag!
        //assert_eq!(html, "");
        check_html(&html, "title", "Invalid email address");
        assert!(html.contains("Invalid email address <b>meet-os.com</b> Please try again"));
        check_guest_menu(&html);
    });
}

// #[test]
// fn event_page() {
//    run_inprocess(|email_folder, client| {
//         let res = client.get("/event/1").dispatch();
// assert_eq!(res.status(), Status::Ok);
// let html = res.into_string().unwrap();
//         check_html(&html, "title", "Web development with Rocket");
//         check_html(&html, "h1", "Web development with Rocket");
//         assert!(html.contains(r#"Organized by <a href="/group/1">Rust Maven</a>."#));
//         assert!(html.contains("<div><b>Date</b>: 2024-02-04T17:00:00 UTC</div>"));
//         assert!(html.contains("<div><b>Location</b>: Virtual</div>"));
//     });
// }

// #[test]
// fn group_page() {
//    run_inprocess(|email_folder, client| {
//         let res = client.get("/group/1").dispatch();
// assert_eq!(res.status(), Status::Ok);
// let html = res.into_string().unwrap();

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
    run_inprocess(|email_folder, client| {
        let res = client
            .post("/login")
            .header(ContentType::Form)
            .body(params!([
                ("email", "other@meet-os.com"),
                ("password", "123456")
            ]))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        assert!(res.headers().get_one("set-cookie").is_none());
        let html = res.into_string().unwrap();
        check_html(&html, "title", "No such user");
        assert!(html.contains("No user with address <b>other@meet-os.com</b>"));
        check_guest_menu(&html);
    });
}

#[test]
fn login_with_bad_password() {
    run_inprocess(|email_folder, client| {
        let _cookie_str = register_user_helper(
            &client,
            "Foo Bar",
            "foo@meet-os.com",
            "123456",
            &email_folder,
        );

        let res = client.get("/logout").dispatch();
        // TODO

        let res = client
            .post("/login")
            .header(ContentType::Form)
            .body(params!([
                ("email", "foo@meet-os.com"),
                ("password", "123457")
            ]))
            .dispatch();

        assert_eq!(res.status(), Status::Ok);
        assert!(res.headers().get_one("set-cookie").is_none());
        let html = res.into_string().unwrap();

        check_html(&html, "title", "Invalid password");
        check_html(&html, "h1", "Invalid password");
        // assert_eq!(&html, "");
        check_guest_menu(&html);
    });
}

#[test]
fn login_with_unverified_email() {
    run_inprocess(|email_folder, client| {
        let res = client
            .post("/register")
            .header(ContentType::Form)
            .body(params!([
                ("name", "Foo Bar"),
                ("email", "foo@meet-os.com"),
                ("password", "123456"),
            ]))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();

        let res = client
            .post("/login")
            .header(ContentType::Form)
            .body(params!([
                ("email", "foo@meet-os.com"),
                ("password", "123456")
            ]))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);

        assert!(res.headers().get_one("set-cookie").is_none());
        let html = res.into_string().unwrap();
        check_html(&html, "title", "Unverified email");
        assert!(html.contains("Email must be verified before login."));
        check_guest_menu(&html);
    });
}
