use crate::{EmailAddress, MyConfig};

use sendgrid::v3::{
    ClickTrackingSetting, Content, Email, Message, OpenTrackingSetting, Personalization, Sender,
    SubscriptionTrackingSetting, TrackingSettings,
};

use std::env;
use std::fs::create_dir_all;
use std::fs::File;
use std::io::Write;
use std::path::Path;

// TODO display some error if the sendgrid key is empty
// TODO display some error if the email sending failed
/// # Panics
///
/// Panics when there is an error
pub async fn sendmail(
    myconfig: &MyConfig,
    from: &EmailAddress,
    to: &EmailAddress,
    subject: &str,
    text: &str,
) {
    if let Ok(email_folder) = env::var("EMAIL_FOLDER") {
        rocket::info!("email_folder: {email_folder}");
        let email_folder = Path::new(&email_folder);
        if !email_folder.exists() {
            create_dir_all(email_folder).unwrap();
        }
        let dir = email_folder
            .read_dir()
            .expect("read_dir call failed")
            .flatten()
            .collect::<Vec<_>>();
        rocket::info!("number of entries {}", dir.len());
        let filename = format!("{}.txt", dir.len());
        let email_file = email_folder.join(filename);
        rocket::info!("email_file: {email_file:?}");
        let mut file = File::create(email_file).unwrap();
        writeln!(&mut file, "{}", &text).unwrap();
    } else {
        // TODO display some error if the sendgrid key is empty
        // TODO display some error if the email sending failed
        sendgrid(&myconfig.sendgrid_api_key, from, to, subject, text).await;
    }
}

async fn sendgrid(
    api_key: &str,
    from: &EmailAddress,
    to: &EmailAddress,
    subject: &str,
    html: &str,
) {
    let person = Personalization::new(Email::new(&to.email).set_name(&to.name));

    let message = Message::new(Email::new(&from.email).set_name(&from.name))
        .set_subject(subject)
        .add_content(Content::new().set_content_type("text/html").set_value(html))
        .set_tracking_settings(TrackingSettings {
            click_tracking: Some(ClickTrackingSetting {
                enable: Some(false),
                enable_text: None,
            }),
            subscription_tracking: Some(SubscriptionTrackingSetting {
                enable: Some(false),
            }),
            open_tracking: Some(OpenTrackingSetting {
                enable: Some(false),
                substitution_tag: None,
            }),
        })
        .add_personalization(person);

    let sender = Sender::new(api_key.to_owned());
    match sender.send(&message).await {
        Ok(res) => rocket::info!("sent {}", res.status()),
        Err(err) => rocket::error!("err: {err}",),
    }
}
