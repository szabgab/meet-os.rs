use crate::test_lib::{
    check_admin_menu, check_guest_menu, check_html, check_message, check_not_logged_in,
    check_profile_by_guest, check_profile_by_user, check_user_menu, logout, params,
    read_code_from_email, register_and_verify_user, setup_admin, setup_many_users, setup_owner,
    setup_unverified_user, setup_user, TestRunner, ADMIN_EMAIL, ADMIN_NAME, ADMIN_PW, OTHER_NAME,
    OWNER_EMAIL, OWNER_NAME, OWNER_PW, UNVERIFIED_NAME, USER_EMAIL, USER_NAME,
};
use rocket::http::{ContentType, Status};

#[test]
fn protected_pages_as_guest() {
    let tr = TestRunner::new();

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
        "/edit-profile",
        "/profile",
    ] {
        let res = tr.client.get(path).dispatch();
        check_not_logged_in!(res);
    }
}

#[test]
fn protected_post_requests_as_guest() {
    let tr = TestRunner::new();

    for path in [
        "/contact-members",
        "/add-event",
        "/edit-event",
        "/edit-group",
        "/admin/search",
        "/admin/create-group",
    ] {
        let res = tr.client.post(path).header(ContentType::Form).dispatch();

        check_not_logged_in!(res);
    }

    // Create group should fail even if we have the parameters
    let res = tr
        .client
        .post("/admin/create-group")
        .body(params!([
            ("name", "Rust Maven"),
            ("location", ""),
            ("description", ""),
            ("owner", "1"),
        ]))
        .dispatch();
    check_not_logged_in!(res);
}

#[test]
fn register_user() {
    let tr = TestRunner::new();

    let res = tr
        .client
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
    let expected = format!("We sent you an email to <b>{OWNER_EMAIL}</b> Please check your inbox and verify your email address.");
    check_message!(&html, "We sent you an email", &expected);
    check_guest_menu!(&html);

    let (uid, code) = read_code_from_email(&tr.email_folder, "0.txt", "verify-email");

    // Verify the email
    let res = tr
        .client
        .get(format!("/verify-email/{uid}/{code}"))
        .dispatch();
    assert_eq!(res.status(), Status::Ok);

    let html = res.into_string().unwrap();
    check_message!(
        &html,
        "Thank you for registering",
        "Your email was verified."
    );
    check_user_menu!(&html);

    check_profile_by_user!(&tr.client, OWNER_EMAIL, OWNER_NAME);
}

#[test]
fn get_verify_with_non_existent_id() {
    let tr = TestRunner::new();

    let res = tr.client.get(format!("/verify-email/1/abc")).dispatch();
    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    check_message!(&html, "Invalid id", "Invalid id <b>1</b>");
}

#[test]
fn get_verify_email_with_bad_code() {
    let tr = TestRunner::new();

    let res = tr
        .client
        .post(format!("/register"))
        .header(ContentType::Form)
        .body(params!([
            ("name", OWNER_NAME),
            ("email", OWNER_EMAIL),
            ("password", OWNER_PW),
        ]))
        .dispatch();
    assert_eq!(res.status(), Status::Ok);

    let res = tr.client.get(format!("/verify-email/1/abc")).dispatch();
    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    check_message!(&html, "Invalid code", "Invalid code <b>abc</b>");
}

#[test]
fn post_register_duplicate_email() {
    let tr = TestRunner::new();

    let res = tr
        .client
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
    check_guest_menu!(&html);
    let expected = format!("We sent you an email to <b>{OWNER_EMAIL}</b> Please check your inbox and verify your email address.");
    check_message!(&html, "We sent you an email", &expected);

    let res = tr
        .client
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
    check_html!(&html, "title", "Registration failed");
    check_guest_menu!(&html);
}

