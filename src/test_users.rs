use crate::test_helpers::{
    logout, register_and_verify_user, setup_admin, setup_many_users, setup_owner,
    setup_unverified_user, setup_user, ADMIN_EMAIL, ADMIN_NAME, ADMIN_PW, FOO_EMAIL,
    UNVERIFIED_NAME,
};
use crate::test_lib::{
    check_admin_menu, check_guest_menu, check_html, check_profile_page_in_process, check_user_menu,
    params, read_code_from_email, run_inprocess,
};
use rocket::http::{ContentType, Status};

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

        let (uid, code) = read_code_from_email(&email_folder, "0.txt", "verify-email");

        // Verify the email
        let res = client.get(format!("/verify-email/{uid}/{code}")).dispatch();
        assert_eq!(res.status(), Status::Ok);

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
        register_and_verify_user(&client, "Foo Bar", FOO_EMAIL, "123456", &email_folder);

        check_profile_page_in_process(&client, &FOO_EMAIL, "Foo Bar");

        let res = client
            .post("/login")
            .header(ContentType::Form)
            .body(params!([("email", FOO_EMAIL), ("password", "123456")]))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);

        let html = res.into_string().unwrap();

        //assert_eq!(html, "");
        check_html(&html, "title", "Welcome back");
        check_user_menu(&html);

        // Access the profile with the cookie
        check_profile_page_in_process(&client, &FOO_EMAIL, "Foo Bar");

        // TODO: logout requires a logged in user
        //let res = client.get("/logout").dispatch();
        // assert_eq!(res.status(), Status::Unauthorized);
        //let html = res.into_string().unwrap();
        // check_html(&html, "title", "Not logged in");
        // assert!(html.contains("You are not logged in"));

        // logout
        let res = client
            .get("/logout")
            .private_cookie(("meet-os", FOO_EMAIL))
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
    });
}

