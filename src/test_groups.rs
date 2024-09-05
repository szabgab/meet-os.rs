use crate::test_helpers::{
    create_group_helper, logout, register_and_verify_user, setup_admin, setup_owner, setup_user,
    ADMIN_EMAIL, FOO_EMAIL, USER_EMAIL,
};
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
        setup_admin(&client, &email_folder);
        setup_owner(&client, &email_folder);

        // Access the Group creation page with authorized user
        let res = client
            .get("/admin/create-group?uid=2")
            .private_cookie(("meet-os", ADMIN_EMAIL))
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
            .private_cookie(("meet-os", ADMIN_EMAIL))
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
            .private_cookie(("meet-os", ADMIN_EMAIL))
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
        register_and_verify_user(&client, "Peti Bar", email, "petibar", &email_folder);

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
        setup_owner(&client, &email_folder);

        let res = client
            .get("/join-group?gid=20")
            .private_cookie(("meet-os", FOO_EMAIL))
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
        setup_admin(&client, &email_folder);
        setup_owner(&client, &email_folder);
        setup_user(&client, &email_folder);
        create_group_helper(&client, "First Group", 2);
        logout(&client);

        // user joins group
        let res = client
            .get("/join-group?gid=1")
            .private_cookie(("meet-os", USER_EMAIL))
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

        // visit the group page as a member of the group
        let res = client
            .get("/group/1")
            .private_cookie(("meet-os", USER_EMAIL))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        //assert_eq!(html, "");
        check_html(&html, "title", "First Group");
        check_html(&html, "h1", "First Group");
        assert!(html.contains(r#"<h2 class="title is-4">Members</h2>"#));
        assert!(html.contains(r#"<a href="/user/3">Foo 1</a>"#));
        assert!(html.contains(r#"You are a member."#));
        assert!(html.contains(r#"<a href="/leave-group?gid=1"><button class="button is-link">leave group</button></a>"#));

        // try to join the same group again - should fail
        let res = client
            .get("/join-group?gid=1")
            .private_cookie(("meet-os", USER_EMAIL))
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
            .private_cookie(("meet-os", USER_EMAIL))
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
        setup_admin(&client, &email_folder);
        setup_owner(&client, &email_folder);
        create_group_helper(&client, "First Group", 2);

        let res = client
            .get("/join-group?gid=1")
            .private_cookie(("meet-os", FOO_EMAIL))
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
        setup_owner(&client, &email_folder);

        let res = client
            .get("/leave-group?gid=20")
            .private_cookie(("meet-os", FOO_EMAIL))
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
        setup_admin(&client, &email_folder);
        setup_owner(&client, &email_folder);
        create_group_helper(&client, "First Group", 2);

        let res = client
            .get("/leave-group?gid=1")
            .private_cookie(("meet-os", FOO_EMAIL))
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
        setup_admin(&client, &email_folder);
        setup_owner(&client, &email_folder);
        setup_user(&client, &email_folder);
        create_group_helper(&client, "First Group", 2);

        let res = client
            .get("/leave-group?gid=1")
            .private_cookie(("meet-os", USER_EMAIL))
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
        setup_owner(&client, &email_folder);

        let res = client
            .get("/edit-group?gid=1")
            .private_cookie(("meet-os", FOO_EMAIL))
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
        setup_admin(&client, &email_folder);
        setup_owner(&client, &email_folder);
        setup_user(&client, &email_folder);
        create_group_helper(&client, "First Group", 2);

        let res = client
            .get("/edit-group?gid=1")
            .private_cookie(("meet-os", USER_EMAIL))
            .dispatch();

        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        //assert_eq!(html, "");
        check_html(&html, "title", "Not the owner");
        check_html(&html, "h1", "Not the owner");
        assert!(html.contains("You are not the owner of the group <b>1</b>"));
    });
}

#[test]
fn edit_group_get_by_owner() {
    run_inprocess(|email_folder, client| {
        setup_admin(&client, &email_folder);
        setup_owner(&client, &email_folder);
        create_group_helper(&client, "First Group", 2);

        let res = client
            .get("/edit-group?gid=1")
            .private_cookie(("meet-os", FOO_EMAIL))
            .dispatch();

        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        //assert_eq!(html, "");
        check_html(&html, "title", "Edit Group");
        check_html(&html, "h1", "Edit Group");
        assert!(html.contains(r#"<form method="POST" action="/edit-group">"#));
        assert!(html.contains(r#"<input type="hidden" name="gid" value="1">"#));
        assert!(
            html.contains(r#"Name: <input name="name" id="name" type="text" value="First Group">"#)
        );
        assert!(html
            .contains(r#"Location: <input name="location" id="location" type="text" value="">"#));
        assert!(html.contains(r#"Description (<a href="/markdown">Markdown</a>): <textarea name="description" id="description"></textarea>"#));
        assert!(html.contains(r#"<input type="submit" value="Save">"#));
    });
}

#[test]
fn visit_group_that_does_not_exist() {
    run_inprocess(|email_folder, client| {
        let res = client.get("/group/42").dispatch();

        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        //assert_eq!(html, "");
        check_html(&html, "title", "No such group");
        check_html(&html, "h1", "No such group");
        assert!(html.contains("The group <b>42</b> does not exist."));
    });
}

#[test]
fn edit_group_post_guest() {
    run_inprocess(|email_folder, client| {
        let res = client
            .post("/edit-group")
            .header(ContentType::Form)
            .dispatch();

        assert_eq!(res.status(), Status::Unauthorized);
        let html = res.into_string().unwrap();
        check_html(&html, "title", "Not logged in");
        check_html(&html, "h1", "Not logged in");
        assert!(html.contains("You are not logged in"));
    });
}
#[test]
fn edit_group_post_user_missing_gid() {
    run_inprocess(|email_folder, client| {
        setup_owner(&client, &email_folder);

        let res = client
            .post("/edit-group")
            .header(ContentType::Form)
            .private_cookie(("meet-os", FOO_EMAIL))
            .dispatch();

        assert_eq!(res.status(), Status::UnprocessableEntity);
        let html = res.into_string().unwrap();
        //assert_eq!(html, "");
        check_html(&html, "title", "422 Unprocessable Entity");
        check_html(&html, "h1", "422: Unprocessable Entity");
        assert!(html.contains(
            "The request was well-formed but was unable to be followed due to semantic errors."
        ));
    });
}

#[test]
fn edit_group_post_user_no_such_group() {
    run_inprocess(|email_folder, client| {
        setup_owner(&client, &email_folder);

        let res = client
            .post("/edit-group")
            .header(ContentType::Form)
            .private_cookie(("meet-os", FOO_EMAIL))
            .body(params!([
                ("gid", "1"),
                ("name", "Update"),
                ("location", "Virtual"),
                ("description", "",),
            ]))
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
fn edit_group_post_owner() {
    run_inprocess(|email_folder, client| {
        setup_admin(&client, &email_folder);
        setup_owner(&client, &email_folder);
        //setup_foo1(&client, &email_folder);
        create_group_helper(&client, "First Group", 2);
        logout(&client);

        let res = client
            .post("/edit-group")
            .header(ContentType::Form)
            .private_cookie(("meet-os", FOO_EMAIL))
            .body(params!([
                ("gid", "1"),
                ("name", "Updated name"),
                ("location", "Local"),
                ("description", "Some group"),
            ]))
            .dispatch();

        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        //assert_eq!(html, "");
        check_html(&html, "title", "Group updated");
        check_html(&html, "h1", "Group updated");
        assert!(html.contains(r#"Check out the <a href="/group/1">group</a>"#));

        // check if the group was updated
        let res = client.get("/group/1").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let html = res.into_string().unwrap();
        //assert_eq!(html, "");
        check_html(&html, "title", "Updated name");
        check_html(&html, "h1", "Updated name");
        assert!(html.contains(r#"<h2 class="title is-4">Members</h2>"#));
        assert!(html.contains(r#"No members in this group."#));
        assert!(html.contains(r#"<p>Some group</p>"#));
        assert!(html.contains(r#"<b>Location</b>: Local"#));
    });
}
