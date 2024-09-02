use crate::test_helpers::setup_many;
use crate::test_lib::{check_html, run_inprocess};
use rocket::http::Status;

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
