use crate::test_lib::{check_html, run_inprocess};
use rocket::http::Status;

#[test]
fn register_page() {
    run_inprocess(|email_folder, client| {
        let res = client.get("/register").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
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
    run_inprocess(|email_folder, client| {
        let res = client.get("/login").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        check_html(&html, "title", "Login");
        check_html(&html, "h1", "Login");
        assert!(html.contains(r#"Email: <input name="email" id="email" type="email">"#));
    });
}
