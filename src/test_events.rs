use crate::test_helpers::{
    create_group_helper, logout, setup_admin, setup_for_events, setup_owner, setup_user,
    OWNER_EMAIL, USER_EMAIL,
};
use crate::test_lib::{check_html, params, run_inprocess};
use rocket::http::{ContentType, Status};

// Create event
// Edit event
// List events

// Join event
// Leave event

#[test]
fn leave_event_before_joining_it() {
    run_inprocess(|email_folder, client| {
        setup_for_events(&client, &email_folder);

        let res = client
            .get("/rsvp-no-event?eid=1")
            .private_cookie(("meet-os", USER_EMAIL))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        //assert_eq!(html, "");
        check_html(&html, "title", "You were not registered to the event");
        check_html(&html, "h1", "You were not registered to the event");
        assert!(html.contains(r#"You were not registered to the <a href="/event/1">event</a>"#));
    });
}

#[test]
fn join_event() {
    run_inprocess(|email_folder, client| {
        setup_for_events(&client, &email_folder);

        // event page before
        let res = client
            .get("/event/1")
            .private_cookie(("meet-os", USER_EMAIL))
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
            .private_cookie(("meet-os", USER_EMAIL))
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
            .private_cookie(("meet-os", USER_EMAIL))
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
            .private_cookie(("meet-os", USER_EMAIL))
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
            .private_cookie(("meet-os", USER_EMAIL))
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
            .private_cookie(("meet-os", USER_EMAIL))
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
            .private_cookie(("meet-os", USER_EMAIL))
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
            .private_cookie(("meet-os", USER_EMAIL))
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
        setup_for_events(&client, &email_folder);

        let res = client
            .get("/rsvp-yes-event?eid=10")
            .private_cookie(("meet-os", USER_EMAIL))
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
        setup_for_events(&client, &email_folder);

        let res = client
            .get("/rsvp-no-event?eid=10")
            .private_cookie(("meet-os", USER_EMAIL))
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
        setup_for_events(&client, &email_folder);

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
        setup_for_events(&client, &email_folder);

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
        setup_for_events(&client, &email_folder);

        let res = client
            .get("/rsvp-yes-event?eid=1")
            .private_cookie(("meet-os", OWNER_EMAIL))
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
fn post_edit_event_guest() {
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
fn post_edit_event_user_missing_data() {
    run_inprocess(|email_folder, client| {
        setup_owner(&client, &email_folder);

        let res = client
            .post("/edit-event")
            .header(ContentType::Form)
            .private_cookie(("meet-os", OWNER_EMAIL))
            .dispatch();

        assert_eq!(res.status(), Status::UnprocessableEntity);

        let html = res.into_string().unwrap();
        // assert_eq!(html, "");
    });
}

#[test]
fn post_edit_event_user_no_such_event() {
    run_inprocess(|email_folder, client| {
        setup_owner(&client, &email_folder);

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
            .private_cookie(("meet-os", OWNER_EMAIL))
            .dispatch();

        assert_eq!(res.status(), Status::Ok);

        let html = res.into_string().unwrap();
        // assert_eq!(html, "");
        check_html(&html, "title", "No such event");
        check_html(&html, "h1", "No such event");

        assert!(html.contains("The event id <b>1</b> does not exist."));
    });
}

#[test]
fn post_edit_event_user_not_the_owner() {
    run_inprocess(|email_folder, client| {
        setup_for_events(&client, &email_folder);

        // update
        let res = client
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

        assert_eq!(res.status(), Status::Ok);

        let html = res.into_string().unwrap();
        check_html(&html, "title", "Not the owner");
        check_html(&html, "h1", "Not the owner");
        assert!(html.contains(r#"You are not the owner of group <b>1</b>"#));
    });
}

#[test]
fn post_edit_event_owner_title_too_short() {
    run_inprocess(|email_folder, client| {
        setup_for_events(&client, &email_folder);

        // update
        let res = client
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
        //assert_eq!(html, "");
        check_html(&html, "title", "Too short a title");
        check_html(&html, "h1", "Too short a title");
        assert!(html.contains(r#"Minimal title length 10 Current title len: 3"#));
    });
}

#[test]
fn post_edit_event_owner() {
    run_inprocess(|email_folder, client| {
        setup_for_events(&client, &email_folder);

        // check the event page before the update
        let res = client.get("/event/1").dispatch();
        assert_eq!(res.status(), Status::Ok);

        let html = res.into_string().unwrap();
        //assert_eq!(html, "");
        check_html(&html, "title", "First event");
        check_html(&html, "h1", "First event");
        assert!(html.contains("Virtual"));
        assert!(html.contains(r#"<span class="datetime" value="2030-01-01T07:10:00Z"></span>"#));

        // update
        let res = client
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
        //assert_eq!(html, "");
        check_html(&html, "title", "Event udapted");
        check_html(&html, "h1", "Event udapted");
        assert!(html.contains(r#"Event updated: <a href="/event/1">The new title</a>"#));

        // check the event page after the update
        let res = client.get("/event/1").dispatch();
        assert_eq!(res.status(), Status::Ok);

        let html = res.into_string().unwrap();
        //assert_eq!(html, "");
        check_html(&html, "title", "The new title");
        check_html(&html, "h1", "The new title");
        assert!(!html.contains("Virtual"));
        assert!(html.contains("In a pub"));
        assert!(html.contains(r#"<span class="datetime" value="2030-10-10T05:00:00Z"></span>"#));
    });
}

#[test]
fn get_add_event_guest() {
    run_inprocess(|email_folder, client| {
        let res = client.get("/add-event").dispatch();

        assert_eq!(res.status(), Status::Unauthorized);

        let html = res.into_string().unwrap();
        check_html(&html, "title", "Not logged in");
        // assert_eq!(html, "");
    });
}

#[test]
fn get_add_event_user_missing_gid() {
    run_inprocess(|email_folder, client| {
        setup_owner(&client, &email_folder);

        let res = client
            .get("/add-event")
            .private_cookie(("meet-os", OWNER_EMAIL))
            .dispatch();

        assert_eq!(res.status(), Status::NotFound);

        let html = res.into_string().unwrap();
        check_html(&html, "title", "404 Not Found");
        check_html(&html, "h1", "404 Not Found");
        // assert_eq!(html, "");
    });
}

#[test]
fn get_add_event_user_not_the_owner() {
    run_inprocess(|email_folder, client| {
        setup_for_events(&client, &email_folder);

        let res = client
            .get("/add-event?gid=1")
            .private_cookie(("meet-os", USER_EMAIL))
            .dispatch();

        assert_eq!(res.status(), Status::Ok);

        let html = res.into_string().unwrap();
        //assert_eq!(html, "");
        check_html(&html, "title", "Not the owner");
        check_html(&html, "h1", "Not the owner");
        assert!(html.contains("You are not the owner of group <b>1</b>"));
    });
}

#[test]
fn get_add_event_user_is_owner() {
    run_inprocess(|email_folder, client| {
        setup_for_events(&client, &email_folder);

        let res = client
            .get("/add-event?gid=1")
            .private_cookie(("meet-os", OWNER_EMAIL))
            .dispatch();

        assert_eq!(res.status(), Status::Ok);

        let html = res.into_string().unwrap();
        //assert_eq!(html, "");
        check_html(&html, "title", "Add event to the 'First Group' group");
        check_html(&html, "h1", "Add event to the 'First Group' group");
        assert!(html.contains(r#"<form method="POST" action="/add-event" id="add-event">"#));
        assert!(html.contains(r#"<input type="hidden" name="gid" value="1">"#));
        assert!(html.contains(r#"<input type="hidden" name="offset" id="offset">"#));
    });
}

#[test]
fn post_add_event_guest() {
    run_inprocess(|email_folder, client| {
        let res = client
            .post("/add-event")
            .header(ContentType::Form)
            .dispatch();

        assert_eq!(res.status(), Status::Unauthorized);

        let html = res.into_string().unwrap();
        check_html(&html, "title", "Not logged in");
    });
}

#[test]
fn post_add_event_user_not_owner() {
    run_inprocess(|email_folder, client| {
        setup_admin(&client, &email_folder);
        setup_owner(&client, &email_folder);
        setup_user(&client, &email_folder);
        create_group_helper(&client, "My group", 2);
        logout(&client);

        let res = client
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

        assert_eq!(res.status(), Status::Ok);

        let html = res.into_string().unwrap();
        // assert_eq!(html, "");
        check_html(&html, "title", "Not the owner");
        check_html(&html, "h1", "Not the owner");
        assert!(html.contains(r#"You are not the owner of group <b>1</b>"#));
    });
}

#[test]
fn post_add_event_owner_title_too_short() {
    run_inprocess(|email_folder, client| {
        setup_admin(&client, &email_folder);
        setup_owner(&client, &email_folder);
        setup_user(&client, &email_folder);
        create_group_helper(&client, "My group", 2);
        logout(&client);

        let res = client
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
        // assert_eq!(html, "");
        check_html(&html, "title", "Too short a title");
        check_html(&html, "h1", "Too short a title");
        assert!(html.contains(r#"Minimal title length 10 Current title len: 2"#));
    });
}

#[test]
fn post_add_event_owner_invalid_date() {
    run_inprocess(|email_folder, client| {
        setup_admin(&client, &email_folder);
        setup_owner(&client, &email_folder);
        setup_user(&client, &email_folder);
        create_group_helper(&client, "My group", 2);
        logout(&client);

        let res = client
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
        // assert_eq!(html, "");
        check_html(&html, "title", "Invalid date");
        check_html(&html, "h1", "Invalid date");
        assert!(html.contains(r#"Invalid date '2030-02-30 08:00' offset '-180'"#));
    });
}

#[test]
fn post_add_event_owner_event_in_the_past() {
    run_inprocess(|email_folder, client| {
        setup_admin(&client, &email_folder);
        setup_owner(&client, &email_folder);
        //setup_user(&client, &email_folder);
        create_group_helper(&client, "My group", 2);
        logout(&client);

        let res = client
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
        //assert_eq!(html, "");
        check_html(&html, "title", "Can't schedule event to the past");
        check_html(&html, "h1", "Can't schedule event to the past");
        assert!(html.contains(r#"Can't schedule event to the past '2020-02-10 05:00:00 UTC'"#));
    });
}

#[test]
fn post_add_event_owner() {
    run_inprocess(|email_folder, client| {
        setup_admin(&client, &email_folder);
        setup_owner(&client, &email_folder);
        setup_user(&client, &email_folder);
        create_group_helper(&client, "My group", 2);
        logout(&client);

        let res = client
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
        // assert_eq!(html, "");
        check_html(&html, "title", "Event added");
        check_html(&html, "h1", "Event added");
        assert!(html.contains(r#"Event added: <a href="/event/1">Event title</a>"#));
    });
}

#[test]
fn get_event_as_guest() {
    run_inprocess(|email_folder, client| {
        setup_for_events(&client, &email_folder);

        let res = client.get("/event/1").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        //assert_eq!(html, "");
        check_html(&html, "title", "First event");
        check_html(&html, "h1", "First event");
        // TODO check that there are no participants in this event
    });
}

#[test]
fn get_edit_event_as_guest() {
    run_inprocess(|email_folder, client| {
        let res = client.get("/edit-event").dispatch();

        assert_eq!(res.status(), Status::Unauthorized);

        let html = res.into_string().unwrap();
        check_html(&html, "title", "Not logged in");
    });
}

#[test]
fn get_edit_event_as_user_no_eid() {
    run_inprocess(|email_folder, client| {
        setup_owner(&client, &email_folder);

        let res = client
            .get("/edit-event")
            .private_cookie(("meet-os", OWNER_EMAIL))
            .dispatch();

        assert_eq!(res.status(), Status::NotFound);

        let html = res.into_string().unwrap();
        check_html(&html, "title", "404 Not Found");
    });
}

#[test]
fn get_edit_event_as_user_but_not_owner() {
    run_inprocess(|email_folder, client| {
        setup_for_events(&client, &email_folder);

        let res = client
            .get("/edit-event?eid=1")
            .private_cookie(("meet-os", USER_EMAIL))
            .dispatch();

        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        //assert_eq!(html, "");
        check_html(&html, "title", "Not the owner");
        check_html(&html, "h1", "Not the owner");
        assert!(html.contains("You are not the owner of group <b>1</b>"));
    });
}
#[test]
fn get_edit_event_as_owner_with_eid() {
    run_inprocess(|email_folder, client| {
        setup_for_events(&client, &email_folder);

        let res = client
            .get("/edit-event?eid=1")
            .private_cookie(("meet-os", OWNER_EMAIL))
            .dispatch();

        assert_eq!(res.status(), Status::Ok);

        let html = res.into_string().unwrap();
        //assert_eq!(html, "");
        check_html(&html, "title", "Edit event in the 'First Group' group");
        check_html(&html, "h1", "Edit event in the 'First Group' group");
        assert!(html.contains(r#"<form method="POST" action="/edit-event" id="edit-event">"#));
        assert!(html.contains(r#"<input type="hidden" name="eid" value="1">"#));
        assert!(html.contains(r#"<input type="hidden" name="offset" id="offset">"#));
        assert!(html
            .contains(r#"Title: <input name="title" id="title" type="text" value="First event">"#));
        assert!(html.contains(r#"Date: <input placeholder="YYYY-MM-DD HH::MM" name="date" id="date" type="text" original-value="2030-01-01T07:10:00Z">"#));
        assert!(html.contains(
            r#"Location: <input name="location" id="location" type="text" value="Virtual">"#
        ));
        assert!(html.contains(r#"Description (<a href="/markdown">Markdown</a>): <textarea name="description" id="description"></textarea>"#));
    });
}
