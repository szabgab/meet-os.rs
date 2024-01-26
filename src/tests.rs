use rocket::http::Status;
use rocket::local::blocking::Client;

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
