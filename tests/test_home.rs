use utilities::{check_guest_menu, check_html, check_html_list, run_external};

#[test]
fn check_empty_home() {
    run_external(|port| {
        let url = format!("http://localhost:{port}");
        let res = reqwest::blocking::get(format!("{url}/")).unwrap();
        assert_eq!(res.status(), 200);

        let html = res.text().unwrap();
        check_html(&html, "title", "Meet-OS");
        check_html(&html, "h1", "Welcome to the Meet-OS meeting server");
        // check_html_list(
        //     &html,
        //     "li",
        //     vec![
        //         r#"<a href="/event/1">Web development with Rocket</a>"#,
        //         r#"<a href="/group/1">Rust Maven</a>"#,
        //     ],
        // );
        assert!(!html.contains("<h2>Events</h2>"));
        assert!(!html.contains("<h2>Groups</h2>"));
        check_guest_menu(&html);
    });
}
