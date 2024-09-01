use crate::test_lib::{check_html, run_inprocess};
use rocket::http::Status;

#[test]
fn public_pages() {
    run_inprocess(|email_folder, client| {
        let res = client.get("/about").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        check_html(&html, "title", "About Meet-OS");

        let res = client.get("/soc").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        check_html(&html, "title", "Standard of Conduct");

        let res = client.get("/privacy").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        check_html(&html, "title", "Privacy Policy");

        let res = client.get("/faq").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        check_html(&html, "title", "FAQ - Frequently Asked Questions");

        let res = client.get("/markdown").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        check_html(&html, "title", "Markdown at Meet-OS");
    });
}
