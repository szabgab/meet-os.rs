use crate::test_helpers::{setup_many, setup_many_users};
use crate::test_lib::{check_html, params, run_inprocess};
use rocket::http::{ContentType, Status};

// Create event
// Edit event
// List events

// Join event
// Leave event

#[test]
fn join_event() {
    run_inprocess(|email_folder, client| {
        setup_many(&client, &email_folder);

        // event page before
        let res = client
            .get("/event/1")
            .private_cookie(("meet-os", "foo1@meet-os.com"))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        //assert_eq!(html, "");
        check_html(&html, "title", "First event");
        check_html(&html, "h1", "First event");

        assert!(html.contains(r#"<h2 class="title is-4">Participating</h2>"#));
        assert!(!html.contains(r#"<li>Foo 1</li>"#));

        assert!(html.contains(r#"<button class="button is-link">"#));
        assert!(html.contains(r#"RSVP to the event"#));

        // make sure user not in the group
        let res = client.get("/group/1").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        // assert_eq!(html, "");
        check_html(&html, "title", "First Group");
        check_html(&html, "h1", "First Group");
        assert!(html.contains(r#"<h2 class="title is-4">Members</h2>"#));
        assert!(!html.contains(r#"Foo 1"#));

        // RSVP to event
        let res = client
            .get("/rsvp-yes-event?eid=1")
            .private_cookie(("meet-os", "foo1@meet-os.com"))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        //assert_eq!(html, "");
        check_html(&html, "title", "RSVPed to event");
        check_html(&html, "h1", "RSVPed to event");
        assert!(html.contains(r#"User RSVPed to <a href="/event/1">event</a>"#));

        // check if user has joined the group
        let res = client.get("/group/1").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        // assert_eq!(html, "");
        check_html(&html, "title", "First Group");
        check_html(&html, "h1", "First Group");
        assert!(html.contains(r#"<h2 class="title is-4">Members</h2>"#));
        assert!(html.contains(r#"<td><a href="/user/3">Foo 1</a></td>"#));

        //assert!(html.contains(r#""#));

        // check if user is listed on the event page
        let res = client
            .get("/event/1")
            .private_cookie(("meet-os", "foo1@meet-os.com"))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        //assert_eq!(html, "");
        check_html(&html, "title", "First event");
        check_html(&html, "h1", "First event");

        assert!(html.contains(r#"<h2 class="title is-4">Participating</h2>"#));
        assert!(html.contains(r#"<li>Foo 1</li>"#));

        assert!(html.contains(r#"<button class="button is-link">"#));
        assert!(html.contains(r#"Unregister from the event"#));

        // leave event
        let res = client
            .get("/rsvp-no-event?eid=1")
            .private_cookie(("meet-os", "foo1@meet-os.com"))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        //assert_eq!(html, "");
        check_html(&html, "title", "Not attending");
        check_html(&html, "h1", "Not attending");
        assert!(html.contains(r#"User not attending <a href="/event/1">event</a>"#));

        // check if user is NOT listed on the event page
        let res = client
            .get("/event/1")
            .private_cookie(("meet-os", "foo1@meet-os.com"))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        //assert_eq!(html, "");
        check_html(&html, "title", "First event");
        check_html(&html, "h1", "First event");

        assert!(html.contains(r#"<h2 class="title is-4">Participating</h2>"#));
        assert!(!html.contains(r#"<li>Foo 1</li>"#));

        assert!(html.contains(r#"<button class="button is-link">"#));
        assert!(html.contains(r#"RSVP to the event"#));

        // check if user is still in the group
        let res = client.get("/group/1").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        // assert_eq!(html, "");
        check_html(&html, "title", "First Group");
        check_html(&html, "h1", "First Group");
        assert!(html.contains(r#"<h2 class="title is-4">Members</h2>"#));
        assert!(html.contains(r#"<td><a href="/user/3">Foo 1</a></td>"#));

        // join event again
        let res = client
            .get("/rsvp-yes-event?eid=1")
            .private_cookie(("meet-os", "foo1@meet-os.com"))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        //assert_eq!(html, "");
        check_html(&html, "title", "RSVPed to event");
        check_html(&html, "h1", "RSVPed to event");
        assert!(html.contains(r#"User RSVPed to <a href="/event/1">event</a>"#));

        // check if user is listed on the event page
        let res = client
            .get("/event/1")
            .private_cookie(("meet-os", "foo1@meet-os.com"))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        //assert_eq!(html, "");
        check_html(&html, "title", "First event");
        check_html(&html, "h1", "First event");

        assert!(html.contains(r#"<h2 class="title is-4">Participating</h2>"#));
        assert!(html.contains(r#"<li>Foo 1</li>"#));

        assert!(html.contains(r#"<button class="button is-link">"#));
        assert!(html.contains(r#"Unregister from the event"#));

        // join event again while already joined
        let res = client
            .get("/rsvp-yes-event?eid=1")
            .private_cookie(("meet-os", "foo1@meet-os.com"))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        //assert_eq!(html, "");
        check_html(&html, "title", "You were already RSVPed");
        check_html(&html, "h1", "You were already RSVPed");
    })
}

#[test]
fn join_not_existing_event() {
    run_inprocess(|email_folder, client| {
        setup_many(&client, &email_folder);

        let res = client
            .get("/rsvp-yes-event?eid=10")
            .private_cookie(("meet-os", "foo1@meet-os.com"))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        //assert_eq!(html, "");
        check_html(&html, "title", "No such event");
        check_html(&html, "h1", "No such event");
    })
}

#[test]
fn leave_not_existing_event() {
    run_inprocess(|email_folder, client| {
        setup_many(&client, &email_folder);

        let res = client
            .get("/rsvp-no-event?eid=10")
            .private_cookie(("meet-os", "foo1@meet-os.com"))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        //assert_eq!(html, "");
        check_html(&html, "title", "No such event");
        check_html(&html, "h1", "No such event");
    })
}

#[test]
fn join_event_guest() {
    run_inprocess(|email_folder, client| {
        setup_many(&client, &email_folder);

        let res = client.get("/rsvp-yes-event?eid=1").dispatch();
        assert_eq!(res.status(), Status::Unauthorized);
        let html = res.into_string().unwrap();
        //assert_eq!(html, "");
        check_html(&html, "title", "Not logged in");
        check_html(&html, "h1", "Not logged in");
    })
}

#[test]
fn leave_event_guest() {
    run_inprocess(|email_folder, client| {
        setup_many(&client, &email_folder);

        let res = client.get("/rsvp-no-event?eid=1").dispatch();
        assert_eq!(res.status(), Status::Unauthorized);
        let html = res.into_string().unwrap();
        //assert_eq!(html, "");
        check_html(&html, "title", "Not logged in");
        check_html(&html, "h1", "Not logged in");
    })
}

#[test]
fn join_event_by_group_owner() {
    run_inprocess(|email_folder, client| {
        setup_many(&client, &email_folder);

        let res = client
            .get("/rsvp-yes-event?eid=1")
            .private_cookie(("meet-os", "foo@meet-os.com"))
            .dispatch();

        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        //assert_eq!(html, "");
        check_html(&html, "title", "You are the owner of this group");
        check_html(&html, "h1", "You are the owner of this group");

        assert!(html.contains("You cannot join an event in a group you own."));
    });
}

#[test]
fn edit_event_post_guest() {
    run_inprocess(|email_folder, client| {
        let res = client
            .post("/edit-event")
            .header(ContentType::Form)
            .dispatch();

        assert_eq!(res.status(), Status::Unauthorized);

        let html = res.into_string().unwrap();
        check_html(&html, "title", "Not logged in");
    });
}

#[test]
fn edit_event_post_user_missing_data() {
    run_inprocess(|email_folder, client| {
        setup_many_users(&client, &email_folder);

        let foo_email = "foo@meet-os.com";

        let res = client
            .post("/edit-event")
            .header(ContentType::Form)
            .private_cookie(("meet-os", foo_email))
            .dispatch();

        assert_eq!(res.status(), Status::UnprocessableEntity);

        let html = res.into_string().unwrap();
        // assert_eq!(html, "");
    });
}

#[test]
fn edit_event_post_user_no_such_event() {
    run_inprocess(|email_folder, client| {
        setup_many_users(&client, &email_folder);

        let foo_email = "foo@meet-os.com";

        let res = client
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
            .private_cookie(("meet-os", foo_email))
            .dispatch();

        assert_eq!(res.status(), Status::Ok);

        let html = res.into_string().unwrap();
        // assert_eq!(html, "");
        check_html(&html, "title", "No such event");
        check_html(&html, "h1", "No such event");

        assert!(html.contains("The event id <b>1</b> does not exist."));
    });
}
