use crate::test_helpers::{OWNER_EMAIL, OWNER_NAME};
use crate::test_lib::{
    check_guest_menu, check_html, check_profile_page_in_process, check_user_menu, params,
    read_code_from_email, run_inprocess,
};
use rocket::http::{ContentType, Status};

#[test]
fn test_register_with_invalid_email_address() {
    run_inprocess(|email_folder, client| {
        //"name=Foo Bar&email=meet-os.com&password=123456"
        let res = client
            .post("/register")
            .header(ContentType::Form)
            .body(params!([
                ("name", "Foo Bar"),
                ("email", "meet-os.com"),
                ("password", "123456"),
            ]))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        check_html(&html, "title", "Invalid email address");
        assert!(html.contains("Invalid email address <b>meet-os.com</b> Please try again"));
    });
}

#[test]
fn test_register_with_too_long_username() {
    run_inprocess(|email_folder, client| {
        let res = client
            .post("/register")
            .header(ContentType::Form)
            .body("name=QWERTYUIOPASDFGHJKLZXCVBNM QWERTYUIOPASDFGHJKLZXCVBNM&email=long@meet-os.com&password=123456")
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        check_html(&html, "title", "Name is too long");
        assert!(html.contains(
            "Name is too long. Max 50 while the current name is 53 long. Please try again."
        ));
    });
}

#[test]
fn test_simple() {
    run_inprocess(|email_folder, client| {
        // register user
        let res = client
            .post("/register")
            .header(ContentType::Form)
            .body("name=Foo Bar&email=foo@meet-os.com&password=123456")
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        assert!(res.headers().get_one("set-cookie").is_none());

        let html = res.into_string().unwrap();
        check_html(&html, "title", "We sent you an email");
        assert!(html.contains("We sent you an email to <b>foo@meet-os.com</b> Please check your inbox and verify your email address."));
        check_guest_menu(&html);

        // validate the email
        let (uid, code) = read_code_from_email(&email_folder, "0.txt", "verify-email");
        let res = client.get(format!("/verify-email/{uid}/{code}")).dispatch();
        assert_eq!(res.status(), Status::Ok);

        let html = res.into_string().unwrap();
        check_html(&html, "title", "Thank you for registering");
        assert!(html.contains("Your email was verified."));
        check_user_menu(&html);

        // Access the profile with the cookie
        check_profile_page_in_process(&client, OWNER_EMAIL, OWNER_NAME);
        //check_profile_page_in_process(&client, "foo@meet-os.com", "");

        // register with same email should fail
        let res = client
            .post("/register")
            .header(ContentType::Form)
            .body("name=Foo Bar&email=foo@meet-os.com&password=123456")
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        check_html(&html, "title", "Registration failed");
        assert!(html.contains("Could not register <b>foo@meet-os.com</b>"));

        // edit profile page invalid github account
        let res = client
            .post("/edit-profile")
            .private_cookie(("meet-os", OWNER_EMAIL))
            .header(ContentType::Form)
            .body("name=XX&github=szabgab*&gitlab=&linkedin&about=")
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        check_html(&html, "title", "Invalid GitHub username");
        assert!(html.contains(r#"The GitHub username `szabgab*` is not valid."#));

        // edit profile page invalid gitlab account
        let res = client
            .post("/edit-profile")
            .private_cookie(("meet-os", OWNER_EMAIL))
            .header(ContentType::Form)
            .body("name=XX&github=&gitlab=foo*bar&linkedin=&about=")
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        check_html(&html, "title", "Invalid GitLab username");
        assert!(html.contains(r#"The GitLab username `foo*bar` is not valid."#));

        let res = client
            .post("/edit-profile")
            .private_cookie(("meet-os", OWNER_EMAIL))
            .header(ContentType::Form)
            .body("name=XX&github=&gitlab=&linkedin=szabgab&about=")
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        check_html(&html, "title", "Invalid LinkedIn profile link");
        assert!(html.contains(r#"The LinkedIn profile link `szabgab` is not valid."#));

        // TODO test the validation of the other fields as well!

        //assert_eq!(html, "");

        // edit profile page
        // verify that if we submit html tags to the about field, those are properly escaped in the result
        let res = client
            .post("/edit-profile")
            .private_cookie(("meet-os", OWNER_EMAIL))
            .header(ContentType::Form)
            .body("name= Lord 😎 Voldemort &github= alfa &gitlab= beta &linkedin=  https://www.linkedin.com/in/szabgab/  &about=* text\n* more\n* [link](https://meet-os.com/)\n* <b>bold</b>\n* <a href=\"https://meet-os.com/\">bad link</a>\n")
            .dispatch();

        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        check_html(&html, "title", "Profile updated");
        assert!(html.contains(r#"Check out the <a href="/profile">profile</a> and how others see it <a href="/user/1">Lord 😎 Voldemort</a>"#));

        // Check updated profile
        let res = client
            .get("/profile")
            .private_cookie(("meet-os", OWNER_EMAIL))
            .dispatch();

        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        check_html(&html, "title", "Profile");
        assert!(html.contains(r#"<h1 class="title is-3">Lord 😎 Voldemort</h1>"#));
        assert!(html.contains(r#"<div><a href="https://github.com/alfa">GitHub</a></div>"#));
        assert!(html.contains(r#"<div><a href="https://gitlab.com/beta">GitLab</a></div>"#));

        // TODO: do we need to escape the characters when we submit them in the test or is this really what should be expected?
        assert!(html.contains(r#"<div><a href="https:&#x2F;&#x2F;www.linkedin.com&#x2F;in&#x2F;szabgab&#x2F;">LinkedIn</a></div>"#));
        //assert!(html.contains(r#"<div><a href="https:://www.linkedin.com/in/szabgab/">LinkedIn</a></div>"#));

        eprintln!("{html}");
        //assert_eq!(html, "");
        assert!(html.contains(
            r#"<div><ul>
<li>text</li>
<li>more</li>
<li><a href="https://meet-os.com/">link</a></li>
<li>&lt;b&gt;bold&lt;/b&gt;</li>
<li>&lt;a href=&quot;https://meet-os.com/&quot;&gt;bad link&lt;/a&gt;</li>
</ul>
</div>"#
        ));

        // TODO resend code?
        // TODO reset password?
    });
}
