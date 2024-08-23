use rocket::http::Status;
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
    });
}

pub fn run_inprocess(func: fn()) {
    let tmp_dir = tempfile::tempdir().unwrap();
    println!("tmp_dir: {:?}", tmp_dir);
    std::env::set_var("ROCKET_CONFIG", "Rocket.skeleton.toml");
    std::env::set_var(
        "DATABASE_NAMESPACE",
        format!("test-namespace-{}", rand::random::<f64>()),
    );
    std::env::set_var(
        "DATABASE_NAME",
        format!("test-name-{}", rand::random::<f64>()),
    );
    std::env::set_var("EMAIL_FILE", tmp_dir.path().join("email.txt"));

    func();
}