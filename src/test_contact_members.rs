use crate::test_helpers::setup_many_users;
use crate::test_lib::{check_html, run_inprocess};
use rocket::http::Status;

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

// contact_members_get_owner_with_gid() {
// fn contact_members_get_user_with_gid() {
// fn contact_members_get_admin_with_gid() {
