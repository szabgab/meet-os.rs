#![allow(clippy::std_instead_of_core)]

#[macro_use]
extern crate rocket;

#[allow(clippy::pub_with_shorthand)]
pub(crate) mod admin;
#[allow(clippy::pub_with_shorthand)]
pub(crate) mod public;
#[allow(clippy::pub_with_shorthand)]
pub(crate) mod web;

mod notify;

use chrono::{DateTime, Duration, Utc};

use rocket::form::Form;
use rocket::fs::{relative, FileServer};
use rocket::http::CookieJar;
use rocket::serde::uuid::Uuid;
use rocket::{fairing::AdHoc, State};
use rocket_dyn_templates::{context, Template};

use markdown::message;

use pbkdf2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Pbkdf2,
};

use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;

use meetings::db;

use meetings::{get_public_config, sendmail, EmailAddress, Event, MyConfig, User};

use web::Visitor;

#[derive(FromForm)]
struct ContactMembersForm<'r> {
    subject: &'r str,
    content: &'r str,
    gid: usize,
}

#[derive(FromForm)]
struct EventForm<'r> {
    title: &'r str,
    date: &'r str,
    location: &'r str,
    description: &'r str,
    offset: i64,
    gid: usize,
}

#[derive(FromForm)]
struct RegistrationForm<'r> {
    name: &'r str,
    email: &'r str,
    password: &'r str,
}

#[derive(FromForm)]
struct ProfileForm<'r> {
    name: &'r str,
    github: &'r str,
    gitlab: &'r str,
    linkedin: &'r str,
    about: &'r str,
}

#[derive(FromForm)]
struct GroupForm<'r> {
    gid: usize,
    name: &'r str,
    location: &'r str,
    description: &'r str,
}

#[derive(FromForm)]
struct LoginForm<'r> {
    email: &'r str,
    password: &'r str,
}

fn markdown2html(text: &str) -> Result<String, message::Message> {
    markdown::to_html_with_options(
        text,
        &markdown::Options {
            compile: markdown::CompileOptions {
                allow_dangerous_html: true,
                ..markdown::CompileOptions::default()
            },
            ..markdown::Options::gfm()
        },
    )
}

#[get("/")]
async fn index(
    cookies: &CookieJar<'_>,
    dbh: &State<Surreal<Client>>,
    myconfig: &State<MyConfig>,
) -> Template {
    let config = get_public_config();
    let visitor = Visitor::new(cookies, dbh, myconfig).await;

    let events = match db::get_events(dbh).await {
        Ok(val) => val,
        Err(err) => {
            rocket::error!("Error: {err}");
            return Template::render(
                "message",
                context! {title: "Internal error", message: "Internal error", config, visitor},
            );
        }
    };

    let groups = match db::get_groups(dbh).await {
        Ok(val) => val,
        Err(err) => {
            rocket::error!("Error: {err}");
            return Template::render(
                "message",
                context! {title: "Internal error", message: "Internal error", config, visitor},
            );
        }
    };

    Template::render(
        "index",
        context! {
            title: "Meet-OS",
            events,
            groups,
            config,
            visitor,
        },
    )
}

#[get("/events")]
async fn events(
    cookies: &CookieJar<'_>,
    dbh: &State<Surreal<Client>>,
    myconfig: &State<MyConfig>,
) -> Template {
    let config = get_public_config();
    let visitor = Visitor::new(cookies, dbh, myconfig).await;

    let events = match db::get_events(dbh).await {
        Ok(val) => val,
        Err(err) => {
            rocket::error!("Error: {err}");
            return Template::render(
                "message",
                context! {title: "Internal error", message: "Internal error", config, visitor},
            );
        }
    };

    Template::render(
        "events",
        context! {
            title: "Events",
            events,
            config,
            visitor,
        },
    )
}

#[get("/login")]
async fn login_get(
    cookies: &CookieJar<'_>,
    dbh: &State<Surreal<Client>>,
    myconfig: &State<MyConfig>,
) -> Template {
    Template::render(
        "login",
        context! {
            title: "Login",
            config: get_public_config(),
            visitor: Visitor::new(cookies, dbh, myconfig).await,
        },
    )
}

