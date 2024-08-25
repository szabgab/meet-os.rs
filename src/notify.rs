use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;

use meetings::{db, sendmail, EmailAddress, Group, MyConfig, User};

pub async fn admin_new_user_registered(myconfig: &MyConfig, user: &User) {
    let base_url = &myconfig.base_url;
    let subject = "New Meet-OS registration!";
    let text = format!(
        r#"Hi,

        New unverified user: {} {}
    <p>
    Sent from {base_url}
    <p>
    "#,
        user.name, user.email
    );

    let from = EmailAddress {
        name: myconfig.from_name.clone(),
        email: myconfig.from_email.clone(),
    };

    //let admins = myconfig.clone().admins.clone();
    let admins = myconfig.admins.clone();

    for admin_email in admins {
        let to_address = &EmailAddress {
            name: String::new(),
            email: admin_email.clone(),
        };

        sendmail(myconfig, &from, to_address, subject, &text).await;
    }
}

pub async fn admin_new_user_verified(myconfig: &MyConfig, user: &User) {
    let base_url = &myconfig.base_url;
    let subject = "New Meet-OS user verification!";
    let text = format!(
        r#"Hi,

        New verified user: {} {}
    <p>
    Sent from {base_url}
    "#,
        user.name, user.email
    );

    let from = EmailAddress {
        name: myconfig.from_name.clone(),
        email: myconfig.from_email.clone(),
    };

    //let admins = myconfig.clone().admins.clone();
    let admins = myconfig.admins.clone();

    for admin_email in admins {
        let to_address = &EmailAddress {
            name: String::new(),
            email: admin_email.clone(),
        };

        sendmail(myconfig, &from, to_address, subject, &text).await;
    }
}

pub async fn owner_group_was_created(
    dbh: &Surreal<Client>,
    myconfig: &MyConfig,
    owner: &User,
    group: &Group,
) {
    let base_url = &myconfig.base_url;
    let subject = format!("Meet-OS: new group '{}' was created for you!", group.name);
    let text = format!(
        r#"Hi {},
        <p>
        A new group was created for you. <a href="{base_url}/group/{}">{}</a>.
        <p>
        Enjoy!
    <p>
    Sent from {base_url}
    "#,
        owner.name, group.gid, group.name
    );

    let from = EmailAddress {
        name: myconfig.from_name.clone(),
        email: myconfig.from_email.clone(),
    };

    send_to_group_owner(dbh, myconfig, &from, group, &subject, &text).await;
    send_to_admins(dbh, myconfig, &from, &subject, &text).await;
}

pub async fn owner_user_joined_group(
    dbh: &Surreal<Client>,
    myconfig: &MyConfig,
    user: &User,
    group: &Group,
) {
    let base_url = &myconfig.base_url;
    let subject = format!(
        "Meet-OS: user '{}' joined group '{}'!",
        user.name, group.name
    );
    let text = format!(
        r#"Hi,

        User <a href="{base_url}/user/{}">{}</a> has joined the Meet-OS group <a href="{base_url}/group/{}">{}</a>
    <p>
    Sent from {base_url}
    "#,
        user.uid, user.name, group.gid, group.name
    );

    let from = EmailAddress {
        name: myconfig.from_name.clone(),
        email: myconfig.from_email.clone(),
    };

    send_to_group_owner(dbh, myconfig, &from, group, &subject, &text).await;
    send_to_admins(dbh, myconfig, &from, &subject, &text).await;
}

pub async fn owner_user_left_group(
    dbh: &Surreal<Client>,
    myconfig: &MyConfig,
    user: &User,
    group: &Group,
) {
    let base_url = &myconfig.base_url;
    let subject = format!("Meet-OS: user '{}' left group '{}'!", user.name, group.name);
    let text = format!(
        r#"Hi,

        User <a href="{base_url}/user/{}">{}</a> has left the Meet-OS group <a href="{base_url}/group/{}">{}</a>
    <p>
    Sent from {base_url}
    "#,
        user.uid, user.name, group.gid, group.name
    );

    let from = EmailAddress {
        name: myconfig.from_name.clone(),
        email: myconfig.from_email.clone(),
    };

    send_to_group_owner(dbh, myconfig, &from, group, &subject, &text).await;
    send_to_admins(dbh, myconfig, &from, &subject, &text).await;
}

pub async fn group_members(
    dbh: &Surreal<Client>,
    myconfig: &MyConfig,
    subject: &str,
    text: &str,
    gid: usize,
) {
    let from = EmailAddress {
        name: myconfig.from_name.clone(),
        email: myconfig.from_email.clone(),
    };

    let members = db::get_members_of_group(dbh, gid).await.unwrap();
    for member in members {
        let to_address = &EmailAddress {
            name: member.0.name,
            email: member.0.email,
        };

        sendmail(myconfig, &from, to_address, subject, text).await;
    }

    // send to group owner as well

    let admins = myconfig.admins.clone();
    for admin_email in admins {
        let to_address = &EmailAddress {
            name: String::new(),
            email: admin_email.clone(),
        };

        sendmail(myconfig, &from, to_address, subject, text).await;
    }
}

async fn send_to_group_owner(
    dbh: &Surreal<Client>,
    myconfig: &MyConfig,
    from: &EmailAddress,
    group: &Group,
    subject: &str,
    text: &str,
) {
    let owner = db::get_user_by_id(dbh, group.owner).await.unwrap().unwrap();
    let to_address = &EmailAddress {
        name: owner.name,
        email: owner.email,
    };
    sendmail(myconfig, from, to_address, subject, text).await;
}

async fn send_to_admins(
    _dbh: &Surreal<Client>,
    myconfig: &MyConfig,
    from: &EmailAddress,
    subject: &str,
    text: &str,
) {
    let admins = myconfig.admins.clone();

    for admin_email in admins {
        let to_address = &EmailAddress {
            name: String::new(),
            email: admin_email.clone(),
        };

        sendmail(myconfig, from, to_address, subject, text).await;
    }
}
