#[macro_use]
extern crate rocket;

use std::env;
use std::fs::{read_to_string, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use rocket::form::Form;
use rocket::fs::NamedFile;
use rocket::http::CookieJar;
use rocket::serde::uuid::Uuid;
use rocket::State;
use rocket_dyn_templates::{context, Template};
use serde::{Deserialize, Serialize};

use meetings::{
    add_login_code_to_user, add_user, db, get_events_by_group_id, get_user_by_email, load_event,
    load_events, load_group, load_groups, sendgrid, verify_code, EmailAddress, User,
};
use surrealdb::engine::local::Db;
use surrealdb::Surreal;

#[derive(Deserialize, Debug)]
struct PrivateConfig {
    sendgrid_api_key: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct PublicConfig {
    google_analytics: String,
}

#[derive(FromForm)]
struct RegistrationForm<'r> {
    name: &'r str,
    email: &'r str,
}

#[derive(FromForm)]
struct LoginForm<'r> {
    email: &'r str,
}

fn get_private_config() -> PrivateConfig {
    let filename = "private.yaml";
    let raw_string = read_to_string(filename).unwrap();
    let data: PrivateConfig = serde_yaml::from_str(&raw_string).expect("YAML parsing error");
    data
}

fn get_public_config() -> PublicConfig {
    let filename = "config.yaml";
    let raw_string = read_to_string(filename).unwrap();
    let data: PublicConfig = serde_yaml::from_str(&raw_string).expect("YAML parsing error");
    data
}

