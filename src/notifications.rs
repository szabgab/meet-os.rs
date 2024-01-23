use crate::EmailAddress;
use sendgrid::v3::{
    ClickTrackingSetting, Content, Email, Message, OpenTrackingSetting, Personalization, Sender,
    SubscriptionTrackingSetting, TrackingSettings,
};

pub async fn sendgrid(
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
