use crate::test_helpers::{setup_many, setup_many_users};
use crate::test_lib::{check_html, params, run_inprocess};
use rocket::http::{ContentType, Status};

#[test]
fn contact_members_get_guest() {
    run_inprocess(|email_folder, client| {
        let res = client.get("/contact-members").dispatch();

        assert_eq!(res.status(), Status::Unauthorized);
        let html = res.into_string().unwrap();

        check_html(&html, "title", "Not logged in");
        //assert_eq!(html, "");
        // check_html(&html, "title", "Register");
        // check_html(&html, "h1", "Register");
        // assert!(html.contains(r#"<form method="POST" action="/register">"#));
    });
}

#[test]
fn contact_members_get_user_without_gid() {
    run_inprocess(|email_folder, client| {
        setup_many_users(&client, &email_folder);
        let foo_mail = "foo@meet-os.com";

        let res = client
            .get("/contact-members")
            .private_cookie(("meet-os", foo_mail))
            .dispatch();

        assert_eq!(res.status(), Status::NotFound);
        let html = res.into_string().unwrap();

        //assert_eq!(html, "");
        check_html(&html, "title", "404 Not Found");
        check_html(&html, "h1", "404: Not Found");
    });
}

#[test]
fn contact_members_get_user_with_invalid_gid() {
    run_inprocess(|email_folder, client| {
        setup_many_users(&client, &email_folder);
        let foo_mail = "foo@meet-os.com";

        let res = client
            .get("/contact-members?gid=1")
            .private_cookie(("meet-os", foo_mail))
            .dispatch();

        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();

        //assert_eq!(html, "");
        check_html(&html, "title", "No such group");
        check_html(&html, "h1", "No such group");
        assert!(html.contains("Group <b>1</b> does not exist"));
    });
}

#[test]
fn contact_members_get_owner_with_gid() {
    run_inprocess(|email_folder, client| {
        setup_many(&client, &email_folder);

        let foo_mail = "foo@meet-os.com";

        let res = client
            .get("/contact-members?gid=1")
            .private_cookie(("meet-os", foo_mail))
            .dispatch();

        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();

        //assert_eq!(html, "");
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

        let foo1_mail = "foo1@meet-os.com";

        let res = client
            .get("/contact-members?gid=1")
            .private_cookie(("meet-os", foo1_mail))
            .dispatch();

        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();

        //assert_eq!(html, "");
        check_html(&html, "title", "Not the owner");
        check_html(&html, "h1", "Not the owner");
    });
}

// TODO contact_members_get_user_with_gid() {
// TODO contact_members_get_admin_with_gid() {

#[test]
fn contact_members_post_guest() {
    run_inprocess(|email_folder, client| {
        let res = client
            .post("/contact-members")
            .header(ContentType::Form)
            .dispatch();

        assert_eq!(res.status(), Status::Unauthorized);
        let html = res.into_string().unwrap();

        check_html(&html, "title", "Not logged in");
    });
}

#[test]
fn contact_members_post_user_without_gid() {
    run_inprocess(|email_folder, client| {
        setup_many(&client, &email_folder);
        let foo_mail = "foo@meet-os.com";

        let res = client
            .post("/contact-members")
            .private_cookie(("meet-os", foo_mail))
            .header(ContentType::Form)
            .dispatch();

        assert_eq!(res.status(), Status::UnprocessableEntity);
        //let html = res.into_string().unwrap();

        //assert_eq!(html, "");
        // check_html(&html, "title", "Register");
        // check_html(&html, "h1", "Register");
        // assert!(html.contains(r#"<form method="POST" action="/register">"#));
    });
}

#[test]
fn contact_members_post_user_with_all() {
    run_inprocess(|email_folder, client| {
        setup_many(&client, &email_folder);
        let foo_mail = "foo@meet-os.com";

        let res = client
            .post("/contact-members")
            .private_cookie(("meet-os", foo_mail))
            .body(params!([
                ("gid", "1"),
                ("subject", "Test subject line"),
                ("content", "Test content"),
            ]))
            .header(ContentType::Form)
            .dispatch();

        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();

        //assert_eq!(html, "");
        check_html(&html, "title", "Message sent");
        check_html(&html, "h1", "Message sent");
        // TODO read email file
        // TODO check who was this message sent to
    });
}
