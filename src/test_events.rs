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

        // TODO check user has joined the group
        // TODO check if user is listed on the event page
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
fn leave_event() {
    run_inprocess(|email_folder, client| {
        setup_many(&client, &email_folder);

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

        // TODO check if user is NOT listed on the event page
        // TODO user is still in the group
    })
}
