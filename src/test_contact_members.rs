use crate::test_helpers::{setup_many, setup_many_users, OWNER_EMAIL, USER_EMAIL};
use crate::test_lib::{
    check_html, check_not_the_owner, check_unprocessable, params, run_inprocess,
};
use rocket::http::{ContentType, Status};

#[test]
fn contact_members_get_user_without_gid() {
    run_inprocess(|email_folder, client| {
        setup_many_users(&client, &email_folder);

        let res = client
            .get("/contact-members")
            .private_cookie(("meet-os", OWNER_EMAIL))
            .dispatch();

        assert_eq!(res.status(), Status::NotFound);
        let html = res.into_string().unwrap();

        check_html(&html, "title", "404 Not Found");
        check_html(&html, "h1", "404 Not Found");
        check_html(&html, "#message", "404 Not Found");
    });
}

#[test]
fn contact_members_get_user_with_invalid_gid() {
    run_inprocess(|email_folder, client| {
        setup_many_users(&client, &email_folder);

        let res = client
            .get("/contact-members?gid=1")
            .private_cookie(("meet-os", OWNER_EMAIL))
            .dispatch();

        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();

        check_html(&html, "title", "No such group");
        check_html(&html, "h1", "No such group");
        check_html(&html, "#message", "Group <b>1</b> does not exist");
    });
}

#[test]
fn contact_members_get_owner_with_gid() {
    run_inprocess(|email_folder, client| {
        setup_many(&client, &email_folder);

        let res = client
            .get("/contact-members?gid=1")
            .private_cookie(("meet-os", OWNER_EMAIL))
            .dispatch();

        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();

        check_html(&html, "title", "Contact members of the 'First Group' group");
        check_html(&html, "h1", "Contact members of the 'First Group' group");
        assert!(
            html.contains(r#"<form method="POST" action="/contact-members" id="contact-members">"#)
        );
        assert!(html.contains(r#"<input type="hidden" name="gid" value="1">"#));
    });
}

#[test]
fn contact_members_get_user_not_owner() {
    run_inprocess(|email_folder, client| {
        setup_many(&client, &email_folder);

        let res = client
            .get("/contact-members?gid=1")
            .private_cookie(("meet-os", USER_EMAIL))
            .dispatch();
        check_not_the_owner(res);
    });
}

// TODO contact_members_get_user_with_gid() {
// TODO contact_members_get_admin_with_gid() {

#[test]
fn contact_members_post_user_without_gid() {
    run_inprocess(|email_folder, client| {
        setup_many(&client, &email_folder);

        let res = client
            .post("/contact-members")
            .private_cookie(("meet-os", OWNER_EMAIL))
            .header(ContentType::Form)
            .dispatch();
        check_unprocessable(res);
    });
}

#[test]
fn contact_members_post_user_with_all() {
    run_inprocess(|email_folder, client| {
        setup_many(&client, &email_folder);

        let res = client
            .post("/contact-members")
            .private_cookie(("meet-os", OWNER_EMAIL))
            .body(params!([
                ("gid", "1"),
                ("subject", "Test subject line"),
                ("content", "Test content"),
            ]))
            .header(ContentType::Form)
            .dispatch();

        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();

        check_html(&html, "title", "Message sent");
        check_html(&html, "h1", "Message sent");
        check_html(&html, "#message", "Message sent");
        // TODO read email file
        // TODO check who was this message sent to
    });
}

#[test]
fn contact_members_post_user_subject_too_short() {
    run_inprocess(|email_folder, client| {
        setup_many(&client, &email_folder);

        let res = client
            .post("/contact-members")
            .private_cookie(("meet-os", OWNER_EMAIL))
            .body(params!([
                ("gid", "1"),
                ("subject", "Test"),
                ("content", "Test content"),
            ]))
            .header(ContentType::Form)
            .dispatch();

        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();

        check_html(&html, "title", "Too short a subject");
        check_html(&html, "h1", "Too short a subject");
        check_html(
            &html,
            "#message",
            r#"Minimal subject length 5 Current subject len: 4"#,
        );
    });
}

#[test]
fn contact_members_post_user_who_is_not_the_owner() {
    run_inprocess(|email_folder, client| {
        setup_many(&client, &email_folder);

        let res = client
            .post("/contact-members")
            .private_cookie(("meet-os", USER_EMAIL))
            .body(params!([
                ("gid", "1"),
                ("subject", "Test subject line"),
                ("content", "Test content"),
            ]))
            .header(ContentType::Form)
            .dispatch();
        check_not_the_owner(res);
    });
}
