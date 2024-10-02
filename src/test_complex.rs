use crate::test_lib::{check_html, check_message, params, TestRunner};

use rocket::http::{ContentType, Status};

#[test]
fn test_complex() {
    let tr = TestRunner::new();
    tr.setup_admin();
    let owner_id = tr.setup_owner();
    tr.login_owner();

    // profile is not listing any groups
    let res = tr.client.get("/profile").dispatch();
    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    assert!(!html.contains("Owned Groups"));
    assert!(!html.contains("Group Membership"));

    tr.login_admin();
    let group_name = "Rust Maven";
    let res = tr
        .client
        .post("/admin/create-group")
        .header(ContentType::Form)
        .body(params!([
            ("name", group_name),
            ("location", "Virtual"),
            ("description", ""),
            ("owner", &owner_id),
        ]))
        .dispatch();
    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    check_message!(
        &html,
        "Group created",
        r#"Group <b><a href="/group/1">Rust Maven</a></b> created"#
    );

    // The profile now lists the group for the owner
    tr.login_owner();
    let res = tr.client.get("/profile").dispatch();
    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    assert!(html.contains("Owned Groups"));
    assert!(!html.contains("Group Membership"));
    assert!(html.contains(r#"<a href="/group/1">Rust Maven</a>"#));

    // Add event 1
    let first_event_title = "The first meeting";
    let res = tr
        .client
        .post("/add-event")
        .header(ContentType::Form)
        .body(params!([
            ("gid", "1"),
            ("offset", "-180"),
            ("title", first_event_title),
            ("location", "Virtual"),
            ("description", ""),
            ("date", "2030-01-01 10:10"),
        ]))
        .dispatch();
    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    check_message!(
        &html,
        "Event added",
        r#"Event added: <a href="/event/1">The first meeting</a>"#
    );

    // main page lists group and event
    let res = tr.client.get("/").dispatch();
    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    assert!(html.contains(r#"<h2 class="title is-4">Events</h2>"#));
    assert!(html.contains(r#"<h2 class="title is-4">Groups</h2>"#));
    assert!(html.contains(format!(r#"<a href="/group/1">{group_name}</a>"#).as_str()));
    assert!(html.contains(format!(r#"<a href="/event/1">{first_event_title}</a>"#).as_str()));

    // groups page lists group
    let res = tr.client.get("/groups").dispatch();
    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    assert!(html.contains(format!(r#"<a href="/group/1">{group_name}</a>"#).as_str()));

    // events page lists event
    let res = tr.client.get("/events").dispatch();
    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    assert!(html.contains(format!(r#"<a href="/event/1">{first_event_title}</a>"#).as_str()));

    // check event 1 page
    let res = tr.client.get("/event/1").dispatch();
    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    assert!(html.contains(format!(r#"<title>{first_event_title}</title>"#).as_str()));
    assert!(html.contains(format!(r#"<p class="title">{first_event_title}</p>"#).as_str()));
    assert!(html.contains(format!(r#"Organized by <a href="/group/1">{group_name}</a>."#).as_str()));

    // Add event 2
    let second_event_title = "The second excursion";
    let res = tr
        .client
        .post("/add-event")
        .header(ContentType::Form)
        .body(params!([
            ("gid", "1"),
            ("offset", "-180"),
            ("title", second_event_title),
            ("location", "Jerusalem"),
            ("description", ""),
            ("date", "2029-05-01 10:10"),
        ]))
        .dispatch();
    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    check_message!(
        &html,
        "Event added",
        r#"Event added: <a href="/event/2">The second excursion</a>"#
    );

    // Add event 3
    let third_event_title = "The 3rd party";
    let res = tr
        .client
        .post("/add-event")
        .header(ContentType::Form)
        .body(params!([
            ("gid", "1"),
            ("offset", "-180"),
            ("title", third_event_title),
            ("location", "Tel Aviv"),
            ("description", ""),
            ("date", "2029-06-02 10:10"),
        ]))
        .dispatch();
    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    check_message!(
        &html,
        "Event added",
        r#"Event added: <a href="/event/3">The 3rd party</a>"#
    );

    // main page
    let res = tr.client.get("/").dispatch();
    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    assert!(html.contains(r#"<h2 class="title is-4">Events</h2>"#));
    assert!(html.contains(r#"<h2 class="title is-4">Groups</h2>"#));
    assert!(html.contains(format!(r#"<a href="/group/1">{group_name}</a>"#).as_str()));
    assert!(html.contains(format!(r#"<a href="/event/1">{first_event_title}</a>"#).as_str()));
    assert!(html.contains(format!(r#"<a href="/event/2">{second_event_title}</a>"#).as_str()));
    assert!(html.contains(format!(r#"<a href="/event/3">{third_event_title}</a>"#).as_str()));

    // events page
    let res = tr.client.get("/events").dispatch();
    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    assert!(html.contains(format!(r#"<a href="/event/1">{first_event_title}</a>"#).as_str()));
    assert!(html.contains(format!(r#"<a href="/event/2">{second_event_title}</a>"#).as_str()));
    assert!(html.contains(format!(r#"<a href="/event/3">{third_event_title}</a>"#).as_str()));

    // Change event 2
    let second_event_title_2 = "New title for the 2nd event";
    let res = tr
        .client
        .post("/edit-event")
        .header(ContentType::Form)
        .body(params!([
            ("eid", "2"),
            ("offset", "-180"),
            ("title", second_event_title_2),
            ("location", "Ramat Gan"),
            ("description", "We need new description"),
            ("date", "2029-06-03 10:10"),
        ]))
        .dispatch();
    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    check_message!(
        &html,
        "Event updated",
        r#"Event updated: <a href="/event/2">New title for the 2nd event</a>"#
    );

    std::thread::sleep(std::time::Duration::from_millis(1000));
    // events page
    let res = tr.client.get("/events").dispatch();
    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    assert!(html.contains(format!(r#"<a href="/event/1">{first_event_title}</a>"#).as_str()));
    assert!(html.contains(format!(r#"<a href="/event/2">{second_event_title_2}</a>"#).as_str()));
    assert!(html.contains(format!(r#"<a href="/event/3">{third_event_title}</a>"#).as_str()));

    // check event 1 page
    let res = tr.client.get("/event/1").dispatch();
    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    check_html!(&html, "title", first_event_title);
    assert!(html.contains(format!(r#"<p class="title">{first_event_title}</p>"#).as_str()));
    assert!(html.contains(format!(r#"Organized by <a href="/group/1">{group_name}</a>."#).as_str()));

    // check event 2 page
    let res = tr.client.get("/event/2").dispatch();
    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    check_html!(&html, "title", second_event_title_2);
    assert!(html.contains(format!(r#"<p class="title">{second_event_title_2}</p>"#).as_str()));
    assert!(html.contains(format!(r#"Organized by <a href="/group/1">{group_name}</a>."#).as_str()));
}
