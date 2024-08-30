use crate::test_lib::{check_profile_page_in_process, extract_cookie, params, run_inprocess};
use rocket::http::{ContentType, Status};
use utilities::{check_guest_menu, check_html, check_user_menu, read_code_from_email};

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

#[test]
fn register_user() {
    run_inprocess(|email_folder, client| {
        let email = "foo@meet-os.com";
        let res = client
            .post(format!("/register"))
            .header(ContentType::Form)
            .body(params!([
                ("name", "Foo Bar"),
                ("email", email),
                ("password", "123456"),
            ]))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        //println!("{:#?}", res.headers());
        assert!(res.headers().get_one("set-cookie").is_none());

        let html = res.into_string().unwrap();
        check_html(&html, "title", "We sent you an email");
        assert!(html.contains("We sent you an email to <b>foo@meet-os.com</b> Please check your inbox and verify your email address."));
        check_guest_menu(&html);

        let (uid, code) = read_code_from_email(&email_folder, "0.txt");

        // Verify the email
        let res = client.get(format!("/verify-email/{uid}/{code}")).dispatch();
        assert_eq!(res.status(), Status::Ok);

        let cookie_str = extract_cookie(&res);

        let html = res.into_string().unwrap();
        check_html(&html, "title", "Thank you for registering");
        assert!(html.contains("Your email was verified."));
        check_user_menu(&html);

        // Access the profile with the cookie
        check_profile_page_in_process(&client, email, "Foo Bar");
    });
}

#[test]
fn verify_with_non_existent_id() {
    run_inprocess(|email_folder, client| {
        let res = client.get(format!("/verify-email/1/abc")).dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        //assert_eq!(html, "");
        check_html(&html, "title", "Invalid id");
        assert!(html.contains("Invalid id <b>1</b>"));
    });
}

#[test]
fn verify_with_bad_code() {
    run_inprocess(|email_folder, client| {
        let res = client
            .post(format!("/register"))
            .body(params!([
                ("name", "Foo Bar"),
                ("email", "foo@meet-os.com"),
                ("password", "123456"),
            ]))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);

        let res = client.get(format!("/verify-email/1/abc")).dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        //assert_eq!(html, "");
        check_html(&html, "title", "Invalid code");
        assert!(html.contains("Invalid code <b>abc</b>"));
    });
}

#[test]
fn duplicate_email() {
    run_inprocess(|email_folder, client| {
        let res = client
            .post(format!("/register"))
            .body(params!([
                ("name", "Foo Bar"),
                ("email", "foo@meet-os.com"),
                ("password", "123456"),
            ]))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        //println!("{:#?}", res.headers());
        assert!(res.headers().get_one("set-cookie").is_none());
        let html = res.into_string().unwrap();
        check_guest_menu(&html);
        check_html(&html, "title", "We sent you an email");
        assert!(html.contains("We sent you an email to <b>foo@meet-os.com</b> Please check your inbox and verify your email address."));

        let res = client
            .post(format!("/register"))
            .body(params!([
                ("name", "Foo Bar"),
                ("email", "foo@meet-os.com"),
                ("password", "123456"),
            ]))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        //println!("{:#?}", res.headers());
        assert!(res.headers().get_one("set-cookie").is_none());
        let html = res.into_string().unwrap();
        check_html(&html, "title", "Registration failed");
        check_guest_menu(&html);
    });
}