#[test]
fn post_login_regular_user() {
    let tr = TestRunner::new();

    register_and_verify_user(
        &tr.client,
        OWNER_NAME,
        OWNER_EMAIL,
        OWNER_PW,
        &tr.email_folder,
    );

    check_profile_by_user!(&tr.client, &OWNER_EMAIL, OWNER_NAME);

    let res = tr
        .client
        .post("/login")
        .header(ContentType::Form)
        .body(params!([("email", OWNER_EMAIL), ("password", OWNER_PW)]))
        .dispatch();
    assert_eq!(res.status(), Status::Ok);

    let html = res.into_string().unwrap();

    //assert_eq!(html, "");
    check_html!(&html, "title", "Welcome back");
    check_user_menu!(&html);

    // Access the profile with the cookie
    check_profile_by_user!(&tr.client, &OWNER_EMAIL, OWNER_NAME);

    // TODO: logout requires a logged in user
    //let res = client.get("/logout").dispatch();
    // assert_eq!(res.status(), Status::Unauthorized);
    //let html = res.into_string().unwrap();
    // check_html!(&html, "title", "Not logged in");
    // assert!(html.contains("You are not logged in"));

    // logout
    let res = tr
        .client
        .get("/logout")
        .private_cookie(("meet-os", OWNER_EMAIL))
        .dispatch();
    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();

    check_html!(&html, "title", "Logged out");
    check_html!(&html, "h1", "Logged out");
    check_guest_menu!(&html);

    // TODO as the login information is only saved in the client-side cookie, if someone has the cookie they can
    // use it even the user has clicked on /logout and we have asked the browser to remove the cookie.
    // If we want to make sure that the user cannot access the system any more we'll have to manage the login information
    // on the server side.
}

#[test]
fn post_login_admin() {
    let tr = TestRunner::new();

    setup_admin(&tr.client, &tr.email_folder);
    logout(&tr.client);

    // login as admin
    let res = tr
        .client
        .post("/login")
        .header(ContentType::Form)
        .body(params!([("email", ADMIN_EMAIL), ("password", ADMIN_PW)]))
        .dispatch();
    assert_eq!(res.status(), Status::Ok);

    let html = res.into_string().unwrap();
    check_html!(&html, "title", "Welcome back");
    check_admin_menu!(&html);

    check_profile_by_user!(&tr.client, &ADMIN_EMAIL, ADMIN_NAME);

    // logout
    let res = tr.client.get("/logout").dispatch();

    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    check_html!(&html, "title", "Logged out");
    check_html!(&html, "h1", "Logged out");
    check_profile_by_guest!(&tr.client);
}

#[test]
fn post_register_with_invalid_email_address() {
    let tr = TestRunner::new();

    //"name=Foo Bar&email=meet-os.com&password=123456"
    let res = tr
        .client
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
    check_message!(
        &html,
        "Invalid email address",
        "Invalid email address <b>meet-os.com</b> Please try again"
    );
}

#[test]
fn post_register_with_too_long_username() {
    let tr = TestRunner::new();

    let res = tr.client
            .post("/register")
            .header(ContentType::Form)
            .body("name=QWERTYUIOPASDFGHJKLZXCVBNM QWERTYUIOPASDFGHJKLZXCVBNM&email=long@meet-os.com&password=123456")
            .dispatch();
    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    check_message!(
        &html,
        "Name is too long",
        "Name is too long. Max 50 while the current name is 53 long. Please try again."
    );
}

#[test]
fn post_register_with_bad_email_address() {
    let tr = TestRunner::new();

    // register new user
    let res = tr
        .client
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
    check_message!(
        &html,
        "Invalid email address",
        "Invalid email address <b>meet-os.com</b> Please try again"
    );
    check_guest_menu!(&html);
}

// TODO try to login with an email address that was not registered

#[test]
fn post_login_with_unregistered_email() {
    let tr = TestRunner::new();

    let res = tr
        .client
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
    check_message!(
        &html,
        "No such user",
        "No user with address <b>other@meet-os.com</b>. Please try again"
    );
    check_guest_menu!(&html);
}

