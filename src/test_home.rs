use crate::test_lib::{check_guest_menu, check_html, setup_all, TestRunner};
use rocket::http::Status;

#[test]
fn main_page_empty_db() {
    let tr = TestRunner::new();

    let res = tr.client.get("/").dispatch();
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
}

#[test]
fn main_page_with_data() {
    let tr = TestRunner::new();

    setup_all(&tr.client, &tr.email_folder);

    let res = tr.client.get("/").dispatch();
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
}

#[test]
fn main_page_with_import() {
    let tr = TestRunner::from("t0.sql");

    let res = tr.client.get("/").dispatch();
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

    assert!(html.contains(r#"<li><a href="/event/6">First event new3 name</a></li>"#));
    assert!(html.contains(r#"<li><a href="/event/7">Intro to Meet-OS 🎉  </a></li>"#));

    assert!(html.contains(r#"<li><a href="/group/2">Group of  Foo1</a></li>"#));
    assert!(html.contains(r#"<li><a href="/group/5">new group in new style</a></li>"#));
    assert!(html.contains(r#"<li><a href="/group/1">Gabor Maven</a></li>"#));
    assert!(html.contains(r#"<li><a href="/group/4">Group of the Admin</a></li>"#));
    assert!(html.contains(r#"<li><a href="/group/3">Send email to owner</a></li>"#));
}
