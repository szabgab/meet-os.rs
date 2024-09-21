use crate::test_lib::{check_html, TestRunner};
use rocket::http::Status;

#[test]
fn get_register_page() {
    let tr = TestRunner::new();

    let res = tr.client.get("/register").dispatch();
    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    check_html!(&html, "title", "Register");
    check_html!(&html, "h1", "Register");
    assert!(html
        .contains(r#"<input name="name" class="input" id="name" type="text" placeholder="Name">"#));
    assert!(html.contains(
        r#"<input name="email"  class="input" id="email" type="email" placeholder="Email">"#
    ));
    assert!(html.contains(r#"<input name="password"  class="input" id="password" type="password"  placeholder="Password">"#));
}

#[test]
fn get_login_page() {
    let tr = TestRunner::new();

    let res = tr.client.get("/login").dispatch();
    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    check_html!(&html, "title", "Login");
    check_html!(&html, "h1", "Login");
    assert!(html.contains(
        r#"Email: <input name="email" class="input" id="email" type="email" placeholder="Email">"#
    ));
    assert!(html.contains(r#"Password: <input name="password" class="input" id="password" type="password" placeholder=Password">"#));
    assert!(html.contains(r#"<input type="submit" value="Login" class="button">"#));
    assert!(html.contains(r#"<a href="/reset-password">Reset password</a><br>"#));
    assert!(html.contains(
        r#"<a href="/resend-email-verification-code">Resend e-mail verification</a><br>"#
    ));
}