#[test]
fn post_login_with_bad_password() {
    let tr = TestRunner::new();

    setup_user(&tr.client, &tr.email_folder);
    logout(&tr.client);

    let res = tr
        .client
        .post("/login")
        .header(ContentType::Form)
        .body(params!([("email", USER_EMAIL), ("password", OWNER_PW)]))
        .dispatch();

    assert_eq!(res.status(), Status::Ok);
    assert!(res.headers().get_one("set-cookie").is_none());
    let html = res.into_string().unwrap();

    check_html!(&html, "title", "Invalid password");
    check_html!(&html, "h1", "Invalid password");
    check_guest_menu!(&html);
}

#[test]
fn post_login_with_unverified_email() {
    let tr = TestRunner::new();
    let res = tr
        .client
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

    let res = tr
        .client
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
    check_message!(
        &html,
        "Unverified email",
        "Email must be verified before login."
    );
    check_guest_menu!(&html);
}

#[test]
fn post_login_with_invalid_email() {
    let tr = TestRunner::new();

    // no actual user needed in the system for this to work
    let res = tr
        .client
        .post("/login")
        .header(ContentType::Form)
        .body(params!([("email", "meet-os.com"), ("password", "123456")]))
        .dispatch();
    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    //assert_eq!(html, "");
    check_html!(&html, "title", "Invalid email address");
    check_html!(&html, "h1", "Invalid email address");
}

#[test]
fn post_register_with_short_password() {
    let tr = TestRunner::new();

    let res = tr
        .client
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
    check_message!(
        &html,
        "Invalid password",
        "The password must be at least 6 characters long."
    );
    check_guest_menu!(&html);
}

