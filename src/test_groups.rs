use crate::test_helpers::{register_user_helper, setup_many, setup_many_users};
use crate::test_lib::{check_html, params, run_inprocess};
use rocket::http::{ContentType, Status};

// GET /create-group show form
// POST /create-group verify name, add group to database
// GET /groups  list all the groups from the database

// guest cannot access the /create-group pages
// regular user cannot access the /create-group pages
// only admin user can access the /create-group pages
// everyone can access the /groups page

#[test]
fn create_group_by_admin() {
    run_inprocess(|email_folder, client| {
        setup_many_users(&client, &email_folder);
        let admin_email = "admin@meet-os.com";

        // Access the Group creation page with authorized user
        let res = client
            .get("/admin/create-group?uid=2")
            .private_cookie(("meet-os", admin_email))
            .dispatch();

        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        //assert_eq!(html, "x");
        check_html(&html, "title", "Create Group");
        check_html(&html, "h1", "Create Group");

        // Create a Group
        let res = client
            .post("/admin/create-group")
            .header(ContentType::Form)
            .body(params!([
                ("name", "Rust Maven"),
                ("location", "Virtual"),
                (
                    "description",
                    "Text with [link](https://rust.code-maven.com/)",
                ),
                ("owner", "2"),
            ]))
            .private_cookie(("meet-os", admin_email))
            .dispatch();

        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();

        // List the groups
        let res = client.get("/groups").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        //assert_eq!(html, "x");
        assert!(html.contains(r#"<li><a href="/group/1">Rust Maven</a></li>"#));
        check_html(&html, "title", "Groups");
        check_html(&html, "h1", "Groups");

        let res = client
            .post("/admin/create-group")
            .header(ContentType::Form)
            .body(params!([
                ("name", "Python Maven"),
                ("location", "Other"),
                ("description", "Text with [link](https://code-maven.com/)"),
                ("owner", "2"),
            ]))
            .private_cookie(("meet-os", admin_email))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);

        // List the groups
        let res = client.get("/groups").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        //assert_eq!(html, "x");
        assert!(html.contains(r#"<li><a href="/group/1">Rust Maven</a></li>"#));
        assert!(html.contains(r#"<li><a href="/group/2">Python Maven</a></li>"#));
        check_html(&html, "title", "Groups");
        check_html(&html, "h1", "Groups");
    });
}

#[test]
fn create_group_unauthorized() {
    run_inprocess(|email_folder, client| {
        let email = "peti@meet-os.com";
        register_user_helper(&client, "Peti Bar", email, "petibar", &email_folder);

        // Access the Group creation page with unauthorized user
        let res = client
            .get("/admin/create-group?uid=1")
            .private_cookie(("meet-os", email))
            .dispatch();

        assert_eq!(res.status(), Status::Forbidden);
        let html = res.into_string().unwrap();
        // assert_eq!(html, "");
        check_html(&html, "title", "Unauthorized");
        check_html(&html, "h1", "Unauthorized");

        // Create group should fail
        let res = client
            .post("/admin/create-group")
            .body(params!([
                ("name", "Rust Maven"),
                ("location", "Virtual"),
                ("description", "nope"),
                ("owner", "1"),
            ]))
            .private_cookie(("meet-os", email))
            .dispatch();

        assert_eq!(res.status(), Status::Forbidden);
        let html = res.into_string().unwrap();
        check_html(&html, "title", "Unauthorized");
        check_html(&html, "h1", "Unauthorized");

        // List the groups
        let res = client.get("/groups").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        //assert_eq!(html, "x");
        assert!(!html.contains("/group/1"));
        check_html(&html, "title", "Groups");
        check_html(&html, "h1", "Groups");
    });
}

#[test]
fn create_group_guest() {
    run_inprocess(|email_folder, client| {
        // Access the Group creation page without user
        let res = client.get("/admin/create-group?uid=1").dispatch();
        assert_eq!(res.status(), Status::Unauthorized);
        let html = res.into_string().unwrap();

        // assert_eq!(html, "");
        check_html(&html, "title", "Not logged in");
        check_html(&html, "h1", "Not logged in");
        assert!(html.contains("You are not logged in"));

        let res = client.get("/groups").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        //assert_eq!(html, "x");
        assert!(!html.contains("/group/")); // No link to any group
        check_html(&html, "title", "Groups");
        check_html(&html, "h1", "Groups");

        // Create group should fail
        let res = client
            .post("/admin/create-group")
            .body(params!([
                ("name", "Rust Maven"),
                ("location", ""),
                ("description", ""),
                ("owner", "1"),
            ]))
            .dispatch();

        assert_eq!(res.status(), Status::Unauthorized);
        let html = res.into_string().unwrap();
        check_html(&html, "title", "Not logged in");
        check_html(&html, "h1", "Not logged in");
        assert!(html.contains("You are not logged in"));

        // List the groups
        let res = client.get("/groups").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        //assert_eq!(html, "");
        assert!(!html.contains("/group/1"));
        check_html(&html, "title", "Groups");
        check_html(&html, "h1", "Groups");
    });
}

#[test]
fn join_group_guest() {
    run_inprocess(|email_folder, client| {
        let res = client.get("/join-group?gid=1").dispatch();
        assert_eq!(res.status(), Status::Unauthorized);
        let html = res.into_string().unwrap();

        // assert_eq!(html, "");
        check_html(&html, "title", "Not logged in");
        check_html(&html, "h1", "Not logged in");
        assert!(html.contains("You are not logged in"));
    })
}

#[test]
fn join_not_existing_group_as_user() {
    run_inprocess(|email_folder, client| {
        setup_many_users(&client, &email_folder);

        let res = client
            .get("/join-group?gid=20")
            .private_cookie(("meet-os", "foo@meet-os.com"))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();

        // assert_eq!(html, "");
        check_html(&html, "title", "No such group");
        check_html(&html, "h1", "No such group");
        assert!(html.contains("There is not group with id <b>20</b>"));
    })
}

#[test]
fn join_group_as_user() {
    run_inprocess(|email_folder, client| {
        setup_many(&client, &email_folder);

        let res = client
            .get("/join-group?gid=1")
            .private_cookie(("meet-os", "foo1@meet-os.com"))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();

        //assert_eq!(html, "");
        check_html(&html, "title", "Membership");
        check_html(&html, "h1", "Membership");
        assert!(html.contains(r#"User added to <a href="/group/1">group</a>"#));

        // check if user is listed on the group page
        let res = client.get("/group/1").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        //assert_eq!(html, "");
        check_html(&html, "title", "First Group");
        check_html(&html, "h1", "First Group");
        assert!(html.contains(r#"<h2 class="title is-4">Members</h2>"#));
        assert!(html.contains(r#"<a href="/user/3">Foo 1</a>"#));

        // try to join the same group again - should fail
        let res = client
            .get("/join-group?gid=1")
            .private_cookie(("meet-os", "foo1@meet-os.com"))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();

        // assert_eq!(html, "");
        check_html(&html, "title", "You are already a member of this group");
        check_html(&html, "h1", "You are already a member of this group");
        assert!(html.contains(
            r#"You are already a member of the <a href="/group/1">First Group</a> group"#
        ));

        // leave group
        let res = client
            .get("/leave-group?gid=1")
            .private_cookie(("meet-os", "foo1@meet-os.com"))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();

        //assert_eq!(html, "");
        check_html(&html, "title", "Membership");
        check_html(&html, "h1", "Membership");
        assert!(html.contains(r#"User removed from <a href="/group/1">group</a>"#));

        // See that user is NOT listed on the group page any more
        let res = client.get("/group/1").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        //assert_eq!(html, "");
        check_html(&html, "title", "First Group");
        check_html(&html, "h1", "First Group");
        assert!(html.contains(r#"<h2 class="title is-4">Members</h2>"#));
        assert!(!html.contains("Foo 1"));
        assert!(!html.contains("/user/3"));
    })
}

#[test]
fn join_group_as_owner() {
    run_inprocess(|email_folder, client| {
        setup_many(&client, &email_folder);

        let res = client
            .get("/join-group?gid=1")
            .private_cookie(("meet-os", "foo@meet-os.com"))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();

        //assert_eq!(html, "");
        check_html(&html, "title", "You are the owner of this group");
        check_html(&html, "h1", "You are the owner of this group");
        assert!(html.contains(r#"You cannot join a group you own."#));
    });
}

#[test]
fn leave_group_guest() {
    run_inprocess(|email_folder, client| {
        let res = client.get("/leave-group?gid=1").dispatch();
        assert_eq!(res.status(), Status::Unauthorized);
        let html = res.into_string().unwrap();

        // assert_eq!(html, "");
        check_html(&html, "title", "Not logged in");
        check_html(&html, "h1", "Not logged in");
        assert!(html.contains("You are not logged in"));
    })
}

#[test]
fn leave_not_existing_group() {
    run_inprocess(|email_folder, client| {
        setup_many(&client, &email_folder);

        let res = client
            .get("/leave-group?gid=20")
            .private_cookie(("meet-os", "foo@meet-os.com"))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();

        //assert_eq!(html, "");
        check_html(&html, "title", "No such group");
        check_html(&html, "h1", "No such group");
        assert!(html.contains("The group ID <b>20</b> does not exist."));
    })
}

#[test]
fn leave_group_as_owner() {
    run_inprocess(|email_folder, client| {
        setup_many(&client, &email_folder);

        let res = client
            .get("/leave-group?gid=1")
            .private_cookie(("meet-os", "foo@meet-os.com"))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();

        //assert_eq!(html, "");
        check_html(&html, "title", "You are the owner of this group");
        check_html(&html, "h1", "You are the owner of this group");
        assert!(html.contains(r#"You cannot leave a group you own."#));
    });
}

#[test]
fn leave_group_user_does_not_belong_to() {
    run_inprocess(|email_folder, client| {
        setup_many(&client, &email_folder);

        let res = client
            .get("/leave-group?gid=1")
            .private_cookie(("meet-os", "foo1@meet-os.com"))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();

        //assert_eq!(html, "");
        check_html(&html, "title", "You are not a member of this group");
        check_html(&html, "h1", "You are not a member of this group");
        assert!(html.contains(r#"You cannot leave a group where you are not a member."#));
    });
}

#[test]
fn edit_group_get_guest() {
    run_inprocess(|email_folder, client| {
        let res = client.get("/edit-group").dispatch();

        assert_eq!(res.status(), Status::Unauthorized);
        let html = res.into_string().unwrap();
        check_html(&html, "title", "Not logged in");
        check_html(&html, "h1", "Not logged in");
        assert!(html.contains("You are not logged in"));
    });
}

#[test]
fn edit_group_get_user_no_such_group() {
    run_inprocess(|email_folder, client| {
        setup_many_users(&client, &email_folder);
        let foo_email = "foo@meet-os.com";
        let res = client
            .get("/edit-group?gid=1")
            .private_cookie(("meet-os", foo_email))
            .dispatch();

        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        //assert_eq!(html, "");
        check_html(&html, "title", "No such group");
        check_html(&html, "h1", "No such group");
        assert!(html.contains("Group <b>1</b> does not exist"));
    });
}

#[test]
fn edit_group_get_user_is_not_the_owner() {
    run_inprocess(|email_folder, client| {
        setup_many(&client, &email_folder);

        let foo1_email = "foo1@meet-os.com";
        let res = client
            .get("/edit-group?gid=1")
            .private_cookie(("meet-os", foo1_email))
            .dispatch();

        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        //assert_eq!(html, "");
        check_html(&html, "title", "Not the owner");
        check_html(&html, "h1", "Not the owner");
        assert!(html.contains("You are not the owner of the group <b>1</b>"));
    });
}
