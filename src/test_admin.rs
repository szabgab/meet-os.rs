use crate::test_helpers::{login_admin, login_owner, setup_admin, setup_many, setup_owner};
use crate::test_lib::{check_html, params, run_inprocess};

use rocket::http::{ContentType, Status};

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
        setup_owner(&client, &email_folder);
        login_owner(&client);

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
        setup_admin(&client, &email_folder);
        login_admin(&client);

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
        setup_owner(&client, &email_folder);
        login_owner(&client);

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
        login_admin(&client);

        let res = client.get("/admin/users").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        //assert_eq!(html, "");
        check_html(&html, "title", "List Users by Admin");
        check_html(&html, "h1", "List Users by Admin");
        assert!(html.contains(r#"<a href="/user/4">Foo 2</a>"#));
        assert!(html.contains(r#"<td><a href="/user/1">Site Manager</a></td>"#));
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

#[test]
fn admin_search_get_as_user() {
    run_inprocess(|email_folder, client| {
        setup_owner(&client, &email_folder);
        login_owner(&client);

        let res = client.get("/admin/search").dispatch();
        assert_eq!(res.status(), Status::Forbidden);
        let html = res.into_string().unwrap();
        check_html(&html, "title", "Unauthorized");
    })
}

#[test]
fn admin_search_get_as_admin() {
    run_inprocess(|email_folder, client| {
        setup_admin(&client, &email_folder);
        login_admin(&client);

        let res = client.get("/admin/search").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        //assert_eq!(html, "");
        check_html(&html, "title", "Search");
        assert!(html.contains(r#"<form method="POST" action="/admin/search">"#));
    })
}

#[test]
fn admin_search_post_as_guest() {
    run_inprocess(|email_folder, client| {
        let res = client
            .post("/admin/search")
            .header(ContentType::Form)
            .dispatch();
        assert_eq!(res.status(), Status::Unauthorized);
        let html = res.into_string().unwrap();
        check_html(&html, "title", "Not logged in");
    })
}

#[test]
fn admin_search_post_as_user() {
    run_inprocess(|email_folder, client| {
        setup_owner(&client, &email_folder);
        login_owner(&client);

        let res = client
            .post("/admin/search")
            .header(ContentType::Form)
            .dispatch();
        assert_eq!(res.status(), Status::Forbidden);
        let html = res.into_string().unwrap();
        check_html(&html, "title", "Unauthorized");
    })
}

#[test]
fn admin_search_post_as_admin() {
    run_inprocess(|email_folder, client| {
        setup_many(&client, &email_folder);
        login_admin(&client);

        //no params
        let res = client
            .post("/admin/search")
            .header(ContentType::Form)
            .dispatch();
        assert_eq!(res.status(), Status::UnprocessableEntity);

        // only query
        let res = client
            .post("/admin/search")
            .header(ContentType::Form)
            .body(params!([("query", "admin"),]))
            .dispatch();
        assert_eq!(res.status(), Status::UnprocessableEntity);

        let res = client
            .post("/admin/search")
            .header(ContentType::Form)
            .body(params!([("query", "admin"), ("table", "user")]))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        //assert_eq!(html, "");
        check_html(&html, "title", "Search");
        assert!(html.contains(r#"<form method="POST" action="/admin/search">"#));
        assert!(html.contains(r#"<b>Total: 1</b>"#));
        assert!(html.contains(r#"<td><a href="/user/1">Site Manager</a></td>"#));
        assert!(html.contains(r#"<td>admin@meet-os.com</td>"#));
    })
}

#[test]
fn admin_audit_as_guest() {
    run_inprocess(|email_folder, client| {
        let res = client.get("/admin/audit").dispatch();
        assert_eq!(res.status(), Status::Unauthorized);
        let html = res.into_string().unwrap();
        //assert_eq!(html, "");
        check_html(&html, "title", "Not logged in");
    })
}

#[test]
fn admin_audit_as_user() {
    run_inprocess(|email_folder, client| {
        setup_owner(&client, &email_folder);
        login_owner(&client);

        let res = client.get("/admin/audit").dispatch();
        assert_eq!(res.status(), Status::Forbidden);
        let html = res.into_string().unwrap();
        // assert_eq!(html, "");
        check_html(&html, "title", "Unauthorized");
    })
}

#[test]
fn admin_audit_as_admin() {
    run_inprocess(|email_folder, client| {
        setup_admin(&client, &email_folder);
        login_admin(&client);

        let res = client.get("/admin/audit").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        //assert_eq!(html, "");
        check_html(&html, "title", "Audit");
        check_html(&html, "h1", "Audit");
        // TODO call some method that create entries and then check entries
    })
}
