//use regex::Regex;

use utilities::{check_html, register_user_helper, run_external};

// GET /create-group show form
// POST /create-group verify name, add group to database
// GET /groups  list all the groups from the database

// guest cannot access the /create-group pages
// regular user cannot access the /create-group pages
// only admin user can access the /create-group pages
// everyone can access the /groups page

#[test]
fn create_group_by_admin() {
    run_external(|port| {
        let client = reqwest::blocking::Client::new();
        let url = format!("http://localhost:{port}/");

        let foo_cookie_str =
            register_user_helper(&client, &url, "Foo Bar", "foo@meet-os.com", "123foo");
        println!("foo_cookie_str: {foo_cookie_str}");
        let peti_cookie_str =
            register_user_helper(&client, &url, "Peti Bar", "peti@meet-os.com", "123peti");
        println!("peti_cookie_str: {peti_cookie_str}");

        // Access the Group creation page with authorized user
        let res = client
            .get(format!("{url}/create-group"))
            .header("Cookie", format!("meet-os={foo_cookie_str}"))
            .send()
            .unwrap();
        assert_eq!(res.status(), 200);

        let html = res.text().unwrap();
        //assert_eq!(html, "x");
        check_html(&html, "title", "Create Group");
        check_html(&html, "h1", "Create Group");

        // Create a Group
        let res = client
            .post(format!("{url}/create-group"))
            .form(&[
                ("name", "Rust Maven"),
                ("location", "Virtual"),
                (
                    "description",
                    "Text with [link](https://rust.code-maven.com/)",
                ),
            ])
            .header("Cookie", format!("meet-os={foo_cookie_str}"))
            .send()
            .unwrap();
        assert_eq!(res.status(), 200);

        // List the groups
        let res = client.get(format!("{url}/groups")).send().unwrap();
        assert_eq!(res.status(), 200);
        let html = res.text().unwrap();
        //assert_eq!(html, "x");
        assert!(html.contains(r#"<li><a href="/group/1">Rust Maven</a></li>"#));
        check_html(&html, "title", "Groups");
        check_html(&html, "h1", "Groups");

        let res = client
            .post(format!("{url}/create-group"))
            .form(&[
                ("name", "Python Maven"),
                ("location", "Other"),
                ("description", "Text with [link](https://code-maven.com/)"),
            ])
            .header("Cookie", format!("meet-os={foo_cookie_str}"))
            .send()
            .unwrap();
        assert_eq!(res.status(), 200);

        // List the groups
        let res = client.get(format!("{url}/groups")).send().unwrap();
        assert_eq!(res.status(), 200);
        let html = res.text().unwrap();
        //assert_eq!(html, "x");
        assert!(html.contains(r#"<li><a href="/group/1">Rust Maven</a></li>"#));
        assert!(html.contains(r#"<li><a href="/group/2">Python Maven</a></li>"#));
        check_html(&html, "title", "Groups");
        check_html(&html, "h1", "Groups");
    });
}

#[test]
fn create_group_unauthorized() {
    run_external(|port| {
        let client = reqwest::blocking::Client::new();
        let url = format!("http://localhost:{port}/");

        let peti_cookie_str =
            register_user_helper(&client, &url, "Peti Bar", "peti@meet-os.com", "petibar");
        println!("peti_cookie_str: {peti_cookie_str}");

        // Access the Group creation page with unauthorized user
        let res = client
            .get(format!("{url}/create-group"))
            .header("Cookie", format!("meet-os={peti_cookie_str}"))
            .send()
            .unwrap();
        assert_eq!(res.status(), 200);

        let html = res.text().unwrap();
        //assert_eq!(html, "x");
        check_html(&html, "title", "Unauthorized");
        check_html(&html, "h1", "Unauthorized");

        // Create group should fail
        let res = client
            .post(format!("{url}/create-group"))
            .form(&[
                ("name", "Rust Maven"),
                ("location", "Virtual"),
                ("description", "nope"),
            ])
            .header("Cookie", format!("meet-os={peti_cookie_str}"))
            .send()
            .unwrap();
        assert_eq!(res.status(), 200);
        check_html(&html, "title", "Unauthorized");
        check_html(&html, "h1", "Unauthorized");

        // List the groups
        let res = client.get(format!("{url}/groups")).send().unwrap();
        assert_eq!(res.status(), 200);
        let html = res.text().unwrap();
        //assert_eq!(html, "x");
        assert!(!html.contains("/group/1"));
        check_html(&html, "title", "Groups");
        check_html(&html, "h1", "Groups");
    });
}

#[test]
fn create_group_guest() {
    run_external(|port| {
        let client = reqwest::blocking::Client::new();
        let url = format!("http://localhost:{port}");

        // Access the Group creation page without user
        let res = client.get(format!("{url}/create-group")).send().unwrap();
        assert_eq!(res.status(), 200);

        let html = res.text().unwrap();
        //assert_eq!(html, "x");
        check_html(&html, "title", "Not logged in");
        check_html(&html, "h1", "Not logged in");

        let res = client.get(format!("{url}/groups")).send().unwrap();
        assert_eq!(res.status(), 200);
        let html = res.text().unwrap();
        //assert_eq!(html, "x");
        assert!(!html.contains("/group/")); // No link to any group
        check_html(&html, "title", "Groups");
        check_html(&html, "h1", "Groups");

        // Create group should fail
        let res = client
            .post(format!("{url}/create-group"))
            .form(&[
                ("name", "Rust Maven"),
                ("location", ""),
                ("description", ""),
            ])
            .send()
            .unwrap();
        assert_eq!(res.status(), 200);
        let html = res.text().unwrap();
        check_html(&html, "title", "Not logged in");
        check_html(&html, "h1", "Not logged in");

        // // List the groups
        let res = client.get(format!("{url}/groups")).send().unwrap();
        assert_eq!(res.status(), 200);
        let html = res.text().unwrap();
        //assert_eq!(html, "x");
        assert!(!html.contains("/group/1"));
        check_html(&html, "title", "Groups");
        check_html(&html, "h1", "Groups");
    });
}
