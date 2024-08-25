use utilities::{register_user_helper, run_external};

#[test]
fn test_complex() {
    run_external(|port, email_folder| {
        let client = reqwest::blocking::Client::new();
        let url = format!("http://localhost:{port}/");

        let admin_name = "Admin";
        let admin_email = "admin@meet-os.com";
        let admin_password = "123456";
        let admin_cookie_str = register_user_helper(
            &client,
            &url,
            admin_name,
            admin_email,
            admin_password,
            &email_folder,
        );

        let owner_name = "Owner";
        let owner_email = "owner@meet-os.com";
        let owner_password = "123456";
        let owner_cookie_str = register_user_helper(
            &client,
            &url,
            owner_name,
            &owner_email,
            &owner_password,
            &email_folder,
        );

        // profile is not listing the any groups
        let res = client
            .get(format!("{url}/profile"))
            .header("Cookie", format!("meet-os={owner_cookie_str}"))
            .send()
            .unwrap();
        assert_eq!(res.status(), 200);
        let html = res.text().unwrap();
        assert!(!html.contains("Owned Groups"));
        assert!(!html.contains("Group Membership"));

        let group_name = "Rust Maven";
        let res = client
            .post(format!("{url}/admin/create-group"))
            .form(&[
                ("name", group_name),
                ("location", "Virtual"),
                ("description", ""),
                ("owner", "2"),
            ])
            .header("Cookie", format!("meet-os={admin_cookie_str}"))
            .send()
            .unwrap();
        assert_eq!(res.status(), 200);

        // The profile now lists the group for the owner
        let res = client
            .get(format!("{url}/profile"))
            .header("Cookie", format!("meet-os={owner_cookie_str}"))
            .send()
            .unwrap();
        assert_eq!(res.status(), 200);
        let html = res.text().unwrap();
        assert!(html.contains("Owned Groups"));
        assert!(!html.contains("Group Membership"));
        assert!(html.contains(r#"<a href="/group/1">Rust Maven</a>"#));
        //assert_eq!(html, "");

        // TODO Add events
        // let event_title = "The first meeting";
        // let res = client
        //     .post(format!("{url}/add-events"))
        //     .form(&[
        //         ("gid", "1"),
        //         ("offset", "-180"),
        //         ("title", event_title),
        //         ("location", "Virtual"),
        //         ("description", ""),
        //         ("date", "2030-01-01 10:10"),
        //     ])
        //     .header("Cookie", format!("meet-os={owner_cookie_str}"))
        //     .send()
        //     .unwrap();
        // assert_eq!(res.status(), 200);

        // TODO list events
        // TODO check event pages
        // TODO change event
        // TODO check event pages
    });
}
