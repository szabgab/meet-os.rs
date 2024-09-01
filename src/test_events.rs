use crate::test_lib::{check_html, run_inprocess, setup_many};
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
