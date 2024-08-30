use crate::test_lib::{register_user_helper, run_inprocess};
use rocket::http::Status;
use utilities::check_html;

#[test]
fn admin_list_users() {
    run_inprocess(|email_folder, client| {
        let _cookie_str = register_user_helper(
            &client,
            "Foo Bar",
            "foo@meet-os.com",
            "123foo",
            &email_folder,
        );

        let name = "Site Manager";
        let email = "admin@meet-os.com";
        let password = "123456";

        let admin_cookie_str = register_user_helper(&client, name, email, password, &email_folder);
        //login_helper(&client, &url, email, password);

        // Admin listing of users
        let res = client
            .get("/admin/users")
            .private_cookie(("meet-os", email))
            .dispatch();

        // TODO check that the user was verified
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        check_html(&html, "title", "List Users by Admin");
        println!("{html}");
        //check_html(&html, "title", "Meet-OS");
        assert!(html.contains("Foo Bar"));
        assert!(html.contains(name));

        // Regular listing of users by admin
        let res = client
            .get("/users")
            .private_cookie(("meet-os", email))
            .dispatch();

        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        println!("{html}");
        check_html(&html, "title", "List Users");
        assert!(html.contains("Foo Bar"));
        assert!(html.contains(name));
    });
}
