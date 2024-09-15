use crate::test_lib::{
    check_guest_menu, check_html, check_message, check_profile_by_user, check_user_menu,
    clean_emails, logout, params, read_code_from_email, register_and_verify_user, run_inprocess,
    setup_admin, setup_owner, OWNER_EMAIL,
};

use rocket::http::{ContentType, Status};

#[test]
fn reset_password_full() {
    run_inprocess(|email_folder, client| {
        let name = "Foo Bar";
        register_and_verify_user(&client, name, OWNER_EMAIL, "123456", &email_folder);

        let res = client.get("/profile").dispatch();
        assert_eq!(
            res.status(),
            Status::Ok,
            "User is logged in after calling register_user_helper"
        );
        let html = res.into_string().unwrap();
        check_user_menu!(&html);

        let res = client.get("/logout").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        check_guest_menu!(&html);

        let res = client.get("/reset-password").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        check_guest_menu!(&html);
        check_html!(&html, "title", "Reset password");
        assert!(html.contains(r#"Email: <input name="email" class="input" id="email" type="email" placeholder="Email">"#));
        assert!(html.contains(r#"<input type="submit" class="button" value="Send code">"#));

        // Try with other email addredd
        let res = client
            .post("/reset-password")
            .header(ContentType::Form)
            .body(params!([("email", "peter@meet-os.com"),]))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        check_guest_menu!(&html);
        check_message!(
            &html,
            "No such user",
            "No user with address <b>peter@meet-os.com</b>. Please try again"
        );

        clean_emails(&email_folder);
        // Try with the right email address
        let res = client
            .post("/reset-password")
            .header(ContentType::Form)
            .body(params!([("email", OWNER_EMAIL),]))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        check_guest_menu!(&html);
        let expected = format!("We sent you an email to <b>{OWNER_EMAIL}</b> Please click on the link to reset your password.");
        check_message!(&html, "We sent you an email", &expected);

        // get code from email
        let (uid, code) = read_code_from_email(&email_folder, "0.txt", "save-password");

        let res = client
            .get(format!("/save-password/{uid}/{code}"))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        check_guest_menu!(&html);
        // TODO check the form exists
        check_html!(&html, "title", "Type in your new password");
        assert!(html.contains(r#"<input name="uid" id="uid" type="hidden" value="1">"#));
        let expected = format!(r#"<input name="code" id="code" type="hidden" value="{code}">"#);
        assert!(html.contains(&expected));
        assert!(html.contains(r#"Password: <input name="password" class="input" id="password" type="password" placeholder="Password">"#));
        assert!(html.contains(r#"<input type="submit" value="Save" class="button">"#));

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
        check_guest_menu!(&html);
        check_message!(
            &html,
            "Invalid password",
            "The password must be at least 6 characters long."
        );

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
        check_guest_menu!(&html);
        check_message!(&html, "Password updated", "Your password was updated.");

        // Try to login
        let res = client
            .post("/login")
            .header(ContentType::Form)
            .body(params!([
                ("email", OWNER_EMAIL),
                ("password", &new_password)
            ]))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();

        //assert_eq!(html, "");
        check_html!(&html, "title", "Welcome back");
        check_user_menu!(&html);
        check_profile_by_user!(&client, &OWNER_EMAIL, name);

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
        check_message!(&html, "Invalid id", "Invalid id <b>42</b>");
    });
}

#[test]
fn save_password_get_invalid_code() {
    run_inprocess(|email_folder, client| {
        setup_admin(&client, &email_folder);
        setup_owner(&client, &email_folder);
        logout(&client);

        let res = client.get("/save-password/2/abc").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        check_message!(&html, "Invalid code", "Invalid code <b>abc</b>");
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
        check_message!(&html, "Invalid userid", "Invalid userid <b>42</b>.");
    });
}

#[test]
fn save_password_post_invalid_code() {
    run_inprocess(|email_folder, client| {
        setup_admin(&client, &email_folder);
        setup_owner(&client, &email_folder);
        logout(&client);

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
        check_message!(&html, "Invalid code", "Invalid code <b>abc</b>.");
    });
}
