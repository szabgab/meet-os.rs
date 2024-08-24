//use regex::Regex;

use utilities::{check_html, run_external};

#[test]
fn register_page() {
    run_external(|port, _email_folder| {
        let client = reqwest::blocking::Client::new();
        let res = client
            .get(format!("http://localhost:{port}/register"))
            .send()
            .unwrap();
        assert_eq!(res.status(), 200);
        let html = res.text().unwrap();
        check_html(&html, "title", "Register");
        check_html(&html, "h1", "Register");
        assert!(html.contains(
            r#"<tr><td>Name:</td><td><input name="name" id="name" type="text"></td></tr>"#
        ));
        assert!(html.contains(
            r#"<tr><td>Email:</td><td><input name="email" id="email" type="email"></td></tr>"#
        ));
        assert!(html.contains(r#"<tr><td>Password:</td><td><input name="password" id="password" type="password"></td></tr>"#));
    });
}

#[test]
fn login_page() {
    run_external(|port, _email_folder| {
        let client = reqwest::blocking::Client::new();
        let res = client
            .get(format!("http://localhost:{port}/login"))
            .send()
            .unwrap();
        assert_eq!(res.status(), 200);
        let html = res.text().unwrap();
        check_html(&html, "title", "Login");
        check_html(&html, "h1", "Login");
        assert!(html.contains(r#"Email: <input name="email" id="email" type="email">"#));
    });
}

#[test]
fn fixed_pages() {
    run_external(|port, _email_folder| {
        let client = reqwest::blocking::Client::new();

        let res = client
            .get(format!("http://localhost:{port}/about"))
            .send()
            .unwrap();
        assert_eq!(res.status(), 200);
        let html = res.text().unwrap();
        check_html(&html, "title", "About Meet-OS");

        let res = client
            .get(format!("http://localhost:{port}/soc"))
            .send()
            .unwrap();
        assert_eq!(res.status(), 200);
        let html = res.text().unwrap();
        check_html(&html, "title", "Standard of Conduct");

        let res = client
            .get(format!("http://localhost:{port}/privacy"))
            .send()
            .unwrap();
        assert_eq!(res.status(), 200);
        let html = res.text().unwrap();
        check_html(&html, "title", "Privacy Policy");
    });
}