fn markdown2html(text: &str) -> Result<String, String> {
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
fn index() -> Template {
    rocket::info!("home");

    let events = load_events();
    let groups = load_groups();

    Template::render(
        "index",
        context! {
            title: "Meet-OS",
            events: events,
            groups: groups,
            config: get_public_config(),
        },
    )
}

#[get("/about")]
fn about() -> Template {
    Template::render(
        "about",
        context! {
            title: "About Meet-OS",
            config: get_public_config(),
        },
    )
}

#[get("/privacy")]
fn privacy() -> Template {
    Template::render(
        "privacy",
        context! {
            title: "Privacy Policy",
            config: get_public_config(),
        },
    )
}

#[get("/soc")]
fn soc() -> Template {
    Template::render(
        "soc",
        context! {
            title: "Standard of Conduct",
            config: get_public_config(),
        },
    )
}

#[get("/logout")]
fn logout_get(cookies: &CookieJar<'_>) -> Template {
    // TODO shall we check if the cookie was even there?
    cookies.remove_private("meet-os");
    Template::render(
        "message",
        context! {title: "Logged out", message: "We have logged you out from the system", config: get_public_config()},
    )
}

#[get("/login")]
fn login_get() -> Template {
    Template::render(
        "login",
        context! {
            title: "Login",
            config: get_public_config(),
        },
    )
}

#[post("/login", data = "<input>")]
async fn login_post(db: &State<Surreal<Db>>, input: Form<LoginForm<'_>>) -> Template {
    rocket::info!("rocket login: {:?}", input.email);

    let email = input.email.to_lowercase().trim().to_owned();
    if !validator::validate_email(&email) {
        return Template::render(
            "message",
            context! {title: "Invalid email address", message: format!("Invalid email address <b>{}</b>. Please try again", input.email), config: get_public_config()},
        );
    }

    let user: User = match get_user_by_email(db, &email).await {
        Ok(user) => match user {
            Some(user) => user,
            None => {
                return Template::render(
                    "message",
                    context! {title: "No such user", message: format!("No user with address <b>{}</b>. Please try again", input.email), config: get_public_config()},
                )
            }
        },
        Err(err) => {
            rocket::error!("Error: {err}");
            return Template::render(
                "message",
                context! {title: "No such user", message: format!("No user with address <b>{}</b>. Please try again", input.email), config: get_public_config()},
            );
        }
    };

    rocket::info!("email: {}", user.email);

    let process = "login";
    let code = Uuid::new_v4();

    match add_login_code_to_user(db, &email, process, code.to_string().as_str()).await {
        Ok(_result) => (),
        Err(err) => {
            rocket::info!("Error while trying to add user {err}");
            return Template::render(
                "message",
                context! {title: "Internal error", message: "Oups", config: get_public_config()},
            );
        }
    };

    let base_url = rocket::Config::figment()
        .extract_inner::<String>("base_url")
        .unwrap_or_default();

    let subject = "Verify your Meet-OS login!";
    let text = format!(
        r#"Hi,
    Someone used your email to try to login the Meet-OS web site.
    If it was you, please <a href="{base_url}/verify/{process}/{code}">click on this link</a> to finish the login process.
    <p>
    <p>
    If it was not you, we would like to apolozie. You don't need to do anything..
    ";
    "#
    );

    // TODO: read from some config file
    let from = EmailAddress {
        name: String::from("Meet OS"),
        email: String::from("gabor@szabgab.com"),
    };
    let to_address = &EmailAddress {
        name: user.name.clone(),
        email: input.email.to_owned(),
    };

    if let Ok(email_file) = env::var("EMAIL_FILE") {
        rocket::info!("email_file: {email_file}");
        let mut file = File::create(email_file).unwrap();
        writeln!(&mut file, "{}", &text).unwrap();
    } else {
        let private = get_private_config();
        sendgrid(&private.sendgrid_api_key, &from, to_address, subject, &text).await;
    }

    Template::render(
        "message",
        context! {title: "We sent you an email", message: format!("We sent you an email to <b>{}</b> Please click on the link to finish the login process.", to_address.email), config: get_public_config()},
    )
}

#[get("/register")]
fn register_get() -> Template {
    Template::render(
        "register",
        context! {
            title: "Register",
            config: get_public_config(),
        },
    )
}

#[post("/register", data = "<input>")]
async fn register_post(db: &State<Surreal<Db>>, input: Form<RegistrationForm<'_>>) -> Template {
    rocket::info!("rocket input: {:?} {:?}", input.email, input.name);

    // email: lowerase, remove spaces from sides
    // validate format @
    let email = input.email.to_lowercase().trim().to_owned();
    if !validator::validate_email(&email) {
        return Template::render(
            "message",
            context! {title: "Invalid email address", message: format!("Invalid email address <b>{}</b> Please try again", input.email), config: get_public_config()},
        );
    }
    let process = "register";
    let code = Uuid::new_v4();

    let user = User {
        name: input.name.to_owned(),
        email,
        process: process.to_owned(),
        code: format!("{code}"),
        date: "date".to_owned(), // TODO get current timestamp
        verified: false,
    };
    match add_user(db, &user).await {
        Ok(result) => result,
        Err(err) => {
            rocket::info!("Error while trying to add user {err}");
            // TODO special reporting when the email is already in the system
            return Template::render(
                "message",
                context! {title: "Registration failed", message: format!("Could not register <b>{}</b>.", user.email), config: get_public_config()},
            );
        }
    };

    let base_url = rocket::Config::figment()
        .extract_inner::<String>("base_url")
        .unwrap_or_default();

    let subject = "Verify your Meet-OS registration!";
    let text = format!(
        r#"Hi,
    Someone used your email to register on the Meet-OS web site.
    If it was you, please <a href="{base_url}/verify/{process}/{code}">click on this link</a> to verify your email address.
    <p>
    <p>
    If it was not you, we would like to apolozie. You don't need to do anything. We'll discard your registration if it is not validated.
    ";
    "#
    );

    // TODO: read from some config file
    let from = EmailAddress {
        name: String::from("Meet OS"),
        email: String::from("gabor@szabgab.com"),
    };
    let to_address = &EmailAddress {
        name: input.name.to_owned(),
        email: input.email.to_owned(),
    };

    if let Ok(email_file) = env::var("EMAIL_FILE") {
        rocket::info!("email_file: {email_file}");
        let mut file = File::create(email_file).unwrap();
        writeln!(&mut file, "{}", &text).unwrap();
    } else {
        let private = get_private_config();
        sendgrid(&private.sendgrid_api_key, &from, to_address, subject, &text).await;
    }

    Template::render(
        "message",
        context! {title: "We sent you an email", message: format!("We sent you an email to <b>{}</b> Please check your inbox and verify your email address.", to_address.email), config: get_public_config()},
    )

    // Template::render(
    //     "register",
    //     context! {title: "Register", config: get_public_config()},
    // )
}

// TODO limit the possible values for the process to register and login
#[get("/verify/<process>/<code>")]
async fn verify(
    db: &State<Surreal<Db>>,
    process: &str,
    code: &str,
    cookies: &CookieJar<'_>,
) -> Template {
    rocket::info!("process: {process}, code: {code}");

    // TODO take the process into account at the verification
    if let Ok(Some(user)) = verify_code(db, process, code).await {
        rocket::info!("verified: {}", user.email);
        cookies.add_private(("meet-os", user.email)); // TODO this should be the user ID, right?
        let (title, message) = match process {
            "register" => ("Thank you for registering", "Your email was verified."),
            "login" => ("Welcome back", r#"<a href="/profile">profile</a>"#),
            _ => ("Oups", "Big opus and TODO"),
        };
        return Template::render(
            "message",
            context! {title: title, message: message, config: get_public_config()},
        );
    }
    Template::render(
        "message",
        context! {title: "Invalid code", message: format!("Invalid code <b>{code}</b>"), config: get_public_config()},
    )
}

#[get("/profile")]
async fn show_profile(db: &State<Surreal<Db>>, cookies: &CookieJar<'_>) -> Template {
    if let Some(cookie) = cookies.get_private("meet-os") {
        let email = cookie.value();
        rocket::info!("cookie value received from user: {email}");
        if let Ok(Some(user)) = get_user_by_email(db, email).await {
            rocket::info!("email: {}", user.email);
            return Template::render(
                "profile",
                context! {title: "Profile", user: user, config: get_public_config()},
            );
        }
    }

    Template::render(
        "message",
        context! {title: "Missing cookie", message: format!("It seems you are not logged in"), config: get_public_config()},
    )
}

#[get("/event/<id>")]
fn event_get(id: usize) -> Template {
    let event = load_event(id);
    let group = load_group(event.group_id);

    let body = markdown2html(&event.body).unwrap();

    Template::render(
        "event",
        context! {
            title: &event.title,
            event: &event,
            body: body,
            group: group,
            config: get_public_config(),
        },
    )
}

#[get("/group/<id>")]
fn group_get(id: usize) -> Template {
    let group = load_group(id);
    let events = get_events_by_group_id(id);

    let description = markdown2html(&group.description).unwrap();

    Template::render(
        "group",
        context! {
            title: &group.name,
            group: &group,
            description: description,
            events: events,
            config: get_public_config(),
        },
    )
}

#[get("/js/<file..>")]
async fn js_files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/js/").join(file))
        .await
        .ok()
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount(
            "/",
            routes![
                about,
                event_get,
                group_get,
                index,
                js_files,
                logout_get,
                login_get,
                login_post,
                privacy,
                register_get,
                register_post,
                show_profile,
                soc,
                verify
            ],
        )
        .attach(Template::fairing())
        .attach(db::fairing())
}
