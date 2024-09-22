use crate::test_lib::{
    check_html, check_message, check_only_guest, check_profile_by_user, check_user_menu, params,
    read_code_from_email, TestRunner, UNVERIFIED_EMAIL, UNVERIFIED_NAME, USER_EMAIL,
};

use rocket::http::{ContentType, Status};

#[test]
fn get_resend_email_verification_guest() {
    let tr = TestRunner::new();

    let res = tr.client.get("/resend-email-verification-code").dispatch();
    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    check_html!(&html, "title", "Resend code for email verification");
    check_html!(&html, "h1", "Resend code for email verification");
    assert!(html.contains(r#"<form method="POST" action="/resend-email-verification-code">"#));
    assert!(html.contains(
        r#"Email: <input name="email" class="input" id="email" type="email" placeholder="Email">"#
    ));
    assert!(html.contains(r#"<input type="submit" class="button" value="Send code">"#));
}

#[test]
fn get_resend_email_verification_logged_in_user() {
    let tr = TestRunner::new();

    tr.setup_user();
    let res = tr.client.get("/resend-email-verification-code").dispatch();
    check_only_guest!(res);
}

#[test]
fn post_resend_email_verification_logged_in_user() {
    let tr = TestRunner::new();

    tr.setup_user();
    let res = tr
        .client
        .post("/resend-email-verification-code")
        .header(ContentType::Form)
        .body(params!([("email", "any@meet-os.com"),]))
        .dispatch();
    check_only_guest!(res);
}

#[test]
fn post_resend_email_verification_verified_email() {
    let tr = TestRunner::new();

    tr.setup_user();
    tr.logout();

    let res = tr
        .client
        .post("/resend-email-verification-code")
        .header(ContentType::Form)
        .body(params!([("email", USER_EMAIL),]))
        .dispatch();
    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    check_message!(
        &html,
        "Already verified",
        r#"This email address is already verified. Try to <a href="/login">login</a>."#
    );
}

#[test]
fn post_resend_email_verification_unverified_email() {
    let tr = TestRunner::new();

    tr.setup_unverified_user();
    tr.logout();
    tr.clean_emails();

    let res = tr
        .client
        .post("/resend-email-verification-code")
        .header(ContentType::Form)
        .body(params!([("email", UNVERIFIED_EMAIL)]))
        .dispatch();
    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    check_message!(
        &html,
        "We sent you an email",
        r#"We sent you an email to <b>unverified@meet-os.com</b> Please click on the link to reset your password."#
    );

    let (uid, code) = read_code_from_email(&tr.email_folder, "0.txt", "verify-email");

    assert_eq!(uid, 1);
    //assert_eq!(code, "");

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

    check_profile_by_user!(&tr.client, UNVERIFIED_EMAIL, UNVERIFIED_NAME);
}
