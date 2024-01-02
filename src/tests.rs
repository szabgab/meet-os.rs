use rocket::http::Status;
use rocket::local::blocking::Client;

#[test]
fn hello_world() {
    let client = Client::tracked(super::rocket()).unwrap();
    let response = client.get("/").dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string(), Some("Hello, world!".into()));
}

// Web based register user to the web site

// CLI register user (including sending email)
// CLI list users
// CLI create group owned by user X
// CLI list groups
// CLI edit group info
// CLI create event in group
