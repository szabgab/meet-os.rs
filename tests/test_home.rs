use utilities::{check_guest_menu, check_html, check_html_list, run_external};

#[test]
fn home() {
    run_external(|port| {
        let url = format!("http://localhost:{port}");
        match reqwest::blocking::get(format!("{url}/")) {
            Ok(res) => {
                assert_eq!(res.status(), 200);
                match res.text() {
                    Ok(html) => {
                        check_html(&html, "title", "Meet-OS");
                        check_html(&html, "h1", "Welcome to the Rust meeting server");
                        // check_html_list(
                        //     &html,
                        //     "li",
                        //     vec![
                        //         r#"<a href="/event/1">Web development with Rocket</a>"#,
                        //         r#"<a href="/group/1">Rust Maven</a>"#,
                        //     ],
                        // );
                        check_html_list(&html, "h2", vec!["Events", "Groups"]);
                        check_guest_menu(&html);

                        //println!("{}", html)
                    }
                    Err(err) => assert_eq!(err.to_string(), ""),
                };
            }
            Err(err) => {
                assert_eq!(err.to_string(), "");
            }
        };
    });
}
