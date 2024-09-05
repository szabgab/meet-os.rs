use crate::test_helpers::{register_and_verify_user, setup_many, FOO_EMAIL};
use crate::test_lib::{
    check_guest_menu, check_html, check_profile_page_in_process, check_user_menu, params,
    read_code_from_email, run_inprocess,
};

use rocket::http::{ContentType, Status};

#[test]
fn reset_password_full() {
    run_inprocess(|email_folder, client| {
        let name = "Foo Bar";
        register_and_verify_user(&client, name, FOO_EMAIL, "123456", &email_folder);

        let res = client.get("/profile").dispatch();
        assert_eq!(
            res.status(),
            Status::Ok,
            "User is logged in after calling register_user_helper"
        );
        let html = res.into_string().unwrap();
        check_user_menu(&html);

        let res = client.get("/logout").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        check_guest_menu(&html);

        let res = client.get("/reset-password").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        check_guest_menu(&html);
        check_html(&html, "title", "Reset password");

        // Try with other email addredd
        let res = client
            .post("/reset-password")
            .header(ContentType::Form)
            .body(params!([("email", "peter@meet-os.com"),]))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        check_guest_menu(&html);
        //assert_eq!(html, "");
        check_html(&html, "title", "No such user");
        assert!(html.contains("No user with address <b>peter@meet-os.com</b>"));

        // Try with the right email address
        let res = client
            .post("/reset-password")
            .header(ContentType::Form)
            .body(params!([("email", FOO_EMAIL),]))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        check_guest_menu(&html);
        // assert_eq!(html, "");
        check_html(&html, "title", "We sent you an email");
        assert!(html.contains("We sent you an email to <b>foo@meet-os.com</b> Please click on the link to reset your password."));

        // get code from email
        let (uid, code) = read_code_from_email(&email_folder, "3.txt", "save-password");

        let res = client
            .get(format!("/save-password/{uid}/{code}"))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        check_guest_menu(&html);
        //assert_eq!(html, "");
        // TODO check the form exists
        check_html(&html, "title", "Type in your new password");
        assert!(html.contains(r#"<input name="uid" id="uid" type="hidden" value="1">"#));
        let expected = format!(r#"<input name="code" id="code" type="hidden" value="{code}">"#);
        assert!(html.contains(&expected));

        // Cannot save invalid password (too short)
        let res = client
            .post("/save-password")
            .header(ContentType::Form)
            .body(params!([
                ("uid", uid.to_string()),
                ("code", code.clone()),
                ("password", String::from("abc"))
            ]))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        //assert_eq!(html, "");
        check_guest_menu(&html);
        check_html(&html, "title", "Invalid password");
        check_html(&html, "h1", "Invalid password");
        assert!(html.contains("The password must be at least 6 characters long."));

        let new_password = String::from("new password");
        let res = client
            .post("/save-password")
            .header(ContentType::Form)
            .body(params!([
                ("uid", uid.to_string()),
                ("code", code),
                ("password", new_password.clone())
            ]))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        check_guest_menu(&html);
        check_html(&html, "title", "Password updated");
        assert!(html.contains("Your password was updated."));
        //assert_eq!(html, "");

        // Try to login

        let res = client
            .post("/login")
            .header(ContentType::Form)
            .body(params!([("email", FOO_EMAIL), ("password", &new_password)]))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();

        //assert_eq!(html, "");
        check_html(&html, "title", "Welcome back");
        check_user_menu(&html);
        check_profile_page_in_process(&client, &FOO_EMAIL, name);

        // Try again with the same code
        // Try with id that does not exist
        // Try invalid password
    });
}

#[test]
fn save_password_get_invalid_uid() {
    run_inprocess(|email_folder, client| {
        let res = client.get("/save-password/42/abc").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        //assert_eq!(html, "");
        check_html(&html, "title", "Invalid id");
        assert!(html.contains("Invalid id <b>42</b>"));
    });
}

#[test]
fn save_password_get_invalid_code() {
    run_inprocess(|email_folder, client| {
        setup_many(&client, &email_folder);
        let res = client.get("/save-password/2/abc").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        //assert_eq!(html, "");
        check_html(&html, "title", "Invalid code");
        assert!(html.contains("Invalid code <b>abc</b>"));
    });
}

#[test]
fn save_password_post_invalid_uid() {
    run_inprocess(|email_folder, client| {
        let res = client
            .post("/save-password")
            .header(ContentType::Form)
            .body(params!([
                ("uid", "42"),
                ("code", "abc"),
                ("password", "new_password")
            ]))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        //assert_eq!(html, "");
        check_html(&html, "title", "Invalid userid");
        check_html(&html, "h1", "Invalid userid");
        assert!(html.contains("Invalid userid <b>42</b>"));
    });
}

#[test]
fn save_password_post_invalid_code() {
    run_inprocess(|email_folder, client| {
        setup_many(&client, &email_folder);
        let res = client
            .post("/save-password")
            .header(ContentType::Form)
            .body(params!([
                ("uid", "2"),
                ("code", "abc"),
                ("password", "new_password")
            ]))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        //assert_eq!(html, "");
        check_html(&html, "title", "Invalid code");
        check_html(&html, "h1", "Invalid code");
        assert!(html.contains("Invalid code <b>abc</b>"));
    });
}
