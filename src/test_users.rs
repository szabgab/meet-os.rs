use crate::test_helpers::{
    logout, register_and_verify_user, setup_admin, setup_many_users, setup_owner,
    setup_unverified_user, setup_user, ADMIN_EMAIL, ADMIN_NAME, ADMIN_PW, OWNER_EMAIL, OWNER_NAME,
    OWNER_PW, UNVERIFIED_NAME,
};
use crate::test_lib::{
    check_admin_menu, check_guest_menu, check_html, check_not_logged_in, check_profile_by_guest,
    check_profile_by_user, check_user_menu, params, read_code_from_email, run_inprocess,
};
use rocket::http::{ContentType, Status};

#[test]
fn protected_pages_as_guest() {
    run_inprocess(|email_folder, client| {
        for path in [
            "/admin",
            "/admin/audit",
            "/admin/create-group?uid=1",
            "/admin/search",
            "/admin/users",
            "/join-group?gid=1",
            "/leave-group?gid=1",
            "/edit-group",
            "/contact-members",
            "/add-event",
            "/edit-event",
            "/rsvp-yes-event?eid=1",
            "/rsvp-no-event?eid=1",
            "/profile",
        ] {
            let res = client.get(path).dispatch();
            check_not_logged_in(res);
        }
    });
}

#[test]
fn protected_post_requests_as_guest() {
    run_inprocess(|email_folder, client| {
        for path in [
            "/contact-members",
            "/add-event",
            "/edit-event",
            "/edit-group",
            "/admin/search",
            "/admin/create-group",
        ] {
            let res = client.post(path).header(ContentType::Form).dispatch();

            check_not_logged_in(res);
        }

        // Create group should fail even if we have the parameters
        let res = client
            .post("/admin/create-group")
            .body(params!([
                ("name", "Rust Maven"),
                ("location", ""),
                ("description", ""),
                ("owner", "1"),
            ]))
            .dispatch();
        check_not_logged_in(res);
    });
}

#[test]
fn register_user() {
    run_inprocess(|email_folder, client| {
        let res = client
            .post(format!("/register"))
            .header(ContentType::Form)
            .body(params!([
                ("name", OWNER_NAME),
                ("email", OWNER_EMAIL),
                ("password", OWNER_PW),
            ]))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        assert!(res.headers().get_one("set-cookie").is_none());

        let html = res.into_string().unwrap();
        check_html(&html, "title", "We sent you an email");
        let expected = format!("We sent you an email to <b>{OWNER_EMAIL}</b> Please check your inbox and verify your email address.");
        assert!(html.contains(&expected));
        check_guest_menu(&html);

        let (uid, code) = read_code_from_email(&email_folder, "0.txt", "verify-email");

        // Verify the email
        let res = client.get(format!("/verify-email/{uid}/{code}")).dispatch();
        assert_eq!(res.status(), Status::Ok);

        let html = res.into_string().unwrap();
        check_html(&html, "title", "Thank you for registering");
        assert!(html.contains("Your email was verified."));
        check_user_menu(&html);

        check_profile_by_user(&client, OWNER_EMAIL, OWNER_NAME);
    });
}

// TODO resend code?

