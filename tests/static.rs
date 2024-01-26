//use regex::Regex;

use utilities::{check_html, run_external};

#[test]
fn register_page() {
    run_external(|port| {
        let client = reqwest::blocking::Client::new();
        let res = client
            .get(format!("http://localhost:{port}/register"))
            .send()
            .unwrap();
        assert_eq!(res.status(), 200);
        let html = res.text().unwrap();
        check_html(&html, "title", "Register");
        //check_html(&html, "h1", "Web development with Rocket");
        assert!(html.contains(r#"Name: <input name="name" id="name" type="text">"#));
        assert!(html.contains(r#"Email: <input name="email" id="email" type="email">"#));
    });
}
