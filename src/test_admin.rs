use crate::test_lib::{
    check_html, check_message, check_unauthorized, login_owner, params, setup_admin, setup_all,
    setup_owner, TestRunner, ADMIN_EMAIL, ADMIN_NAME, OTHER_NAME,
};

use rocket::http::{ContentType, Status};

#[test]
fn admin_pages_as_user() {
    let tr = TestRunner::new();

    setup_owner(&tr.client, &tr.email_folder);
    login_owner(&tr.client);

    for path in ["/admin", "/admin/users", "/admin/audit", "/admin/search"] {
        let res = tr.client.get(path).dispatch();
        check_unauthorized!(res);
    }

    let res = tr
        .client
        .post("/admin/search")
        .header(ContentType::Form)
        .dispatch();
    check_unauthorized!(res);
}

#[test]
fn admin_page_as_admin() {
    let tr = TestRunner::new();

    setup_admin(&tr.client, &tr.email_folder);
    tr.login_admin();

    let res = tr.client.get("/admin").dispatch();
    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    check_html!(&html, "title", "Admin");
    check_html!(&html, "h1", "Admin");
    assert!(html.contains(r#"<div><a href="/admin/search">Search</a></div>"#));
    assert!(html.contains(r#"<div><a href="/admin/users">List users</a></div>"#));
    assert!(html.contains(r#"<div><a href="/admin/audit">Audit</a></div>"#));
}

#[test]
fn admin_users_page_as_admin() {
    let tr = TestRunner::new();
    setup_all(&tr.client, &tr.email_folder);
    tr.login_admin();

    let res = tr.client.get("/admin/users").dispatch();
    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    check_html!(&html, "title", "List Users by Admin");
    check_html!(&html, "h1", "List Users by Admin");
    //assert_eq!(html, "");
    let expected = format!(r#"<a href="/user/4">{OTHER_NAME}</a>"#);
    assert!(html.contains(&expected));
    let expected = format!(r#"<td><a href="/user/1">{ADMIN_NAME}</a></td>"#);
    assert!(html.contains(&expected));
    assert!(html.contains(r#"<b>Total: 4</b>"#));
}

#[test]
fn admin_search_get_as_admin() {
    let tr = TestRunner::new();
    setup_admin(&tr.client, &tr.email_folder);
    tr.login_admin();

    let res = tr.client.get("/admin/search").dispatch();
    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    check_html!(&html, "title", "Search");
    assert!(html.contains(r#"<form method="POST" action="/admin/search">"#));
}

#[test]
fn admin_search_post_as_admin() {
    let tr = TestRunner::new();
    setup_all(&tr.client, &tr.email_folder);
    tr.login_admin();

    //no params
    let res = tr
        .client
        .post("/admin/search")
        .header(ContentType::Form)
        .dispatch();
    assert_eq!(res.status(), Status::UnprocessableEntity);

    // only query
    let res = tr
        .client
        .post("/admin/search")
        .header(ContentType::Form)
        .body(params!([("query", "admin"),]))
        .dispatch();
    assert_eq!(res.status(), Status::UnprocessableEntity);

    let res = tr
        .client
        .post("/admin/search")
        .header(ContentType::Form)
        .body(params!([("query", "admin"), ("table", "user")]))
        .dispatch();
    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    check_html!(&html, "title", "Search");
    assert!(html.contains(r#"<form method="POST" action="/admin/search">"#));
    assert!(html.contains(r#"<b>Total: 1</b>"#));
    assert!(html.contains(r#"<td><a href="/user/1">Site Manager</a></td>"#));
    let expect = format!(r#"<td>{ADMIN_EMAIL}</td>"#);
    assert!(html.contains(&expect));
}

#[test]
fn admin_audit_as_admin() {
    let tr = TestRunner::new();
    setup_admin(&tr.client, &tr.email_folder);
    tr.login_admin();

    let res = tr.client.get("/admin/audit").dispatch();
    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    //assert_eq!(html, "");
    check_html!(&html, "title", "Audit");
    check_html!(&html, "h1", "Audit");
    // TODO call some method that create entries and then check entries
}