#[test]
fn get_verify_with_non_existent_id() {
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
fn get_verify_email_with_bad_code() {
    run_inprocess(|email_folder, client| {
        let res = client
            .post(format!("/register"))
            .header(ContentType::Form)
            .body(params!([
                ("name", OWNER_NAME),
                ("email", OWNER_EMAIL),
                ("password", OWNER_PW),
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
fn post_register_duplicate_email() {
    run_inprocess(|email_folder, client| {
        let res = client
            .post(format!("/register"))
            .header(ContentType::Form)
            .body(params!([
                ("name", OWNER_NAME),
                ("email", OWNER_EMAIL),
                ("password", OWNER_PW),
            ]))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        //println!("{:#?}", res.headers());
        assert!(res.headers().get_one("set-cookie").is_none());
        let html = res.into_string().unwrap();
        check_guest_menu(&html);
        check_html(&html, "title", "We sent you an email");
        let expected = format!("We sent you an email to <b>{OWNER_EMAIL}</b> Please check your inbox and verify your email address.");
        assert!(html.contains(&expected));

        let res = client
            .post(format!("/register"))
            .header(ContentType::Form)
            .body(params!([
                ("name", OWNER_NAME),
                ("email", OWNER_EMAIL),
                ("password", OWNER_PW),
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
fn post_login_regular_user() {
    run_inprocess(|email_folder, client| {
        register_and_verify_user(&client, OWNER_NAME, OWNER_EMAIL, OWNER_PW, &email_folder);

        check_profile_by_user(&client, &OWNER_EMAIL, OWNER_NAME);

        let res = client
            .post("/login")
            .header(ContentType::Form)
            .body(params!([("email", OWNER_EMAIL), ("password", OWNER_PW)]))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);

        let html = res.into_string().unwrap();

        //assert_eq!(html, "");
        check_html(&html, "title", "Welcome back");
        check_user_menu(&html);

        // Access the profile with the cookie
        check_profile_by_user(&client, &OWNER_EMAIL, "Foo Bar");

        // TODO: logout requires a logged in user
        //let res = client.get("/logout").dispatch();
        // assert_eq!(res.status(), Status::Unauthorized);
        //let html = res.into_string().unwrap();
        // check_html(&html, "title", "Not logged in");
        // assert!(html.contains("You are not logged in"));

        // logout
        let res = client
            .get("/logout")
            .private_cookie(("meet-os", OWNER_EMAIL))
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
fn post_login_admin() {
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
        check_profile_by_user(&client, &ADMIN_EMAIL, ADMIN_NAME);

        let res = client
            .get("/logout")
            .private_cookie(("meet-os", ADMIN_EMAIL))
            .dispatch();

        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        check_html(&html, "title", "Logged out");
        check_html(&html, "h1", "Logged out");
        check_profile_by_guest(&client);

        // TODO as the login information is only saved in the client-side cookie, if someone has the cookie they can
        // use it even the user has clicked on /logout and we have asked the browser to remove the cookie.
        // If we want to make sure that the user cannot access the system any more we'll have to manage the login information
        // on the server side.
    });
}

#[test]
fn test_register_with_invalid_email_address() {
    run_inprocess(|email_folder, client| {
        //"name=Foo Bar&email=meet-os.com&password=123456"
        let res = client
            .post("/register")
            .header(ContentType::Form)
            .body(params!([
                ("name", "Foo Bar"),
                ("email", "meet-os.com"),
                ("password", "123456"),
            ]))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        check_html(&html, "title", "Invalid email address");
        assert!(html.contains("Invalid email address <b>meet-os.com</b> Please try again"));
    });
}

#[test]
fn test_register_with_too_long_username() {
    run_inprocess(|email_folder, client| {
        let res = client
            .post("/register")
            .header(ContentType::Form)
            .body("name=QWERTYUIOPASDFGHJKLZXCVBNM QWERTYUIOPASDFGHJKLZXCVBNM&email=long@meet-os.com&password=123456")
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        check_html(&html, "title", "Name is too long");
        assert!(html.contains(
            "Name is too long. Max 50 while the current name is 53 long. Please try again."
        ));
    });
}

#[test]
fn post_register_with_bad_email_address() {
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
fn post_login_with_unregistered_email() {
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
fn post_login_with_bad_password() {
    run_inprocess(|email_folder, client| {
        register_and_verify_user(
            &client,
            "Foo Bar",
            "foo@meet-os.com",
            "123456",
            &email_folder,
        );

        logout(&client);

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
fn post_login_with_unverified_email() {
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
fn post_login_with_invalid_email() {
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
fn post_register_with_short_password() {
    run_inprocess(|email_folder, client| {
        let res = client
            .post(format!("/register"))
            .header(ContentType::Form)
            .body(params!([
                ("name", "Foo Bar"),
                ("email", OWNER_EMAIL),
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
fn get_edit_profile_guest() {
    run_inprocess(|email_folder, client| {
        let res = client.get("/edit-profile").dispatch();

        assert_eq!(res.status(), Status::Unauthorized);
        let html = res.into_string().unwrap();

        check_html(&html, "title", "Not logged in");
    });
}

#[test]
fn get_edit_profile_user() {
    run_inprocess(|email_folder, client| {
        setup_owner(&client, &email_folder);

        let res = client
            .get("/edit-profile")
            .private_cookie(("meet-os", OWNER_EMAIL))
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
fn get_register_guest() {
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
fn get_users_list_users_empty_db_guest() {
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
fn get_users_list_users_many_users_guest() {
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
fn get_user_page() {
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

#[test]
fn post_edit_profile_failures() {
    run_inprocess(|email_folder, client| {
        setup_owner(&client, &email_folder);

        // edit profile page invalid github account
        let res = client
            .post("/edit-profile")
            .private_cookie(("meet-os", OWNER_EMAIL))
            .header(ContentType::Form)
            .body("name=XX&github=szabgab*&gitlab=&linkedin&about=")
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        check_html(&html, "title", "Invalid GitHub username");
        assert!(html.contains(r#"The GitHub username `szabgab*` is not valid."#));

        // edit profile page invalid gitlab account
        let res = client
            .post("/edit-profile")
            .private_cookie(("meet-os", OWNER_EMAIL))
            .header(ContentType::Form)
            .body("name=XX&github=&gitlab=foo*bar&linkedin=&about=")
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        check_html(&html, "title", "Invalid GitLab username");
        assert!(html.contains(r#"The GitLab username `foo*bar` is not valid."#));

        let res = client
            .post("/edit-profile")
            .private_cookie(("meet-os", OWNER_EMAIL))
            .header(ContentType::Form)
            .body("name=XX&github=&gitlab=&linkedin=szabgab&about=")
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        check_html(&html, "title", "Invalid LinkedIn profile link");
        assert!(html.contains(r#"The LinkedIn profile link `szabgab` is not valid."#));

        // TODO test the validation of the other fields as well!
    });
}

#[test]
fn post_edit_profile_works() {
    run_inprocess(|email_folder, client| {
        setup_owner(&client, &email_folder);

        // verify that if we submit html tags to the about field, those are properly escaped in the result
        let res = client
            .post("/edit-profile")
            .private_cookie(("meet-os", OWNER_EMAIL))
            .header(ContentType::Form)
            .body("name= Lord 😎 Voldemort &github= alfa &gitlab= beta &linkedin=  https://www.linkedin.com/in/szabgab/  &about=* text\n* more\n* [link](https://meet-os.com/)\n* <b>bold</b>\n* <a href=\"https://meet-os.com/\">bad link</a>\n")
            .dispatch();

        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        check_html(&html, "title", "Profile updated");
        assert!(html.contains(r#"Check out the <a href="/profile">profile</a> and how others see it <a href="/user/1">Lord 😎 Voldemort</a>"#));

        // Check updated profile
        let res = client
            .get("/profile")
            .private_cookie(("meet-os", OWNER_EMAIL))
            .dispatch();

        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        check_html(&html, "title", "Profile");
        assert!(html.contains(r#"<h1 class="title is-3">Lord 😎 Voldemort</h1>"#));
        assert!(html.contains(r#"<div><a href="https://github.com/alfa">GitHub</a></div>"#));
        assert!(html.contains(r#"<div><a href="https://gitlab.com/beta">GitLab</a></div>"#));

        // TODO: do we need to escape the characters when we submit them in the test or is this really what should be expected?
        assert!(html.contains(r#"<div><a href="https:&#x2F;&#x2F;www.linkedin.com&#x2F;in&#x2F;szabgab&#x2F;">LinkedIn</a></div>"#));
        //assert!(html.contains(r#"<div><a href="https:://www.linkedin.com/in/szabgab/">LinkedIn</a></div>"#));

        eprintln!("{html}");
        //assert_eq!(html, "");
        assert!(html.contains(
            r#"<div><ul>
<li>text</li>
<li>more</li>
<li><a href="https://meet-os.com/">link</a></li>
<li>&lt;b&gt;bold&lt;/b&gt;</li>
<li>&lt;a href=&quot;https://meet-os.com/&quot;&gt;bad link&lt;/a&gt;</li>
</ul>
</div>"#
        ));
    });
}