#[test]
fn get_edit_profile_user() {
    let tr = TestRunner::new();

    setup_owner(&tr.client, &tr.email_folder);

    let res = tr
        .client
        .get("/edit-profile")
        .private_cookie(("meet-os", OWNER_EMAIL))
        .dispatch();

    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();

    //assert_eq!(html, "");
    check_html!(&html, "title", "Edit Profile");
    check_html!(&html, "h1", "Edit Profile");
    assert!(html.contains(r#"<form method="POST" action="/edit-profile">"#));
}

// edit_profile_get_unverified_user should fail

#[test]
fn get_register_guest() {
    let tr = TestRunner::new();

    let res = tr.client.get("/register").dispatch();

    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();

    //assert_eq!(html, "");
    check_html!(&html, "title", "Register");
    check_html!(&html, "h1", "Register");
    assert!(html.contains(r#"<form method="POST" action="/register">"#));
}

// register_get_user should fail
//
//
#[test]
fn get_users_list_users_empty_db_guest() {
    let tr = TestRunner::new();

    let res = tr.client.get("/users").dispatch();

    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();

    //assert_eq!(html, "");
    check_html!(&html, "title", "List Users");
    check_html!(&html, "h1", "List Users");
    assert!(html.contains(r#"No users"#));
}

#[test]
fn get_users_list_users_many_users_guest() {
    let tr = TestRunner::new();

    setup_many_users(&tr.client, &tr.email_folder);

    let res = tr.client.get("/users").dispatch();

    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();

    // assert_eq!(html, "");
    check_html!(&html, "title", "List Users");
    check_html!(&html, "h1", "List Users");
    assert!(!html.contains(r#"No users"#));

    let expected = format!(r#"<li><a href="/user/2">{OWNER_NAME}</a></li>"#);
    assert!(html.contains(&expected));
    let expected = format!(r#"<li><a href="/user/3">{USER_NAME}</a></li>"#);
    assert!(html.contains(&expected));
    let expected = format!(r#"<li><a href="/user/4">{OTHER_NAME}</a></li>"#);
    assert!(html.contains(&expected));
    let expected = format!(r#"<li><a href="/user/1">{ADMIN_NAME}</a></li>"#);
    assert!(html.contains(&expected));
}

#[test]
fn user_id_that_does_not_exist() {
    let tr = TestRunner::new();

    let res = tr.client.get("/user/42").dispatch();

    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();

    check_message!(
        &html,
        "User not found",
        r#"There is no user with id <b>42</b>."#
    );
}

#[test]
fn get_user_page() {
    let tr = TestRunner::new();
    setup_admin(&tr.client, &tr.email_folder);
    setup_owner(&tr.client, &tr.email_folder);

    let res = tr.client.get("/user/2").dispatch();

    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();

    //assert_eq!(html, "");
    check_html!(&html, "title", OWNER_NAME);
    check_html!(&html, "h1", OWNER_NAME);
    let expected = format!(r#"<td>Name:</td><td>{OWNER_NAME}</td>"#);
    assert!(html.contains(&expected));
    assert!(html.contains(r#"<td>No GitHub provided.</td"#));
    assert!(html.contains(r#"<td>No GitLab provided.</td"#));
    assert!(html.contains(r#"<td>No LinkedIn provided.</td>"#));
}

#[test]
fn unverified_user_page_by_guest() {
    let tr = TestRunner::new();

    setup_unverified_user(&tr.client, &tr.email_folder);
    logout(&tr.client);

    let res = tr.client.get("/user/1").dispatch();

    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    check_message!(
        &html,
        "Unverified user",
        "This user has not verified the email address yet."
    );
    assert!(!html.contains(UNVERIFIED_NAME));

    // TODO check by logged in user, owner, and admin as well
}

#[test]
fn unverified_user_on_user_page_by_guest() {
    let tr = TestRunner::new();

    setup_admin(&tr.client, &tr.email_folder);
    setup_user(&tr.client, &tr.email_folder);
    setup_unverified_user(&tr.client, &tr.email_folder);
    logout(&tr.client);

    let res = tr.client.get("/users").dispatch();

    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    //assert_eq!(html, "");
    check_html!(&html, "title", "List Users");
    check_html!(&html, "h1", "List Users");
    assert!(html.contains(r#"<a href="/user/1">Site Manager</a>"#));
    let expected = format!(r#"<a href="/user/2">{USER_NAME}</a></li>"#);
    assert!(html.contains(&expected));
    assert!(!html.contains(UNVERIFIED_NAME));

    // TODO check by logged in user, owner, and admin as well
}

#[test]
fn post_edit_profile_failures() {
    let tr = TestRunner::new();

    setup_owner(&tr.client, &tr.email_folder);

    // edit profile page invalid github account
    let res = tr
        .client
        .post("/edit-profile")
        .private_cookie(("meet-os", OWNER_EMAIL))
        .header(ContentType::Form)
        .body("name=XX&github=szabgab*&gitlab=&linkedin")
        .dispatch();
    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    check_message!(
        &html,
        "Invalid GitHub username",
        r#"The GitHub username `szabgab*` is not valid."#
    );

    // edit profile page invalid gitlab account
    let res = tr
        .client
        .post("/edit-profile")
        .private_cookie(("meet-os", OWNER_EMAIL))
        .header(ContentType::Form)
        .body("name=XX&github=&gitlab=foo*bar&linkedin=")
        .dispatch();
    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    check_message!(
        &html,
        "Invalid GitLab username",
        r#"The GitLab username `foo*bar` is not valid."#
    );

    // edit profile invalid linkein
    let res = tr
        .client
        .post("/edit-profile")
        .private_cookie(("meet-os", OWNER_EMAIL))
        .header(ContentType::Form)
        .body("name=XX&github=&gitlab=&linkedin=szabgab")
        .dispatch();
    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    check_message!(
        &html,
        "Invalid LinkedIn profile link",
        r#"The LinkedIn profile link `szabgab` is not valid."#
    );

    // edit profile name too long
    let res = tr
        .client
        .post("/edit-profile")
        .private_cookie(("meet-os", OWNER_EMAIL))
        .header(ContentType::Form)
        .body(
            "name=QWERTYUIOPASDFGHJKLZXCVBNM QWERTYUIOPASDFGHJKLZXCVBNM&github=&gitlab=&linkedin=",
        )
        .dispatch();
    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    check_message!(
        &html,
        "Name is too long",
        "Name is too long. Max 50 while the current name is 53 long. Please try again."
    );

    // edit profile name contains invalid character
    let res = tr
        .client
        .post("/edit-profile")
        .private_cookie(("meet-os", OWNER_EMAIL))
        .header(ContentType::Form)
        .body("name=é&github=&gitlab=&linkedin=")
        .dispatch();
    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    check_message!(
        &html,
        "Invalid character",
        r#"The name 'é' contains a character that we currently don't accept. Use Latin letters for now and comment on <a href="https://github.com/szabgab/meet-os.rs/issues/38">this issue</a> where this topic is discussed."#
    );

    // TODO test the validation of the other fields as well!
}

#[test]
fn post_edit_profile_works() {
    let tr = TestRunner::new();

    setup_owner(&tr.client, &tr.email_folder);

    // verify that if we submit html tags to the about field, those are properly escaped in the result
    let res = tr.client
            .post("/edit-profile")
            .private_cookie(("meet-os", OWNER_EMAIL))
            .header(ContentType::Form)
            .body("name= Lord Voldemort &github= alfa &gitlab= beta &linkedin=  https://www.linkedin.com/in/szabgab/  ")
            .dispatch();
    // &about=* text\n* more\n* [link](https://meet-os.com/)\n* <b>bold</b>\n* <a href=\"https://meet-os.com/\">bad link</a>\n"

    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    check_message!(
        &html,
        "Profile updated",
        r#"Check out the <a href="/profile">profile</a> and how others see it <a href="/user/1">Lord Voldemort</a>"#
    );

    // Check updated profile
    let res = tr
        .client
        .get("/profile")
        .private_cookie(("meet-os", OWNER_EMAIL))
        .dispatch();

    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    check_html!(&html, "title", "Profile");
    assert!(html.contains(r#"<h1 class="title is-3">Lord Voldemort</h1>"#));
    assert!(html.contains(r#"<div><a href="https://github.com/alfa">GitHub</a></div>"#));
    assert!(html.contains(r#"<div><a href="https://gitlab.com/beta">GitLab</a></div>"#));

    // TODO: do we need to escape the characters when we submit them in the test or is this really what should be expected?
    assert!(html.contains(r#"<div><a href="https:&#x2F;&#x2F;www.linkedin.com&#x2F;in&#x2F;szabgab&#x2F;">LinkedIn</a></div>"#));
    //assert!(html.contains(r#"<div><a href="https:://www.linkedin.com/in/szabgab/">LinkedIn</a></div>"#));

    eprintln!("{html}");
    //         assert!(html.contains(
    //             r#"<div><ul>
    // <li>text</li>
    // <li>more</li>
    // <li><a href="https://meet-os.com/">link</a></li>
    // <li>&lt;b&gt;bold&lt;/b&gt;</li>
    // <li>&lt;a href=&quot;https://meet-os.com/&quot;&gt;bad link&lt;/a&gt;</li>
    // </ul>
    // </div>"#
    //         ));
}

#[test]
fn post_register_with_invalid_username() {
    let tr = TestRunner::new();

    let res = tr
        .client
        .post("/register")
        .header(ContentType::Form)
        .body("name=é&email=long@meet-os.com&password=123456")
        .dispatch();
    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    check_message!(
        &html,
        "Invalid character",
        r#"The name 'é' contains a character that we currently don't accept. Use Latin letters for now and comment on <a href="https://github.com/szabgab/meet-os.rs/issues/38">this issue</a> where this topic is discussed."#
    );
}
