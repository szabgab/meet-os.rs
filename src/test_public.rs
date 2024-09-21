use crate::test_lib::{check_html, TestRunner};
use rocket::http::Status;

#[test]
fn public_pages() {
    let tr = TestRunner::new();

    let res = tr.client.get("/about").dispatch();
    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    check_html!(&html, "title", "About Meet-OS");

    let res = tr.client.get("/soc").dispatch();
    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    check_html!(&html, "title", "Standard of Conduct");

    let res = tr.client.get("/privacy").dispatch();
    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    check_html!(&html, "title", "Privacy Policy");

    let res = tr.client.get("/faq").dispatch();
    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    check_html!(&html, "title", "FAQ - Frequently Asked Questions");

    let res = tr.client.get("/markdown").dispatch();
    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    check_html!(&html, "title", "Markdown at Meet-OS");
}