#[post("/login", data = "<input>")]
async fn login_post(
    cookies: &CookieJar<'_>,
    dbh: &State<Surreal<Client>>,
    myconfig: &State<MyConfig>,
    input: Form<LoginForm<'_>>,
) -> Template {
    rocket::info!("rocket login: {:?}", input.email);

    let config = get_public_config();
    let visitor = Visitor::new(cookies, dbh, myconfig).await;

    let email = input.email.to_lowercase().trim().to_owned();
    if !validator::validate_email(&email) {
        return Template::render(
            "message",
            context! {title: "Invalid email address", message: format!("Invalid email address <b>{}</b>. Please try again", input.email), config, visitor},
        );
    }

    let user = match db::get_user_by_email(dbh, &email).await {
        Ok(user) => user,
        Err(err) => {
            rocket::error!("Error: {err}");
            return Template::render(
                "message",
                context! {title: "No such user", message: format!("No user with address <b>{}</b>. Please try again", input.email), config, visitor},
            );
        }
    };

    let Some(user) = user else {
        return Template::render(
            "message",
            context! {title: "No such user", message: format!("No user with address <b>{}</b>. Please try again", input.email), config,visitor},
        );
    };

    if !user.verified {
        return Template::render(
            "message",
            context! {title: "Unverified email", message: "Email must be verified before login.", config,visitor},
        );
    }

    rocket::info!("email: {}", user.email);

    let password = input.password.trim().as_bytes();

    let parsed_hash = match PasswordHash::new(&user.password) {
        Ok(val) => val,
        Err(err) => {
            rocket::error!("Error: {err}");
            return Template::render(
                "message",
                context! {title: "Internal error", message: "Internal error", config, visitor},
            );
        }
    };

    if Pbkdf2.verify_password(password, &parsed_hash).is_err() {
        return Template::render(
            "message",
            context! {title: "Invalid password", message: "Invalid password", config, visitor},
        );
    }

    cookies.add_private(("meet-os", user.email.clone())); // TODO this should be the user ID, right?

    // It seems despite calling add_private, the cookies will still return the old value so
    // for now we have a separate constructor for the Visitor
    #[allow(clippy::shadow_unrelated)]
    let visitor = Visitor::new_after_login(&email, dbh, myconfig).await;
    Template::render(
        "message",
        context! {title: "Welcome back", message: r#"Welcome back. <a href="/profile">profile</a>"#, config, visitor},
    )
}

#[get("/logout")]
async fn logout_get(
    cookies: &CookieJar<'_>,
    dbh: &State<Surreal<Client>>,
    myconfig: &State<MyConfig>,
) -> Template {
    // TODO shall we check if the cookie was even there?
    let visitor = Visitor::new(cookies, dbh, myconfig).await;
    if !visitor.logged_in {
        rocket::warn!("Trying to log out while not logged in");
    }

    cookies.remove_private("meet-os");

    #[allow(clippy::shadow_unrelated)]
    let visitor = Visitor::new_after_logout();

    Template::render(
        "message",
        context! {title: "Logged out", message: "We have logged you out from the system", config: get_public_config(), visitor},
    )
}

// #[post("/reset-password", data = "<input>")]
// async fn reset_password_post(
//     cookies: &CookieJar<'_>,
//     dbh: &State<Surreal<Db>>,
//     input: Form<LoginForm<'_>>,
//     myconfig: &State<MyConfig>,
// ) -> Template {
//     rocket::info!("rocket login: {:?}", input.email);

//     let email = input.email.to_lowercase().trim().to_owned();
//     if !validator::validate_email(&email) {
//         return Template::render(
//             "message",
//             context! {title: "Invalid email address", message: format!("Invalid email address <b>{}</b>. Please try again", input.email), config: get_public_config(), visitor},
//         );
//     }

//     let user: User = match db::get_user_by_email(dbh, &email).await {
//         Ok(user) => match user {
//             Some(user) => user,
//             None => {
//                 return Template::render(
//                     "message",
//                     context! {title: "No such user", message: format!("No user with address <b>{}</b>. Please try again", input.email), config: get_public_config(),visitor},
//                 )
//             }
//         },
//         Err(err) => {
//             rocket::error!("Error: {err}");
//             return Template::render(
//                 "message",
//                 context! {title: "No such user", message: format!("No user with address <b>{}</b>. Please try again", input.email), config: get_public_config(), visitor},
//             );
//         }
//     };

//     let process = "login";
//     let code = Uuid::new_v4();

//     match add_login_code_to_user(dbh, &email, process, code.to_string().as_str()).await {
//         Ok(_result) => (),
//         Err(err) => {
//             rocket::info!("Error while trying to add user {err}");
//             return Template::render(
//                 "message",
//                 context! {title: "Internal error", message: "Oups", config: get_public_config(), visitor,},
//             );
//         }
//     };

//     let base_url = &myconfig.base_url;

//     let subject = "Verify your Meet-OS login!";
//     let text = format!(
//         r#"Hi,
//     Someone used your email to try to login the Meet-OS web site.
//     If it was you, please <a href="{base_url}/verify/{process}/{code}">click on this link</a> to finish the login process.
//     <p>
//     <p>
//     If it was not you, we would like to apologize. You don't need to do anything..
//     ";
//     "#
//     );

//     // TODO: read from some config file
// let from = EmailAddress {
//     name: myconfig.from_name.clone(),
//     email: myconfig.from_email.clone(),
// };

//     let to_address = &EmailAddress {
//         name: user.name.clone(),
//         email: input.email.to_owned(),
//     };

//     sendmail(&myconfig, &from, to_address, subject, &text).await;

//     Template::render(
//         "message",
//         context! {title: "We sent you an email", message: format!("We sent you an email to <b>{}</b> Please click on the link to finish the login process.", to_address.email), config: get_public_config(), visitor},
//     )
// }

#[get("/register")]
async fn register_get(
    cookies: &CookieJar<'_>,
    dbh: &State<Surreal<Client>>,
    myconfig: &State<MyConfig>,
) -> Template {
    Template::render(
        "register",
        context! {
            title: "Register",
            config: get_public_config(),
            visitor: Visitor::new(cookies, dbh, myconfig).await,
        },
    )
}

#[post("/register", data = "<input>")]
async fn register_post(
    cookies: &CookieJar<'_>,
    dbh: &State<Surreal<Client>>,
    myconfig: &State<MyConfig>,
    input: Form<RegistrationForm<'_>>,
) -> Template {
    rocket::info!("rocket input: {:?} {:?}", input.email, input.name);

    let config = get_public_config();
    let visitor = Visitor::new(cookies, dbh, myconfig).await;

    // email: lowerase, remove spaces from sides
    // validate format @
    let email = input.email.to_lowercase().trim().to_owned();
    if !validator::validate_email(&email) {
        return Template::render(
            "message",
            context! {title: "Invalid email address", message: format!("Invalid email address <b>{}</b> Please try again", input.email), config, visitor},
        );
    }

    let password = input.password.trim().as_bytes();
    let pw_min_length = 6;
    if password.len() < pw_min_length {
        return Template::render(
            "message",
            context! {title: "Invalid password", message: format!("The password must be at least {pw_min_length} characters long."), config, visitor},
        );
    }
    let process = "register";
    let code = Uuid::new_v4();
    let salt = SaltString::generate(&mut OsRng);
    let hashed_password = match Pbkdf2.hash_password(password, &salt) {
        Ok(val) => val.to_string(),
        Err(err) => {
            rocket::error!("Error: {err}");
            return Template::render(
                "message",
                context! {title: "Invalid password", message: format!("The password must be at least {pw_min_length} characters long."), config, visitor},
            );
        }
    };

    let uid = db::increment(dbh, "user").await.unwrap();
    let utc: DateTime<Utc> = Utc::now();

    let user = User {
        uid,
        name: input.name.to_owned(),
        email,
        password: hashed_password,
        process: process.to_owned(),
        code: format!("{code}"),
        registration_date: utc,
        verification_date: None,
        verified: false,
        github: None,
        gitlab: None,
        linkedin: None,
        about: None,
    };
    match db::add_user(dbh, &user).await {
        Ok(result) => result,
        Err(err) => {
            rocket::info!("Error while trying to add user {err}");
            // TODO special reporting when the email is already in the system
            return Template::render(
                "message",
                context! {title: "Registration failed", message: format!("Could not register <b>{}</b>.", user.email), config, visitor},
            );
        }
    };

    let base_url = &myconfig.base_url;
    let subject = "Verify your Meet-OS registration!";
    let text = format!(
        r#"Hi,
    Someone used your email to register on the Meet-OS web site.
    If it was you, please <a href="{base_url}/verify/{process}/{code}">click on this link</a> to verify your email address.
    <p>
    <p>
    If it was not you, we would like to apologize. You don't need to do anything. We'll discard your registration if it is not validated.
    ";
    "#
    );

    let from = EmailAddress {
        name: myconfig.from_name.clone(),
        email: myconfig.from_email.clone(),
    };
    let to_address = &EmailAddress {
        name: input.name.to_owned(),
        email: input.email.to_owned(),
    };

    sendmail(myconfig, &from, to_address, subject, &text).await;
    notify::admin_new_user_registered(myconfig, &user).await;

    Template::render(
        "message",
        context! {title: "We sent you an email", message: format!("We sent you an email to <b>{}</b> Please check your inbox and verify your email address.", to_address.email), config, visitor},
    )

    // Template::render(
    //     "register",
    //     context! {title: "Register", config: get_public_config()},
    // )
}

// TODO limit the possible values for the process to register and login
#[get("/verify/<process>/<code>")]
async fn verify(
    cookies: &CookieJar<'_>,
    dbh: &State<Surreal<Client>>,
    myconfig: &State<MyConfig>,
    process: &str,
    code: &str,
) -> Template {
    rocket::info!("process: {process}, code: {code}");

    let config = get_public_config();
    let visitor = Visitor::new(cookies, dbh, myconfig).await;

    // TODO take the process into account at the verification
    if let Ok(Some(user)) = db::verify_code(dbh, process, code).await {
        rocket::info!("verified: {}", user.email);
        cookies.add_private(("meet-os", user.email.clone())); // TODO this should be the user ID, right?
        let (title, message) = match process {
            "register" => ("Thank you for registering", "Your email was verified."),
            "login" => ("Welcome back", "Welcome back"),
            _ => ("Oups", "Big opus and TODO"),
        };

        notify::admin_new_user_verified(myconfig, &user).await;

        // take into account the newly set cookie value
        #[allow(clippy::shadow_unrelated)]
        let visitor = Visitor::new_after_login(&user.email, dbh, myconfig).await;

        return Template::render(
            "message",
            context! {title: title, message: message, config, visitor},
        );
    }
    Template::render(
        "message",
        context! {title: "Invalid code", message: format!("Invalid code <b>{code}</b>"), config, visitor},
    )
}

#[get("/join-group?<gid>")]
async fn join_group_get(
    cookies: &CookieJar<'_>,
    dbh: &State<Surreal<Client>>,
    myconfig: &State<MyConfig>,
    gid: usize,
) -> Template {
    let config = get_public_config();
    let visitor = Visitor::new(cookies, dbh, myconfig).await;

    if !visitor.logged_in {
        return Template::render(
            "message",
            context! {title: "Not logged in", message: format!("It seems you are not logged in"), config, visitor},
        );
    };

    let group = db::get_group_by_gid(dbh, gid).await.unwrap();
    if group.is_none() {
        return Template::render(
            "message",
            context! {title: "No such group", message: "No such group", config, visitor},
        );
    }
    let group = group.unwrap();

    let user = visitor.user.clone().unwrap();
    let uid = visitor.user.clone().unwrap().uid;
    if uid == group.owner {
        return Template::render(
            "message",
            context! {title: "You are the owner of this group", message: "You are the owner of this group", config, visitor},
        );
    }

    // TODO if uid is already a member - reject

    db::join_group(dbh, gid, uid).await.unwrap();
    db::audit(dbh, format!("User {uid} joined group {gid}"))
        .await
        .unwrap();
    notify::owner_user_joined_group(dbh, myconfig, &user, &group).await;
    Template::render(
        "message",
        context! {title: "Membership", message: format!(r#"User added to <a href="/group/{gid}">group</a>"#), config, visitor},
    )
}

#[get("/leave-group?<gid>")]
async fn leave_group_get(
    cookies: &CookieJar<'_>,
    dbh: &State<Surreal<Client>>,
    myconfig: &State<MyConfig>,
    gid: usize,
) -> Template {
    let config = get_public_config();
    let visitor = Visitor::new(cookies, dbh, myconfig).await;

    if !visitor.logged_in {
        return Template::render(
            "message",
            context! {title: "Not logged in", message: format!("It seems you are not logged in"), config, visitor},
        );
    };

    let group = db::get_group_by_gid(dbh, gid).await.unwrap();
    if group.is_none() {
        return Template::render(
            "message",
            context! {title: "No such group", message: "No such group", config, visitor},
        );
    }
    let group = group.unwrap();

    let user = visitor.user.clone().unwrap();
    let uid = visitor.user.clone().unwrap().uid;
    if uid == group.owner {
        return Template::render(
            "message",
            context! {title: "You are the owner of this group", message: "You are the owner of this group", config, visitor},
        );
    }

    // TODO if uid is not a member - reject

    db::leave_group(dbh, gid, uid).await.unwrap();
    notify::owner_user_left_group(dbh, myconfig, &user, &group).await;
    db::audit(dbh, format!("User {uid} left group {gid}"))
        .await
        .unwrap();

    Template::render(
        "message",
        context! {title: "Membership", message: format!(r#"User removed from <a href="/group/{gid}">group</a>"#), config, visitor},
    )
}

#[get("/profile")]
async fn show_profile(
    cookies: &CookieJar<'_>,
    dbh: &State<Surreal<Client>>,
    myconfig: &State<MyConfig>,
) -> Template {
    let config = get_public_config();
    let visitor = Visitor::new(cookies, dbh, myconfig).await;

    if !visitor.logged_in {
        return Template::render(
            "message",
            context! {title: "Not logged in", message: format!("It seems you are not logged in"), config, visitor},
        );
    };

    let uid = visitor.user.clone().unwrap().uid;
    let owned_groups = db::get_groups_by_owner_id(dbh, uid).await.unwrap();

    let groups = db::get_groups_by_membership_id(dbh, uid).await.unwrap();
    rocket::info!("groups: {groups:?}");

    let about = visitor
        .user
        .clone()
        .unwrap()
        .about
        .map(|text| markdown2html(&text).unwrap());

    Template::render(
        "profile",
        context! {title: "Profile", user: visitor.user.clone(), about, owned_groups, groups, config, visitor},
    )
}

#[get("/edit-profile")]
async fn edit_profile_get(
    cookies: &CookieJar<'_>,
    dbh: &State<Surreal<Client>>,
    myconfig: &State<MyConfig>,
) -> Template {
    let config = get_public_config();
    let visitor = Visitor::new(cookies, dbh, myconfig).await;

    if !visitor.logged_in {
        return Template::render(
            "message",
            context! {title: "Not logged in", message: format!("It seems you are not logged in"), config, visitor},
        );
    };

    Template::render(
        "edit_profile",
        context! {title: "Edit Profile", user: visitor.user.clone(), config, visitor},
    )
}

#[post("/edit-profile", data = "<input>")]
async fn edit_profile_post(
    cookies: &CookieJar<'_>,
    dbh: &State<Surreal<Client>>,
    myconfig: &State<MyConfig>,
    input: Form<ProfileForm<'_>>,
) -> Template {
    let config = get_public_config();
    let visitor = Visitor::new(cookies, dbh, myconfig).await;

    if !visitor.logged_in {
        return Template::render(
            "message",
            context! {title: "Not logged in", message: format!("It seems you are not logged in"), config, visitor},
        );
    };

    let uid = visitor.user.clone().unwrap().uid;
    let name = input.name;
    let github = input.github;
    let gitlab = input.gitlab;
    let linkedin = input.linkedin;
    let about = input.about;
    db::update_user(dbh, uid, name, github, gitlab, linkedin, about)
        .await
        .unwrap();

    Template::render(
        "message",
        context! {title: "Profile updated", message: format!(r#"Check out the <a href="/profile">profile</a> and how others see it <a href="/user/{uid}">{name}</a>"#, ), config, visitor},
    )
}

#[get("/event/<id>")]
async fn event_get(
    cookies: &CookieJar<'_>,
    dbh: &State<Surreal<Client>>,
    myconfig: &State<MyConfig>,
    id: usize,
) -> Template {
    let visitor = Visitor::new(cookies, dbh, myconfig).await;
    let event = db::get_event_by_eid(dbh, id).await.unwrap().unwrap();
    let group = db::get_group_by_gid(dbh, event.group_id)
        .await
        .unwrap()
        .unwrap();

    let description = markdown2html(&event.description).unwrap();

    let utc: DateTime<Utc> = Utc::now();
    let editable = utc < event.date;

    Template::render(
        "event",
        context! {
            title: &event.title,
            event: &event,
            description,
            group,
            config: get_public_config(),
            visitor,
            editable,
        },
    )
}

#[get("/group/<gid>")]
async fn group_get(
    cookies: &CookieJar<'_>,
    dbh: &State<Surreal<Client>>,
    myconfig: &State<MyConfig>,
    gid: usize,
) -> Template {
    rocket::info!("group_get: {gid}");
    let config = get_public_config();
    let visitor = Visitor::new(cookies, dbh, myconfig).await;

    let group = match db::get_group_by_gid(dbh, gid).await {
        Ok(group) => match group {
            Some(group) => group,
            None => {
                return Template::render(
                    "message",
                    context! {title: "No such group", message: "No such group", config, visitor},
                )
            } // TODO 404
        },
        Err(err) => {
            rocket::error!("Error: {err}");
            return Template::render(
                "message",
                context! {title: "Internal error", message: "Internal error", config, visitor},
            );
        }
    };

    let membership = if visitor.logged_in {
        db::get_membership(dbh, gid, visitor.clone().user.unwrap().uid)
            .await
            .unwrap()
    } else {
        None
    };

    let members = db::get_members_of_group(dbh, gid).await.unwrap();

    let events = db::get_events_by_group_id(dbh, gid).await;

    let description = markdown2html(&group.description).unwrap();
    let owner = db::get_user_by_id(dbh, group.owner).await.unwrap().unwrap();

    Template::render(
        "group",
        context! {
            title: &group.name,
            group: &group,
            description: description,
            events: events,
            config,
            visitor,
            owner,
            members,
            membership,
        },
    )
}

#[get("/groups")]
async fn groups_get(
    cookies: &CookieJar<'_>,
    dbh: &State<Surreal<Client>>,
    myconfig: &State<MyConfig>,
) -> Template {
    let config = get_public_config();
    let visitor = Visitor::new(cookies, dbh, myconfig).await;

    match db::get_groups(dbh).await {
        Ok(groups) => Template::render(
            "groups",
            context! {title: "Groups", groups: groups, config, visitor},
        ),
        Err(err) => {
            rocket::error!("Error {err}");
            Template::render(
                "message",
                context! {title: "Internal error", message: "Internal error", config, visitor},
            )
        }
    }

    // if let Ok(groups) = db::get_groups_from_database(dbh).await {
    //     return Template::render(
    //         "groups",
    //         context! {title: "Groups", groups: groups, config: get_public_config(), visitor},
    //     );
    // }
    // Template::render(
    //     "message",
    //     context! {title: "Internal error", message: "Internal error", config: get_public_config(), visitor},
    // )
}

#[get("/users")]
async fn list_users(
    cookies: &CookieJar<'_>,
    dbh: &State<Surreal<Client>>,
    myconfig: &State<MyConfig>,
) -> Template {
    let config = get_public_config();

    let visitor = Visitor::new(cookies, dbh, myconfig).await;

    if !visitor.logged_in {
        return Template::render(
            "message",
            context! {title: "Not logged in", message: format!("It seems you are not logged in"), config, visitor},
        );
    };

    rocket::info!(
        "cookie value received from user: {}",
        visitor.user.clone().unwrap().email
    );

    // TODO filtering  could be moved to the database call
    let all_users = db::get_users(dbh).await.unwrap();
    let users = all_users
        .into_iter()
        .filter(|user| user.verified)
        .collect::<Vec<_>>();

    Template::render(
        "users",
        context! {
            title: "List Users",
            config ,
            visitor,
            users,
        },
    )
}

#[get("/user/<uid>")]
async fn user(
    cookies: &CookieJar<'_>,
    dbh: &State<Surreal<Client>>,
    myconfig: &State<MyConfig>,
    uid: usize,
) -> Template {
    let config = get_public_config();

    let visitor = Visitor::new(cookies, dbh, myconfig).await;

    if !visitor.logged_in {
        return Template::render(
            "message",
            context! {title: "Not logged in", message: format!("It seems you are not logged in"), config, visitor},
        );
    };

    let user = match db::get_user_by_id(dbh, uid).await.unwrap() {
        None => {
            return Template::render(
                "message",
                context! {title: "User not found", message: format!("This user does not exist"), config, visitor},
            )
        }
        Some(user) => user,
    };

    if !user.verified {
        return Template::render(
            "message",
            context! {title: "Unverified user", message: format!("This user has not verified his email address yet"), config, visitor},
        );
    }

    let about = user.clone().about.map(|text| markdown2html(&text).unwrap());
    let owned_groups = db::get_groups_by_owner_id(dbh, user.uid).await.unwrap();
    let groups = db::get_groups_by_membership_id(dbh, user.uid)
        .await
        .unwrap();

    Template::render(
        "user",
        context! {
            title: user.name.clone(),
            config ,
            visitor,
            user,
            about,
            groups,
            owned_groups
        },
    )
}

#[get("/edit-group?<gid>")]
async fn edit_group_get(
    cookies: &CookieJar<'_>,
    dbh: &State<Surreal<Client>>,
    myconfig: &State<MyConfig>,
    gid: usize,
) -> Template {
    let config = get_public_config();

    let visitor = Visitor::new(cookies, dbh, myconfig).await;

    if !visitor.logged_in {
        return Template::render(
            "message",
            context! {title: "Not logged in", message: format!("It seems you are not logged in"), config, visitor},
        );
    };

    let uid = visitor.user.clone().unwrap().uid;
    let group = db::get_group_by_gid(dbh, gid).await.unwrap().unwrap();

    if group.owner != uid {
        return Template::render(
            "message",
            context! {title: "Not the owner", message: format!("Not the owner"), config, visitor},
        );
    }

    Template::render(
        "edit_group",
        context! {
            title: "Edit Group",
            config: get_public_config(),
            visitor,
            gid,
            group
        },
    )
}

#[post("/edit-group", data = "<input>")]
async fn edit_group_post(
    cookies: &CookieJar<'_>,
    dbh: &State<Surreal<Client>>,
    myconfig: &State<MyConfig>,
    input: Form<GroupForm<'_>>,
) -> Template {
    let config = get_public_config();

    let visitor = Visitor::new(cookies, dbh, myconfig).await;

    if !visitor.logged_in {
        return Template::render(
            "message",
            context! {title: "Not logged in", message: format!("It seems you are not logged in"), config, visitor},
        );
    };

    let uid = visitor.user.clone().unwrap().uid;
    let gid = input.gid;
    let group = db::get_group_by_gid(dbh, gid).await.unwrap().unwrap();

    if group.owner != uid {
        return Template::render(
            "message",
            context! {title: "Not the owner", message: format!("Not the owner"), config, visitor},
        );
    }

    let name = input.name;
    let location = input.location;
    let description = input.description;
    db::update_group(dbh, gid, name, location, description)
        .await
        .unwrap();

    Template::render(
        "message",
        context! {title: "Group updated", message: format!(r#"Check out the <a href="/group/{gid}">group</a>"#, ), config, visitor},
    )
}

#[get("/add-event?<gid>")]
async fn add_event_get(
    cookies: &CookieJar<'_>,
    dbh: &State<Surreal<Client>>,
    myconfig: &State<MyConfig>,
    gid: usize,
) -> Template {
    rocket::info!("add-event to {gid}");
    let config = get_public_config();

    let visitor = Visitor::new(cookies, dbh, myconfig).await;

    if !visitor.logged_in {
        return Template::render(
            "message",
            context! {title: "Not logged in", message: format!("It seems you are not logged in"), config, visitor},
        );
    };

    let uid = visitor.user.clone().unwrap().uid;
    let group = db::get_group_by_gid(dbh, gid).await.unwrap().unwrap();

    if group.owner != uid {
        return Template::render(
            "message",
            context! {title: "Not the owner", message: format!("Not the owner"), config, visitor},
        );
    }

    Template::render(
        "edit_event",
        context! {
            title: format!("Add event to the '{}' group", group.name),
            config: get_public_config(),
            visitor: Visitor::new(cookies, dbh, myconfig).await,
            gid: gid,
            group,
        },
    )
}

#[post("/edit-event", data = "<input>")]
async fn add_event_post(
    cookies: &CookieJar<'_>,
    dbh: &State<Surreal<Client>>,
    myconfig: &State<MyConfig>,
    input: Form<EventForm<'_>>,
) -> Template {
    rocket::info!("input: gid: {:?} title: '{:?}'", input.gid, input.title);

    let config = get_public_config();

    let visitor = Visitor::new(cookies, dbh, myconfig).await;

    if !visitor.logged_in {
        return Template::render(
            "message",
            context! {title: "Not logged in", message: format!("It seems you are not logged in"), config, visitor},
        );
    };

    let uid = visitor.user.clone().unwrap().uid;
    let group = db::get_group_by_gid(dbh, input.gid).await.unwrap().unwrap();

    if group.owner != uid {
        return Template::render(
            "message",
            context! {title: "Not the owner", message: format!("Not the owner"), config, visitor},
        );
    }

    let min_title_length = 10;
    let title = input.title.to_owned();
    if title.len() < min_title_length {
        return Template::render(
            "message",
            context! {title: "Too short a title", message: format!("Minimal title length {} Current title len: {}", min_title_length, title.len()), config, visitor},
        );
    }
    // TODO: no < in title

    let description = input.description.to_owned();
    // TODO validate the description - disable < character

    let location = input.location.to_owned();

    let date_str = input.date.to_owned();
    let offset = input.offset.to_owned();
    let mydate = format!("{date_str}:00 +00:00");
    let Ok(ts) = DateTime::parse_from_str(&mydate, "%Y-%m-%d %H:%M:%S %z") else {
        return Template::render(
            "message",
            context! {title: "Invalid date", message: format!("Invalid date '{}' offset '{}'", date_str, offset), config, visitor},
        );
    };

    #[allow(clippy::arithmetic_side_effects)]
    let date = ts.to_utc() + Duration::minutes(offset);

    let utc: DateTime<Utc> = Utc::now();
    if date < utc {
        return Template::render(
            "message",
            context! {title: "Can't schedule event to the past", message: format!("Can't schedule event to the past '{}'", date), config, visitor},
        );
    }

    let eid = db::increment(dbh, "event").await.unwrap();

    let event = Event {
        eid,
        title: title.clone(),
        description,
        date,
        location,
        group_id: input.gid,
    };
    match db::add_event(dbh, &event).await {
        Ok(result) => result,
        Err(err) => {
            rocket::info!("Error while trying to add event {err}");
            // TODO special reporting when the email is already in the system
            return Template::render(
                "message",
                context! {title: "Adding event failed", message: "Could not add event.", config, visitor},
            );
        }
    };

    Template::render(
        "message",
        context! {title: "Event added", message: format!(r#"Event added: <a href="/event/{}">{}</a>"#, eid, title ), config, visitor},
    )
}

#[get("/contact-members?<gid>")]
async fn contact_members_get(
    cookies: &CookieJar<'_>,
    dbh: &State<Surreal<Client>>,
    myconfig: &State<MyConfig>,
    gid: usize,
) -> Template {
    let config = get_public_config();

    let visitor = Visitor::new(cookies, dbh, myconfig).await;

    if !visitor.logged_in {
        return Template::render(
            "message",
            context! {title: "Not logged in", message: format!("It seems you are not logged in"), config, visitor},
        );
    };

    let uid = visitor.user.clone().unwrap().uid;
    let group = db::get_group_by_gid(dbh, gid).await.unwrap().unwrap();

    if group.owner != uid {
        return Template::render(
            "message",
            context! {title: "Not the owner", message: format!("Not the owner"), config, visitor},
        );
    }

    Template::render(
        "contact_members",
        context! {
            title: format!("Contact members of the '{}' group", group.name),
            config: get_public_config(),
            visitor: Visitor::new(cookies, dbh, myconfig).await,
            gid: gid,
            group,
        },
    )
}

#[post("/contact-members", data = "<input>")]
async fn contact_members_post(
    cookies: &CookieJar<'_>,
    dbh: &State<Surreal<Client>>,
    myconfig: &State<MyConfig>,
    input: Form<ContactMembersForm<'_>>,
) -> Template {
    let config = get_public_config();

    let visitor = Visitor::new(cookies, dbh, myconfig).await;

    if !visitor.logged_in {
        return Template::render(
            "message",
            context! {title: "Not logged in", message: format!("It seems you are not logged in"), config, visitor},
        );
    };

    let uid = visitor.user.clone().unwrap().uid;
    let group = db::get_group_by_gid(dbh, input.gid).await.unwrap().unwrap();

    if group.owner != uid {
        return Template::render(
            "message",
            context! {title: "Not the owner", message: format!("Not the owner"), config, visitor},
        );
    }

    let min_subject_length = 5;
    let subject = input.subject.to_owned();
    if subject.len() < min_subject_length {
        return Template::render(
            "message",
            context! {title: "Too short a subject", message: format!("Minimal subject length {} Current subject len: {}", min_subject_length, subject.len()), config, visitor},
        );
    }
    // TODO: no < in title

    let content = input.content.to_owned();
    let html = markdown2html(&content).unwrap();
    // TODO validate the content - disable < character

    notify::group_members(dbh, myconfig, &subject, &html, input.gid).await;

    Template::render(
        "message",
        context! {title: "Message sent", message: "Message sent", config, visitor},
    )
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/admin", admin::routes())
        .mount("/", public::routes())
        .mount(
            "/",
            routes![
                add_event_get,
                add_event_post,
                contact_members_get,
                contact_members_post,
                edit_group_get,
                edit_group_post,
                edit_profile_get,
                edit_profile_post,
                event_get,
                events,
                groups_get,
                group_get,
                index,
                join_group_get,
                leave_group_get,
                list_users,
                logout_get,
                login_get,
                login_post,
                register_get,
                register_post,
                show_profile,
                user,
                verify
            ],
        )
        .mount("/", FileServer::from(relative!("static")))
        .attach(Template::fairing())
        .attach(AdHoc::config::<MyConfig>())
        .attach(db::fairing())
}

#[cfg(test)]
mod test_simple;
