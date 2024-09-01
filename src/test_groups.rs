use crate::test_lib::{check_html, params, register_user_helper, run_inprocess};
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
        let admin_email = "admin@meet-os.com";
        let admin_cookie_str = register_user_helper(
            &client,
            "Site Manager",
            admin_email,
            "123Secret",
            &email_folder,
        );
        println!("admin_cookie_str: {admin_cookie_str}");
        let peti_cookie_str = register_user_helper(
            &client,
            "Peti Bar",
            "peti@meet-os.com",
            "123peti",
            &email_folder,
        );
        println!("peti_cookie_str: {peti_cookie_str}");

        // Access the Group creation page with authorized user
        let res = client
            .get("/admin/create-group?uid=1")
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
                ("owner", "1"),
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
