use crate::test_lib::{
    check_html, check_message, check_not_the_owner, check_unprocessable, params, TestRunner,
};
use rocket::http::{ContentType, Status};

#[test]
fn contact_members_get_user_without_gid() {
    let tr = TestRunner::new();

    tr.setup_many_users();
    tr.login_owner();

    let res = tr.client.get("/contact-members").dispatch();

    assert_eq!(res.status(), Status::NotFound);
    let html = res.into_string().unwrap();

    check_message!(&html, "404 Not Found", "404 Not Found");
}

#[test]
fn contact_members_get_user_with_invalid_gid() {
    let tr = TestRunner::new();

    tr.setup_many_users();
    tr.login_owner();

    let res = tr.client.get("/contact-members?gid=1").dispatch();

    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    check_message!(&html, "No such group", "Group <b>1</b> does not exist");
}

#[test]
fn contact_members_get_owner_with_gid() {
    let tr = TestRunner::new();

    tr.setup_all();
    tr.login_owner();

    let res = tr.client.get("/contact-members?gid=1").dispatch();

    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();

    check_html!(&html, "title", "Contact members of the 'First Group' group");
    check_html!(&html, "h1", "Contact members of the 'First Group' group");
    assert!(html.contains(r#"<form method="POST" action="/contact-members" id="contact-members">"#));
    assert!(html.contains(r#"<input type="hidden" name="gid" value="1">"#));
}

#[test]
fn contact_members_get_user_not_owner() {
    let tr = TestRunner::new();
    tr.setup_for_groups();
    tr.login_user();

    let res = tr.client.get("/contact-members?gid=1").dispatch();
    check_not_the_owner!(res);
}

// TODO contact_members_get_user_with_gid() {
// TODO contact_members_get_admin_with_gid() {

#[test]
fn contact_members_post_user_without_gid() {
    let tr = TestRunner::new();

    tr.setup_all();
    tr.login_owner();

    let res = tr
        .client
        .post("/contact-members")
        .header(ContentType::Form)
        .dispatch();
    check_unprocessable!(res);
}

#[test]
fn contact_members_post_user_with_all() {
    let tr = TestRunner::new();

    tr.setup_all();
    tr.login_owner();

    let res = tr
        .client
        .post("/contact-members")
        .body(params!([
            ("gid", "1"),
            ("subject", "Test subject line"),
            ("content", "Test content"),
        ]))
        .header(ContentType::Form)
        .dispatch();

    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();

    check_message!(&html, "Message sent", "Message sent");
    // TODO read email file
    // TODO check who was this message sent to
}

#[test]
fn contact_members_post_user_subject_too_short() {
    let tr = TestRunner::new();

    tr.setup_all();
    tr.login_owner();

    let res = tr
        .client
        .post("/contact-members")
        .body(params!([
            ("gid", "1"),
            ("subject", "Test"),
            ("content", "Test content"),
        ]))
        .header(ContentType::Form)
        .dispatch();

    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    check_message!(
        &html,
        "Too short a subject",
        r#"Minimal subject length 5 Current subject len: 4"#
    );
}

#[test]
fn contact_members_post_user_who_is_not_the_owner() {
    let tr = TestRunner::new();

    tr.setup_for_groups();
    tr.login_user();

    let res = tr
        .client
        .post("/contact-members")
        .body(params!([
            ("gid", "1"),
            ("subject", "Test subject line"),
            ("content", "Test content"),
        ]))
        .header(ContentType::Form)
        .dispatch();
    check_not_the_owner!(res);
}
