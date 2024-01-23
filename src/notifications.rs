use sendgrid::SGClient;
use sendgrid::{Destination, Mail};

use crate::EmailAddress;

pub async fn sendgrid(
    api_key: &str,
    from: &EmailAddress,
    to: &EmailAddress,
    subject: &str,
    html: &str,
) {
    let sg = SGClient::new(api_key);

    let mail_info = Mail::new()
        .add_to(Destination {
            address: &to.email,
            name: &to.name,
        })
        .add_from(&from.email)
        .add_from_name(&from.name)
        .add_subject(subject)
        .add_html(html);

    sg.send(mail_info).await.ok();
}
