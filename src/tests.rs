use rocket::http::{ContentType, Status};
use rocket::local::blocking::Client;

#[test]
fn home() {
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
    let tmp_dir = tempfile::tempdir().unwrap();
    println!("tmp_dir: {:?}", tmp_dir);
    std::env::set_var("DATABASE_PATH", tmp_dir.path().join("db"));

    std::env::set_var("EMAIL_FILE", tmp_dir.path().join("email.txt"));
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
    use meetings::get_user_by_email;
    use regex::Regex;

    std::env::set_var("ROCKET_CONFIG", "Debug.toml");
    let email_address = "foo@meet-os.com";

    let tmp_dir = tempfile::tempdir().unwrap();

    println!("tmp_dir: {:?}", tmp_dir);
    std::env::set_var("DATABASE_PATH", tmp_dir.path().join("db"));

    let email_file = tmp_dir.path().join("email.txt");
    std::env::set_var("EMAIL_FILE", &email_file);
    let client = Client::tracked(super::rocket()).unwrap();
    let response = client
        .post("/register")
        .header(ContentType::Form)
        .body("name=Foo Bar&email=foo@meet-os.com")
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert!(response.headers().get_one("set-cookie").is_none());

    let html = response.into_string().unwrap();
    assert!(html.contains("<title>We sent you an email</title>"));
    assert!(html.contains(r#"We sent you an email to <b>foo@meet-os.com</b> Please check your inbox and verify your email address."#));

    drop(client);
    // Without dropping the client here we get an error on the next line
    // value: Db(Tx("IO error: lock hold by current process, acquire time 1705858854 acquiring thread 58266: /tmp/.tmpVlNPFx/db/LOCK: No locks available"))
    let res = tokio_test::block_on(get_user_by_email("foo@meet-os.com"))
        .unwrap()
        .unwrap();
    assert_eq!(res.email, "foo@meet-os.com");
    assert_eq!(res.name, "Foo Bar");
    assert!(!res.verified);
    // date? code?

    //assert_eq!(email_file.to_str().unwrap(), "");
    let email = std::fs::read_to_string(email_file).unwrap();
    // https://meet-os.com/verify/register/c0514ec6-c51e-4376-ae8e-df82ef79bcef
    let re = Regex::new(r"http://localhost:8000/verify/register/([a-z0-9-]+)").unwrap();

    log::info!("email: {email}");
    let code = match re.captures(&email) {
        Some(value) => value[1].to_owned(),
        None => panic!("Code not found in email"),
    };

    assert_eq!(code, res.code);

    let client = Client::tracked(super::rocket()).unwrap();

    // Access the profile without a cookie
    let response = client.get(format!("/profile")).dispatch();
    assert_eq!(response.status(), Status::Ok);
    let html = response.into_string().unwrap();
    //assert_eq!(html, "");
    assert!(html.contains("<title>Missing cookie</title>"));
    assert!(html.contains("It seems you are not logged in"));

    // Verify the email
    let response = client.get(format!("/verify/register/{code}")).dispatch();
    assert_eq!(response.status(), Status::Ok);
    let cookie = response.headers().get_one("set-cookie").unwrap();
    assert!(cookie.contains("meet-os="));
    let html = response.into_string().unwrap();
    assert!(html.contains("<title>Thank you for registering</title>"));
    assert!(html.contains("Your email was verified."));

    // Access the profile with the cookie
    let response = client
        .get(format!("/profile"))
        .private_cookie(("meet-os", email_address))
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let html = response.into_string().unwrap();
    //assert_eq!(html, "x");
    assert!(html.contains("<title>Profile</title>"));
    assert!(html.contains(r#"<h1 class="title is-3">Foo Bar</h1>"#));

    // Try to register with email that is already in our system
    // let client = Client::tracked(super::rocket()).unwrap();
    let response = client
        .post("/register")
        .header(ContentType::Form)
        .body("name=Peti Bar&email=foo@meet-os.com")
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert!(response.headers().get_one("set-cookie").is_none());
    let html = response.into_string().unwrap();
    assert!(html.contains("<title>Registration failed</title>"));
    //assert_eq!(html, "x");
}

#[test]
fn verify_with_non_existent_code() {
    let tmp_dir = tempfile::tempdir().unwrap();
    println!("tmp_dir: {:?}", tmp_dir);
    std::env::set_var("DATABASE_PATH", tmp_dir.path().join("db"));

    let email_file = tmp_dir.path().join("email.txt");
    std::env::set_var("EMAIL_FILE", &email_file);

    let client = Client::tracked(super::rocket()).unwrap();
    let response = client.get("/verify/register/abc").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let html = response.into_string().unwrap();
    //assert_eq!(html, "");
    //assert!(html.contains("<title>Thank you for registering</title>"));
    assert!(html.contains("Invalid code <b>abc</b>"));
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
