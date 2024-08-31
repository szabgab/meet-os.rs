use crate::test_lib::{params, register_user_helper, run_inprocess, setup_many};
use rocket::http::{ContentType, Status};
use utilities::check_html;

#[test]
fn admin_list_users() {
    run_inprocess(|email_folder, client| {
        register_user_helper(
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

#[test]
fn admin_page_as_guest() {
    run_inprocess(|email_folder, client| {
        let res = client.get("/admin").dispatch();
        assert_eq!(res.status(), Status::Unauthorized);
        let html = res.into_string().unwrap();
        //assert_eq!(html, "");
        check_html(&html, "title", "Not logged in");
    })
}

// /admin_page_as_user
// /admin_page_as_admin

#[test]
fn admin_users_page_as_guest() {
    run_inprocess(|email_folder, client| {
        let res = client.get("/admin/users").dispatch();
        assert_eq!(res.status(), Status::Unauthorized);
        let html = res.into_string().unwrap();
        //assert_eq!(html, "");
        check_html(&html, "title", "Not logged in");
    })
}

// /admin_users_page_as_user
// /admin_users_page_as_admin

#[test]
fn admin_search_page_as_guest() {
    run_inprocess(|email_folder, client| {
        let res = client.get("/admin/search").dispatch();
        assert_eq!(res.status(), Status::Unauthorized);
        let html = res.into_string().unwrap();
        check_html(&html, "title", "Not logged in");
    })
}

// /admin_search_page_as_user

#[test]
fn admin_search_page_as_admin() {
    run_inprocess(|email_folder, client| {
        setup_many(&client, &email_folder);

        let res = client
            .post(format!("/login"))
            .header(ContentType::Form)
            .body(params!([
                ("email", "admin@meet-os.com"),
                ("password", "123456")
            ]))
            .dispatch();

        let res = client.get("/admin/search").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        //assert_eq!(html, "");
        check_html(&html, "title", "Search");
        assert!(html.contains(r#"<form method="POST" action="/admin/search">"#));
    })
}

// search POST
