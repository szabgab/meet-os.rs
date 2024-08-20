use std::env;
use std::fs::File;
use std::io::Write;

use meetings::{sendgrid, EmailAddress, MyConfig, User};

pub async fn admin_new_user_registered(myconfig: &MyConfig, user: &User) {
    let base_url = &myconfig.base_url;
    let subject = "New Meet-OS registration!";
    let text = format!(
        r#"Hi,

        New unverified user at {base_url}: {} {}
    <p>
    <p>
    If it was not you, we would like to apolozie. You don't need to do anything. We'll discard your registration if it is not validated.
    ";
    "#,
        user.name, user.email
    );

    // TODO: read from some config file
    let from = EmailAddress {
        name: String::from("Meet OS"),
        email: String::from("gabor@szabgab.com"),
    };

    //let admins = myconfig.clone().admins.clone();
    let admins = myconfig.admins.clone();

    for admin_email in admins {
        let to_address = &EmailAddress {
            name: String::new(),
            email: admin_email.clone(),
        };

        if let Ok(email_file) = env::var("EMAIL_FILE") {
            rocket::info!("email_file: {email_file}");
            let mut file = File::create(email_file).unwrap();
            writeln!(&mut file, "{}", &text).unwrap();
        } else {
            // TODO display some error if the sendgrid key is empty
            // TODO display some error if the email sending failed
            sendgrid(
                &myconfig.sendgrid_api_key,
                &from,
                to_address,
                subject,
                &text,
            )
            .await;
        }
    }
}
