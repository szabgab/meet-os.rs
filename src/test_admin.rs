use crate::test_lib::{login_helper, params, register_user_helper, run_inprocess, setup_many};
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

#[test]
fn admin_page_as_user() {
    run_inprocess(|email_folder, client| {
        setup_many(&client, &email_folder);
        login_helper(&client, "foo@meet-os.com", "123foo");

        let res = client.get("/admin").dispatch();
        assert_eq!(res.status(), Status::Forbidden);
        let html = res.into_string().unwrap();
        // assert_eq!(html, "");
        check_html(&html, "title", "Unauthorized");
    })
}

#[test]
fn admin_page_as_admin() {
    run_inprocess(|email_folder, client| {
        setup_many(&client, &email_folder);
        login_helper(&client, "admin@meet-os.com", "123456");

        let res = client.get("/admin").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        //assert_eq!(html, "");
        check_html(&html, "title", "Admin");
        check_html(&html, "h1", "Admin");
        assert!(html.contains(r#"<div><a href="/admin/search">Search</a></div>"#));
        assert!(html.contains(r#"<div><a href="/admin/users">List users</a></div>"#));
        assert!(html.contains(r#"<div><a href="/admin/audit">Audit</a></div>"#));
    })
}

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

#[test]
fn admin_users_page_as_user() {
    run_inprocess(|email_folder, client| {
        setup_many(&client, &email_folder);
        login_helper(&client, "foo@meet-os.com", "123foo");

        let res = client.get("/admin/users").dispatch();
        assert_eq!(res.status(), Status::Forbidden);
        let html = res.into_string().unwrap();
        //assert_eq!(html, "");
        check_html(&html, "title", "Unauthorized");
    })
}

#[test]
fn admin_users_page_as_admin() {
    run_inprocess(|email_folder, client| {
        setup_many(&client, &email_folder);
        login_helper(&client, "admin@meet-os.com", "123456");

        let res = client.get("/admin/users").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        //assert_eq!(html, "");
        check_html(&html, "title", "List Users by Admin");
        check_html(&html, "h1", "List Users by Admin");
        assert!(html.contains(r#"<a href="/user/3">Foo 2</a>"#));
        assert!(html.contains(r#"<td><a href="/user/4">Site Manager</a></td>"#));
        assert!(html.contains(r#"<b>Total: 4</b>"#));
    })
}

#[test]
fn admin_search_get_as_guest() {
    run_inprocess(|email_folder, client| {
        let res = client.get("/admin/search").dispatch();
        assert_eq!(res.status(), Status::Unauthorized);
        let html = res.into_string().unwrap();
        check_html(&html, "title", "Not logged in");
    })
}

// /admin_search_get_as_user

#[test]
fn admin_search_get_as_admin() {
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
