use crate::test_lib::run_inprocess;
use rocket::http::Status;
use utilities::{check_guest_menu, check_html};

#[test]
fn try_page_without_cookie() {
    run_inprocess(|email_folder, client| {
        for path in ["/profile", "/admin/create-group?uid=1", "/admin"] {
            // Access the profile without a cookie
            let res = client.get(path).dispatch();
            assert_eq!(res.status(), Status::Unauthorized, "{path}");
            let html = res.into_string().unwrap();
            //assert_eq!(html, "");
            check_html(&html, "title", "Not logged in");
            assert!(html.contains("You are not logged in"));
            check_guest_menu(&html);
        }
    });
}
