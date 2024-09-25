use crate::test_lib::{
    check_html, check_message, check_not_the_owner, check_unprocessable, params, TestRunner,
    OWNER_EMAIL, USER_EMAIL, USER_NAME,
};
use rocket::http::{ContentType, Status};

// Create event
// Edit event
// List events

// Join event
// Leave event

#[test]
fn leave_event_before_joining_it() {
    let tr = TestRunner::new();

    tr.setup_for_events();
    tr.login_user();

    let res = tr.client.get("/rsvp-no-event?eid=1").dispatch();
    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    check_message!(
        &html,
        "You were not registered to the event",
        r#"You were not registered to the <a href="/event/1">event</a>"#
    );
}

#[test]
fn join_event() {
    let tr = TestRunner::new();
    tr.setup_for_events();
    tr.login_user();

    // event page before
    let res = tr.client.get("/event/1").dispatch();
    assert_eq!(res.status(), Status::Ok);

    let html = res.into_string().unwrap();
    check_html!(&html, "title", "First event");
    check_html!(&html, "h1", "First event");

    assert!(html.contains(r#"<h2 class="title is-4">Participating</h2>"#));
    let expected_user_name_listed_as_participant =
        format!(r#"<li><a href="/user/3">{USER_NAME}</a></li>"#);
    assert!(!html.contains(&expected_user_name_listed_as_participant));

    assert!(html.contains(r#"<button class="button is-link">"#));
    assert!(html.contains(r#"RSVP to the event"#));

    // make sure user not in the group
    let res = tr.client.get("/group/1").dispatch();
    assert_eq!(res.status(), Status::Ok);

    let html = res.into_string().unwrap();
    check_html!(&html, "title", "First Group");
    check_html!(&html, "h1", "First Group");
    assert!(html.contains(r#"<h2 class="title is-4">Members</h2>"#));
    assert!(!html.contains(&expected_user_name_listed_as_participant));

    // RSVP to event
    let res = tr.client.get("/rsvp-yes-event?eid=1").dispatch();
    assert_eq!(res.status(), Status::Ok);

    let html = res.into_string().unwrap();
    check_message!(
        &html,
        "RSVPed to event",
        r#"User RSVPed to <a href="/event/1">event</a>"#
    );

    // check if user has joined the group
    let res = tr.client.get("/group/1").dispatch();
    assert_eq!(res.status(), Status::Ok);

    let html = res.into_string().unwrap();
    check_html!(&html, "title", "First Group");
    check_html!(&html, "h1", "First Group");
    assert!(html.contains(r#"<h2 class="title is-4">Members</h2>"#));
    let expected = format!(r#"<td><a href="/user/3">{USER_NAME}</a></td>"#);
    assert!(html.contains(&expected));

    // check if user is listed on the event page
    let res = tr.client.get("/event/1").dispatch();
    assert_eq!(res.status(), Status::Ok);

    let html = res.into_string().unwrap();
    check_html!(&html, "title", "First event");
    check_html!(&html, "h1", "First event");
    assert!(html.contains(r#"<h2 class="title is-4">Participating</h2>"#));
    assert!(html.contains(USER_NAME));
    assert!(html.contains(r#"<button class="button is-link">"#));
    assert!(html.contains(r#"Unregister from the event"#));

    // leave event
    let res = tr.client.get("/rsvp-no-event?eid=1").dispatch();
    assert_eq!(res.status(), Status::Ok);

    let html = res.into_string().unwrap();
    check_message!(
        &html,
        "Not attending",
        r#"User not attending <a href="/event/1">event</a>"#
    );

    // check if user is NOT listed on the event page
    let res = tr.client.get("/event/1").dispatch();
    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    check_html!(&html, "title", "First event");
    check_html!(&html, "h1", "First event");

    assert!(html.contains(r#"<h2 class="title is-4">Participating</h2>"#));
    assert!(!html.contains(&expected_user_name_listed_as_participant));

    assert!(html.contains(r#"<button class="button is-link">"#));
    assert!(html.contains(r#"RSVP to the event"#));

    // check if user is still in the group
    let res = tr.client.get("/group/1").dispatch();
    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    // assert_eq!(html, "");
    check_html!(&html, "title", "First Group");
    check_html!(&html, "h1", "First Group");
    assert!(html.contains(r#"<h2 class="title is-4">Members</h2>"#));
    let expected = format!(r#"<td><a href="/user/3">{USER_NAME}</a></td>"#);
    assert!(html.contains(&expected));

    // join event again
    let res = tr.client.get("/rsvp-yes-event?eid=1").dispatch();
    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    check_message!(
        &html,
        "RSVPed to event",
        r#"User RSVPed to <a href="/event/1">event</a>"#
    );

    // check if user is listed on the event page
    let res = tr.client.get("/event/1").dispatch();
    assert_eq!(res.status(), Status::Ok);

    let html = res.into_string().unwrap();
    check_html!(&html, "title", "First event");
    check_html!(&html, "h1", "First event");

    assert!(html.contains(r#"<h2 class="title is-4">Participating</h2>"#));
    assert!(html.contains(USER_NAME));

    assert!(html.contains(r#"<button class="button is-link">"#));
    assert!(html.contains(r#"Unregister from the event"#));

    // join event again while already joined
    let res = tr.client.get("/rsvp-yes-event?eid=1").dispatch();
    assert_eq!(res.status(), Status::Ok);

    let html = res.into_string().unwrap();
    check_message!(&html, "You were already RSVPed", "You were already RSVPed");
}

#[test]
fn join_not_existing_event() {
    let tr = TestRunner::new();

    tr.setup_for_events();
    tr.login_user();

    let res = tr.client.get("/rsvp-yes-event?eid=10").dispatch();
    assert_eq!(res.status(), Status::Ok);

    let html = res.into_string().unwrap();
    check_message!(&html, "No such event", "No such event");
}

#[test]
fn leave_not_existing_event() {
    let tr = TestRunner::new();
    tr.setup_for_events();
    tr.login_user();

    let res = tr.client.get("/rsvp-no-event?eid=10").dispatch();
    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    check_message!(&html, "No such event", "No such event");
}

#[test]
fn join_event_by_group_owner() {
    let tr = TestRunner::new();

    tr.setup_for_events();
    tr.login_owner();

    let res = tr.client.get("/rsvp-yes-event?eid=1").dispatch();

    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    check_message!(
        &html,
        "You are the owner of this group",
        "You cannot join an event in a group you own."
    );
}

#[test]
fn post_edit_event_user_missing_data() {
    let tr = TestRunner::new();
    tr.setup_owner();
    tr.login_owner();

    let res = tr
        .client
        .post("/edit-event")
        .header(ContentType::Form)
        .dispatch();
    check_unprocessable!(res);
}

#[test]
fn post_edit_event_user_no_such_event() {
    let tr = TestRunner::new();

    tr.setup_owner();

    let res = tr
        .client
        .post("/edit-event")
        .header(ContentType::Form)
        .body(params!([
            ("title", "New title"),
            ("date", "2030-10-10 08:00"),
            ("location", "Virtual"),
            ("description", ""),
            ("offset", "-180"),
            ("eid", "1"),
        ]))
        .private_cookie(("meet-os", OWNER_EMAIL))
        .dispatch();

    assert_eq!(res.status(), Status::Ok);

    let html = res.into_string().unwrap();
    check_message!(
        &html,
        "No such event",
        "The event id <b>1</b> does not exist."
    );
}

#[test]
fn post_edit_event_user_not_the_owner() {
    let tr = TestRunner::new();
    tr.setup_for_events();

    // update
    let res = tr
        .client
        .post("/edit-event")
        .header(ContentType::Form)
        .body(params!([
            ("title", "The new title"),
            ("date", "2030-10-10 08:00"),
            ("location", "In a pub"),
            ("description", "This is the explanation"),
            ("offset", "-180"),
            ("eid", "1"),
        ]))
        .private_cookie(("meet-os", USER_EMAIL))
        .dispatch();

    check_not_the_owner!(res);
}

#[test]
fn post_edit_event_owner_title_too_short() {
    let tr = TestRunner::new();

    tr.setup_for_events();

    // update
    let res = tr
        .client
        .post("/edit-event")
        .header(ContentType::Form)
        .body(params!([
            ("title", "The"),
            ("date", "2030-10-10 08:00"),
            ("location", "In a pub"),
            ("description", "This is the explanation"),
            ("offset", "-180"),
            ("eid", "1"),
        ]))
        .private_cookie(("meet-os", OWNER_EMAIL))
        .dispatch();

    assert_eq!(res.status(), Status::Ok);

    let html = res.into_string().unwrap();
    check_message!(
        &html,
        "Too short a title",
        r#"Minimal title length 10 Current title len: 3"#
    );
}

#[test]
fn post_edit_event_owner_invalid_date() {
    let tr = TestRunner::new();

    tr.setup_for_events();

    // update
    let res = tr
        .client
        .post("/edit-event")
        .header(ContentType::Form)
        .body(params!([
            ("title", "The title is good"),
            ("date", "2030-13-10 08:00"),
            ("location", "In a pub"),
            ("description", "This is the explanation"),
            ("offset", "-180"),
            ("eid", "1"),
        ]))
        .private_cookie(("meet-os", OWNER_EMAIL))
        .dispatch();

    assert_eq!(res.status(), Status::Ok);

    let html = res.into_string().unwrap();
    check_message!(
        &html,
        "Invalid date",
        r#"Invalid date '2030-13-10 08:00' offset '-180'"#
    );
}

#[test]
fn post_edit_event_owner_date_in_the_past() {
    let tr = TestRunner::new();

    tr.setup_for_events();

    // update
    let res = tr
        .client
        .post("/edit-event")
        .header(ContentType::Form)
        .body(params!([
            ("title", "The title is good"),
            ("date", "2020-10-10 08:00"),
            ("location", "In a pub"),
            ("description", "This is the explanation"),
            ("offset", "-180"),
            ("eid", "1"),
        ]))
        .private_cookie(("meet-os", OWNER_EMAIL))
        .dispatch();

    assert_eq!(res.status(), Status::Ok);

    let html = res.into_string().unwrap();
    check_message!(
        &html,
        "Can't schedule event to the past",
        r#"Can't schedule event to the past '2020-10-10 05:00:00 UTC'"#
    );
}

#[test]
fn post_edit_event_owner() {
    let tr = TestRunner::new();

    tr.setup_for_events();

    // check the event page before the update
    let res = tr.client.get("/event/1").dispatch();
    assert_eq!(res.status(), Status::Ok);

    let html = res.into_string().unwrap();
    check_html!(&html, "title", "First event");
    check_html!(&html, "h1", "First event");
    assert!(html.contains("Virtual"));
    assert!(html.contains(r#"<span class="datetime" value="2030-01-01T07:10:00Z"></span>"#));

    // update
    let res = tr
        .client
        .post("/edit-event")
        .header(ContentType::Form)
        .body(params!([
            ("title", "The new title"),
            ("date", "2030-10-10 08:00"),
            ("location", "In a pub"),
            ("description", "This is the explanation"),
            ("offset", "-180"),
            ("eid", "1"),
        ]))
        .private_cookie(("meet-os", OWNER_EMAIL))
        .dispatch();

    assert_eq!(res.status(), Status::Ok);

    let html = res.into_string().unwrap();
    check_message!(
        &html,
        "Event updated",
        r#"Event updated: <a href="/event/1">The new title</a>"#
    );

    // check the event page after the update
    let res = tr.client.get("/event/1").dispatch();
    assert_eq!(res.status(), Status::Ok);

    let html = res.into_string().unwrap();
    check_html!(&html, "title", "The new title");
    check_html!(&html, "h1", "The new title");
    assert!(!html.contains("Virtual"));
    assert!(html.contains("In a pub"));
    assert!(html.contains(r#"<span class="datetime" value="2030-10-10T05:00:00Z"></span>"#));
}

#[test]
fn get_add_event_user_missing_gid() {
    let tr = TestRunner::new();

    tr.setup_owner();

    let res = tr
        .client
        .get("/add-event")
        .private_cookie(("meet-os", OWNER_EMAIL))
        .dispatch();

    assert_eq!(res.status(), Status::NotFound);

    let html = res.into_string().unwrap();
    check_message!(&html, "404 Not Found", "404 Not Found");
}

#[test]
fn get_add_event_user_not_the_owner() {
    let tr = TestRunner::new();

    tr.setup_for_events();

    let res = tr
        .client
        .get("/add-event?gid=1")
        .private_cookie(("meet-os", USER_EMAIL))
        .dispatch();
    check_not_the_owner!(res);
}

#[test]
fn get_add_event_user_is_owner() {
    let tr = TestRunner::new();

    tr.setup_for_events();

    let res = tr
        .client
        .get("/add-event?gid=1")
        .private_cookie(("meet-os", OWNER_EMAIL))
        .dispatch();

    assert_eq!(res.status(), Status::Ok);

    let html = res.into_string().unwrap();
    check_html!(&html, "title", "Add event to the 'First Group' group");
    check_html!(&html, "h1", "Add event to the 'First Group' group");
    assert!(html.contains(r#"<form method="POST" action="/add-event" id="add-event">"#));
    assert!(html.contains(r#"<input type="hidden" name="gid" value="1">"#));
    assert!(html.contains(r#"<input type="hidden" name="offset" id="offset">"#));
    // TODO the rest of the form
}

#[test]
fn post_add_event_user_not_owner() {
    let tr = TestRunner::new();

    tr.setup_for_events();

    let res = tr
        .client
        .post("/add-event")
        .header(ContentType::Form)
        .private_cookie(("meet-os", USER_EMAIL))
        .body(params!([
            ("title", "Event title"),
            ("date", "2030-10-10 08:00"),
            ("location", "Virtual"),
            ("description", ""),
            ("offset", "-180"),
            ("gid", "1"),
        ]))
        .dispatch();
    check_not_the_owner!(res);
}

#[test]
fn post_add_event_owner_title_too_short() {
    let tr = TestRunner::new();

    tr.setup_admin();
    tr.setup_owner();
    tr.setup_user();
    tr.create_group_helper("My group", 2);
    tr.logout();

    let res = tr
        .client
        .post("/add-event")
        .header(ContentType::Form)
        .private_cookie(("meet-os", OWNER_EMAIL))
        .body(params!([
            ("title", "OK"),
            ("date", "2030-10-10 08:00"),
            ("location", "Virtual"),
            ("description", ""),
            ("offset", "-180"),
            ("gid", "1"),
        ]))
        .dispatch();

    assert_eq!(res.status(), Status::Ok);

    let html = res.into_string().unwrap();
    check_message!(
        &html,
        "Too short a title",
        r#"Minimal title length 10 Current title len: 2"#
    );
}

#[test]
fn post_add_event_owner_invalid_date() {
    let tr = TestRunner::new();

    tr.setup_admin();
    tr.setup_owner();
    tr.setup_user();
    tr.create_group_helper("My group", 2);
    tr.logout();

    let res = tr
        .client
        .post("/add-event")
        .header(ContentType::Form)
        .private_cookie(("meet-os", OWNER_EMAIL))
        .body(params!([
            ("title", "Event title"),
            ("date", "2030-02-30 08:00"),
            ("location", "Virtual"),
            ("description", ""),
            ("offset", "-180"),
            ("gid", "1"),
        ]))
        .dispatch();

    assert_eq!(res.status(), Status::Ok);

    let html = res.into_string().unwrap();
    check_message!(
        &html,
        "Invalid date",
        r#"Invalid date '2030-02-30 08:00' offset '-180'"#
    );
}

#[test]
fn post_add_event_owner_event_in_the_past() {
    let tr = TestRunner::new();

    tr.setup_admin();
    tr.setup_owner();
    //tr.setup_user();
    tr.create_group_helper("My group", 2);
    tr.logout();

    let res = tr
        .client
        .post("/add-event")
        .header(ContentType::Form)
        .private_cookie(("meet-os", OWNER_EMAIL))
        .body(params!([
            ("title", "Event title"),
            ("date", "2020-02-10 08:00"),
            ("location", "Virtual"),
            ("description", ""),
            ("offset", "-180"),
            ("gid", "1"),
        ]))
        .dispatch();

    assert_eq!(res.status(), Status::Ok);

    let html = res.into_string().unwrap();
    check_message!(
        &html,
        "Can't schedule event to the past",
        r#"Can't schedule event to the past '2020-02-10 05:00:00 UTC'"#
    );
}

#[test]
fn post_add_event_owner() {
    let tr = TestRunner::new();

    tr.setup_admin();
    tr.setup_owner();
    tr.setup_user();
    tr.create_group_helper("My group", 2);
    tr.logout();

    let res = tr
        .client
        .post("/add-event")
        .header(ContentType::Form)
        .private_cookie(("meet-os", OWNER_EMAIL))
        .body(params!([
            ("title", "Event title"),
            ("date", "2030-10-10 08:00"),
            ("location", "Virtual"),
            ("description", ""),
            ("offset", "-180"),
            ("gid", "1"),
        ]))
        .dispatch();

    assert_eq!(res.status(), Status::Ok);

    let html = res.into_string().unwrap();
    check_message!(
        &html,
        "Event added",
        r#"Event added: <a href="/event/1">Event title</a>"#
    );
}

#[test]
fn get_event_as_guest() {
    let tr = TestRunner::new();

    tr.setup_for_events();

    let res = tr.client.get("/event/1").dispatch();
    assert_eq!(res.status(), Status::Ok);
    let html = res.into_string().unwrap();
    check_html!(&html, "title", "First event");
    check_html!(&html, "h1", "First event");
    // TODO check that there are no participants in this event
}

#[test]
fn get_edit_event_as_user_no_eid() {
    let tr = TestRunner::new();

    tr.setup_owner();

    let res = tr
        .client
        .get("/edit-event")
        .private_cookie(("meet-os", OWNER_EMAIL))
        .dispatch();

    assert_eq!(res.status(), Status::NotFound);

    let html = res.into_string().unwrap();
    check_message!(&html, "404 Not Found", "404 Not Found");
}

#[test]
fn get_edit_event_as_user_but_not_owner() {
    let tr = TestRunner::new();

    tr.setup_for_events();

    let res = tr
        .client
        .get("/edit-event?eid=1")
        .private_cookie(("meet-os", USER_EMAIL))
        .dispatch();
    check_not_the_owner!(res);
}
#[test]
fn get_edit_event_as_owner_with_eid() {
    let tr = TestRunner::new();

    tr.setup_for_events();

    let res = tr
        .client
        .get("/edit-event?eid=1")
        .private_cookie(("meet-os", OWNER_EMAIL))
        .dispatch();

    assert_eq!(res.status(), Status::Ok);

    let html = res.into_string().unwrap();
    check_html!(&html, "title", "Edit event in the 'First Group' group");
    check_html!(&html, "h1", "Edit event in the 'First Group' group");
    assert!(html.contains(r#"<form method="POST" action="/edit-event" id="edit-event">"#));
    assert!(html.contains(r#"<input type="hidden" name="eid" value="1">"#));
    assert!(html.contains(r#"<input type="hidden" name="offset" id="offset">"#));
    assert!(
        html.contains(r#"Title: <input name="title" id="title" type="text" value="First event">"#)
    );
    assert!(html.contains(r#"Date: <input placeholder="YYYY-MM-DD HH::MM" name="date" id="date" type="text" original-value="2030-01-01T07:10:00Z">"#));
    assert!(html.contains(
        r#"Location: <input name="location" id="location" type="text" value="Virtual">"#
    ));
    assert!(html.contains(r#"Description (<a href="/markdown">Markdown</a>): <textarea name="description" id="description"></textarea>"#));
}
