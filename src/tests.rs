use rocket::http::{ContentType, Status};
use rocket::local::blocking::Client;
use tempdir::TempDir;

#[test]
fn index_page() {
    let client = Client::tracked(super::rocket()).unwrap();
    let response = client.get("/").dispatch();

    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().unwrap();
    assert!(body.contains("Welcome to the Rust meeting server"));
    assert!(body.contains("Web development with Rocket"));
    assert!(body.contains("<h2>Events</h2>"));
    assert!(body.contains(r#"<a href="/event/1">Web development with Rocket</a>"#));
    assert!(body.contains("<h2>Groups</h2>"));
    assert!(body.contains(r#"<a href="/group/1">Rust Maven</a>"#));
}

#[test]
fn event_page() {
    let client = Client::tracked(super::rocket()).unwrap();
    let response = client.get("/event/1").dispatch();

    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().unwrap();
    assert!(body.contains(r#"<h1 class="title is-3">Web development with Rocket</h1>"#));
    assert!(body.contains(r#"Organized by <a href="/group/1">Rust Maven</a>."#));
    assert!(body.contains(r#"<div><b>Date</b>: 2024-02-04T17:00:00 UTC</div>"#));
    assert!(body.contains(r#"<div><b>Location</b>: Virtual</div>"#));
}

#[test]
fn group_page() {
    let client = Client::tracked(super::rocket()).unwrap();
    let response = client.get("/group/1").dispatch();

    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().unwrap();
    assert!(body.contains(r#"<h1 class="title is-3">Rust Maven</h1>"#));
    assert!(body.contains(
        r#"<li><a href="/event/1">2024-02-04T17:00:00 - Web development with Rocket</a></li>"#
    ));
    assert!(body.contains(r#"<div><b>Location</b>: Virtual</div>"#));
}

#[test]
fn register_page() {
    let client = Client::tracked(super::rocket()).unwrap();
    let response = client.get("/register").dispatch();

    assert_eq!(response.status(), Status::Ok);
    let html = response.into_string().unwrap();
    assert!(html.contains("<title>Register</title>"));
    assert!(html.contains(r#"Name: <input name="name" id="name" type="text">"#));
    assert!(html.contains(r#"Email: <input name="email" id="email" type="email">"#));
}

#[test]
fn register_with_bad_email_address() {
    let tmp_dir = TempDir::new("counter").unwrap();
    println!("tmp_dir: {:?}", tmp_dir);
    std::env::set_var("DATABASE_PATH", tmp_dir.path());

    std::env::set_var("TEST_APP", "1");
    let client = Client::tracked(super::rocket()).unwrap();
    let response = client
        .post("/register")
        .header(ContentType::Form)
        .body("name=Foo Bar&email=meet-os.com")
        .dispatch();

    assert_eq!(response.status(), Status::Ok); // TODO should this stay 200 OK?
    let html = response.into_string().unwrap();
    // TODO make these tests parse the HTML and verify the extracted title tag!
    //assert_eq!(html, "");
    assert!(html.contains("<title>Invalid email address</title>"));
    assert!(html.contains("Invalid email address <b>meet-os.com</b> Please try again"));
}

#[test]
fn register_user() {
    let tmp_dir = TempDir::new("counter").unwrap();
    println!("tmp_dir: {:?}", tmp_dir);
    std::env::set_var("DATABASE_PATH", tmp_dir.path());

    std::env::set_var("TEST_APP", "1");
    let client = Client::tracked(super::rocket()).unwrap();
    let response = client
        .post("/register")
        .header(ContentType::Form)
        .body("name=Foo Bar&email=foo@meet-os.com")
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
    let html = response.into_string().unwrap();
    assert!(html.contains("<title>We sent you an email</title>"));
    assert!(html.contains(r#"We sent you an email to <b>foo@meet-os.com</b> Please check your inbox and verify your email address."#));

    let response = client.get("/verify/abc").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let html = response.into_string().unwrap();
    assert!(html.contains("<title>Thank you for registering</title>"));
    assert!(html.contains("Your email was verified."));
}

#[test]
fn about_page() {
    let client = Client::tracked(super::rocket()).unwrap();
    let response = client.get("/about").dispatch();

    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().unwrap();
    assert!(body.contains(r#"<title>About Meet-OS</title>"#));
}

#[test]
fn soc_page() {
    let client = Client::tracked(super::rocket()).unwrap();
    let response = client.get("/soc").dispatch();

    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().unwrap();
    assert!(body.contains(r#"<title>Standard of Conduct</title>"#));
}

#[test]
fn privacy_page() {
    let client = Client::tracked(super::rocket()).unwrap();
    let response = client.get("/privacy").dispatch();

    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().unwrap();
    assert!(body.contains(r#"<title>Privacy Policy</title>"#));
}
