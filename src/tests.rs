use rocket::http::Status;
use rocket::local::blocking::Client;

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
    assert!(body.contains("<h1>Web development with Rocket</h1>"));
}

#[test]
fn group_page() {
    let client = Client::tracked(super::rocket()).unwrap();
    let response = client.get("/group/1").dispatch();

    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().unwrap();
    assert!(body.contains("<h1>Rust Maven</h1>"));
}

#[test]
fn register_page() {
    let client = Client::tracked(super::rocket()).unwrap();
    let response = client.get("/register").dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert!(response
        .into_string()
        .unwrap()
        .contains("<h2>Register</h2>"));
}

// Web based register user to the web site

// CLI register user (including sending email)
// CLI list users
// CLI create group owned by user X
// CLI list groups
// CLI edit group info
// CLI create event in group
