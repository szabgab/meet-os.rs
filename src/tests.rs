use rocket::http::Status;
use rocket::local::blocking::Client;

#[test]
fn index_page() {
    let client = Client::tracked(super::rocket()).unwrap();
    let response = client.get("/").dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert!(response
        .into_string()
        .unwrap()
        .contains("Welcome to the Rust meeting server"));
}

// Web based register user to the web site

// CLI register user (including sending email)
// CLI list users
// CLI create group owned by user X
// CLI list groups
// CLI edit group info
// CLI create event in group
