use crate::test_helpers::setup_all;
use crate::test_lib::{check_guest_menu, check_html, run_inprocess};
use rocket::http::Status;

#[test]
fn main_page_empty_db() {
    run_inprocess(|email_folder, client| {
        let res = client.get("/").dispatch();
        assert_eq!(res.status(), Status::Ok);
        assert_eq!(
            res.headers().get_one("Content-Type").unwrap(),
            "text/html; charset=utf-8"
        );

        let html = res.into_string().unwrap();
        check_html!(&html, "title", "Meet-OS");
        check_html!(&html, "h1", "Welcome to the Meet-OS meeting server");
        assert!(!html.contains(r#"<h2 class="title is-4">Events</h2>"#));
        assert!(!html.contains(r#"<h2 class="title is-4">Groups</h2>"#));
        check_guest_menu!(&html);
    });
}

#[test]
fn main_page_with_data() {
    run_inprocess(|email_folder, client| {
        setup_all(&client, &email_folder);

        let res = client.get("/").dispatch();
        assert_eq!(res.status(), Status::Ok);
        assert_eq!(
            res.headers().get_one("Content-Type").unwrap(),
            "text/html; charset=utf-8"
        );

        let html = res.into_string().unwrap();
        check_html!(&html, "title", "Meet-OS");
        check_html!(&html, "h1", "Welcome to the Meet-OS meeting server");
        assert!(html.contains(r#"<h2 class="title is-4">Events</h2>"#));
        assert!(html.contains(r#"<h2 class="title is-4">Groups</h2>"#));
        check_guest_menu!(&html);

        assert!(html.contains(r#"<li><a href="/event/1">First event</a></li>"#));
        assert!(html.contains(r#"<li><a href="/event/2">Second event</a></li>"#));
        assert!(html.contains(r#"<li><a href="/event/3">Third event</a></li>"#));

        assert!(html.contains(r#"<li><a href="/group/1">First Group</a></li>"#));
        assert!(html.contains(r#"<li><a href="/group/2">Second Group</a></li>"#));
        assert!(html.contains(r#"<li><a href="/group/3">Third Group</a></li>"#));
    });
}
