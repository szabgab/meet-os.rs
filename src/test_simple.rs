use rocket::http::{ContentType, Status};
use rocket::local::blocking::Client;
use utilities::{check_guest_menu, check_html};

#[test]
fn simple_check_empty_home() {
    run_inprocess(|| {
        let client = Client::tracked(super::rocket()).unwrap();

        let res = client.get("/").dispatch();

        assert_eq!(res.status(), Status::Ok);
        assert_eq!(
            res.headers().get_one("Content-Type").unwrap(),
            "text/html; charset=utf-8"
        );
        let html = res.into_string().unwrap();

        check_html(&html, "title", "Meet-OS");
        check_html(&html, "h1", "Welcome to the Meet-OS meeting server");
        assert!(!html.contains("<h2>Events</h2>"));
        assert!(!html.contains("<h2>Groups</h2>"));
        check_guest_menu(&html);

        // register user
        let res = client
            .post("/register")
            .header(ContentType::Form)
            .body("name=Foo Bar&email=foo@meet-os.com&password=123456")
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        assert!(res.headers().get_one("set-cookie").is_none());

        let html = res.into_string().unwrap();
        check_html(&html, "title", "We sent you an email");
        assert!(html.contains("We sent you an email to <b>foo@meet-os.com</b> Please check your inbox and verify your email address."));
        check_guest_menu(&html);

        // register with same email should fail
        let res = client
            .post("/register")
            .header(ContentType::Form)
            .body("name=Foo Bar&email=foo@meet-os.com&password=123456")
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        check_html(&html, "title", "Registration failed");
        assert!(html.contains("Could not register <b>foo@meet-os.com</b>"));

        // register with invalid email address
        let res = client
            .post("/register")
            .header(ContentType::Form)
            .body("name=Foo Bar&email=meet-os.com&password=123456")
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        check_html(&html, "title", "Invalid email address");
        assert!(html.contains("Invalid email address <b>meet-os.com</b> Please try again"));

        //assert_eq!(html, "");
    });
}

pub fn run_inprocess(func: fn()) {
    let tmp_dir = tempfile::tempdir().unwrap();
    println!("tmp_dir: {:?}", tmp_dir);
    let rocket_toml = std::fs::read_to_string("Rocket.skeleton.toml").unwrap();
    let db_name = format!("test-name-{}", rand::random::<f64>());
    let db_namespace = format!("test-namespace-{}", rand::random::<f64>());
    let rocket_toml = rocket_toml.replace("meet-os-local-db", &db_name);
    let rocket_toml = rocket_toml.replace("meet-os-local-ns", &db_namespace);
    let rocket_toml = rocket_toml.replace("Sendgrid | Folder", "Folder");
    let rocket_toml = rocket_toml.replace(
        "/path/to/email_folder",
        tmp_dir.path().join("emails").to_str().unwrap(),
    );

    let rocket_toml_path = tmp_dir.path().join("Rocket.toml");
    std::fs::write(&rocket_toml_path, rocket_toml).unwrap();

    std::env::set_var("ROCKET_CONFIG", rocket_toml_path);

    func();
}