#[test]
fn login_admin_user() {
    run_inprocess(|email_folder, client| {
        setup_admin(&client, &email_folder);
        logout(&client);

        let res = client
            .post("/login")
            .header(ContentType::Form)
            .body(params!([("email", ADMIN_EMAIL), ("password", ADMIN_PW)]))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);

        let html = res.into_string().unwrap();

        //assert_eq!(html, "x");
        check_html(&html, "title", "Welcome back");
        check_admin_menu(&html);

        // // Access the profile with the cookie
        check_profile_page_in_process(&client, &ADMIN_EMAIL, ADMIN_NAME);

        let res = client
            .get("/logout")
            .private_cookie(("meet-os", ADMIN_EMAIL))
            .dispatch();

        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        check_html(&html, "title", "Logged out");
        check_html(&html, "h1", "Logged out");

        // TODO as the login information is only saved in the client-side cookie, if someone has the cookie they can
        // use it even the user has clicked on /logout and we have asked the browser to remove the cookie.
        // If we want to make sure that the user cannot access the system any more we'll have to manage the login information
        // on the server side.
    });
}

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
        register_and_verify_user(
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

#[test]
fn login_with_invalid_email() {
    run_inprocess(|email_folder, client| {
        // no actual user needed in the system for this to work
        let res = client
            .post("/login")
            .header(ContentType::Form)
            .body(params!([("email", "meet-os.com"), ("password", "123456")]))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        //assert_eq!(html, "");
        check_html(&html, "title", "Invalid email address");
        check_html(&html, "h1", "Invalid email address");
    });
}

#[test]
fn register_with_short_password() {
    run_inprocess(|email_folder, client| {
        let res = client
            .post(format!("/register"))
            .header(ContentType::Form)
            .body(params!([
                ("name", "Foo Bar"),
                ("email", FOO_EMAIL),
                ("password", "123"),
            ]))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        assert!(res.headers().get_one("set-cookie").is_none());

        let html = res.into_string().unwrap();
        check_html(&html, "title", "Invalid password");
        check_html(&html, "h1", "Invalid password");
        //assert_eq!(html, "");
        assert!(html.contains("The password must be at least 6 characters long."));
        check_guest_menu(&html);
    });
}

#[test]
fn edit_profile_get_guest() {
    run_inprocess(|email_folder, client| {
        let res = client.get("/edit-profile").dispatch();

        assert_eq!(res.status(), Status::Unauthorized);
        let html = res.into_string().unwrap();

        check_html(&html, "title", "Not logged in");
    });
}

#[test]
fn edit_profile_get_user() {
    run_inprocess(|email_folder, client| {
        setup_owner(&client, &email_folder);

        let res = client
            .get("/edit-profile")
            .private_cookie(("meet-os", FOO_EMAIL))
            .dispatch();

        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();

        //assert_eq!(html, "");
        check_html(&html, "title", "Edit Profile");
        check_html(&html, "h1", "Edit Profile");
        assert!(html.contains(r#"<form method="POST" action="/edit-profile">"#));
    });
}

// edit_profile_get_unverified_user should fail

#[test]
fn register_get_guest() {
    run_inprocess(|email_folder, client| {
        let res = client.get("/register").dispatch();

        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();

        //assert_eq!(html, "");
        check_html(&html, "title", "Register");
        check_html(&html, "h1", "Register");
        assert!(html.contains(r#"<form method="POST" action="/register">"#));
    });
}

// register_get_user should fail
//
//
#[test]
fn list_users_empty_db_guest() {
    run_inprocess(|email_folder, client| {
        let res = client.get("/users").dispatch();

        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();

        //assert_eq!(html, "");
        check_html(&html, "title", "List Users");
        check_html(&html, "h1", "List Users");
        assert!(html.contains(r#"No users"#));
    });
}

#[test]
fn list_users_many_users_guest() {
    run_inprocess(|email_folder, client| {
        setup_many_users(&client, &email_folder);

        let res = client.get("/users").dispatch();

        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();

        // assert_eq!(html, "");
        check_html(&html, "title", "List Users");
        check_html(&html, "h1", "List Users");
        assert!(!html.contains(r#"No users"#));

        assert!(html.contains(r#"<li><a href="/user/2">Foo Bar</a></li>"#));
        assert!(html.contains(r#"<li><a href="/user/3">Foo 1</a></li>"#));
        assert!(html.contains(r#"<li><a href="/user/4">Foo 2</a></li>"#));
        assert!(html.contains(r#"<li><a href="/user/1">Site Manager</a></li>"#));
    });
}

#[test]
fn user_id_that_does_not_exist() {
    run_inprocess(|email_folder, client| {
        let res = client.get("/user/42").dispatch();

        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();

        // assert_eq!(html, "");
        check_html(&html, "title", "User not found");
        check_html(&html, "h1", "User not found");
        assert!(html.contains(r#"There is no user with id <b>42</b>."#));
    });
}

#[test]
fn user_page() {
    run_inprocess(|email_folder, client| {
        setup_admin(&client, &email_folder);
        setup_owner(&client, &email_folder);

        let res = client.get("/user/2").dispatch();

        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();

        //assert_eq!(html, "");
        check_html(&html, "title", "Foo Bar");
        check_html(&html, "h1", "Foo Bar");
        assert!(html.contains(r#"<td>Name:</td><td>Foo Bar</td>"#));
        assert!(html.contains(r#"<td>No GitHub provided.</td"#));
        assert!(html.contains(r#"<td>No GitLab provided.</td"#));
        assert!(html.contains(r#"<td>No LinkedIn provided.</td>"#));
    });
}

#[test]
fn unverified_user_page_by_guest() {
    run_inprocess(|email_folder, client| {
        setup_unverified_user(&client, &email_folder);
        logout(&client);

        let res = client.get("/user/1").dispatch();

        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        check_html(&html, "title", "Unverified user");
        check_html(&html, "h1", "Unverified user");
        assert!(html.contains("This user has not verified the email address yet."));
        assert!(!html.contains(UNVERIFIED_NAME));
    });

    // TODO check by logged in user, owner, and admin as well
}

#[test]
fn unverified_user_on_user_page_by_guest() {
    run_inprocess(|email_folder, client| {
        setup_admin(&client, &email_folder);
        setup_user(&client, &email_folder);
        setup_unverified_user(&client, &email_folder);
        logout(&client);

        let res = client.get("/users").dispatch();

        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        //assert_eq!(html, "");
        check_html(&html, "title", "List Users");
        check_html(&html, "h1", "List Users");
        assert!(html.contains(r#"<a href="/user/1">Site Manager</a>"#));
        assert!(html.contains(r#"<a href="/user/2">Foo 1</a></li>"#));
        assert!(!html.contains(UNVERIFIED_NAME));

        // TODO check by logged in user, owner, and admin as well
    });
}
