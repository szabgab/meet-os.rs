#![allow(clippy::std_instead_of_core)]

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
use rocket::{fairing::AdHoc, State};
use rocket_dyn_templates::{context, Template};

use markdown::message;

use serde::{Deserialize, Serialize};

use pbkdf2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Pbkdf2,
};

use meetings::{
    add_group, add_user, db, get_events_by_group_id, get_events_from_database, get_group_by_gid,
    get_groups_from_database, get_user_by_email, load_event, load_group, sendgrid, verify_code,
    EmailAddress, Group, User,
};
use surrealdb::engine::local::Db;
use surrealdb::Surreal;

#[derive(Deserialize, Serialize, Debug)]
struct MyConfig {
    base_url: String,

    #[serde(default = "get_empty_string")]
    sendgrid_api_key: String,

    admins: Vec<String>,
}

fn get_empty_string() -> String {
    String::new()
}

#[derive(Deserialize, Serialize, Debug)]
struct PublicConfig {
    google_analytics: String,
}

#[derive(FromForm)]
struct GroupForm<'r> {
    name: &'r str,
    location: &'r str,
    description: &'r str,
}

#[derive(FromForm)]
struct RegistrationForm<'r> {
    name: &'r str,
    email: &'r str,
    password: &'r str,
}

#[derive(FromForm)]
struct LoginForm<'r> {
    email: &'r str,
    password: &'r str,
}

#[derive(Serialize, Deserialize, Debug)]
struct CookieUser {
    email: String,
}

fn get_public_config() -> PublicConfig {
    let filename = "config.yaml";
    let raw_string = read_to_string(filename).unwrap();
    let data: PublicConfig = serde_yaml::from_str(&raw_string).expect("YAML parsing error");
    data
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

fn get_logged_in(cookies: &CookieJar<'_>) -> Option<CookieUser> {
    if let Some(cookie) = cookies.get_private("meet-os") {
        let email = cookie.value();
        rocket::info!("cookie value received from user: {email}");
        return Some(CookieUser {
            email: email.to_owned(),
        });
    }
    None
}

#[get("/")]
async fn index(cookies: &CookieJar<'_>, db: &State<Surreal<Db>>) -> Template {
    rocket::info!("home");
    let config = get_public_config();
    let logged_in = get_logged_in(cookies);

    let events = match get_events_from_database(db).await {
        Ok(val) => val,
        Err(err) => {
            rocket::error!("Error: {err}");
            return Template::render(
                "message",
                context! {title: "Internal error", message: "Internal error", config, logged_in},
            );
        }
    };

    let groups = match get_groups_from_database(db).await {
        Ok(val) => val,
        Err(err) => {
            rocket::error!("Error: {err}");
            return Template::render(
                "message",
                context! {title: "Internal error", message: "Internal error", config, logged_in},
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
            logged_in,
        },
    )
}

#[get("/about")]
fn about(cookies: &CookieJar<'_>) -> Template {
    Template::render(
        "about",
        context! {
            title: "About Meet-OS",
            config: get_public_config(),
            logged_in: get_logged_in(cookies),
        },
    )
}

#[get("/admin")]
fn admin(cookies: &CookieJar<'_>) -> Template {
    Template::render(
        "admin",
        context! {
            title: "Admin",
            config: get_public_config(),
            logged_in: get_logged_in(cookies),
        },
    )
}

#[get("/privacy")]
fn privacy(cookies: &CookieJar<'_>) -> Template {
    Template::render(
        "privacy",
        context! {
            title: "Privacy Policy",
            config: get_public_config(),
            logged_in: get_logged_in(cookies),
        },
    )
}

#[get("/soc")]
fn soc(cookies: &CookieJar<'_>) -> Template {
    Template::render(
        "soc",
        context! {
            title: "Standard of Conduct",
            config: get_public_config(),
            logged_in: get_logged_in(cookies),
        },
    )
}

#[get("/logout")]
fn logout_get(cookies: &CookieJar<'_>) -> Template {
    // TODO shall we check if the cookie was even there?
    cookies.remove_private("meet-os");
    Template::render(
        "message",
        context! {title: "Logged out", message: "We have logged you out from the system", config: get_public_config(), logged_in: None::<CookieUser>,},
    )
}

#[get("/login")]
fn login_get(cookies: &CookieJar<'_>) -> Template {
    Template::render(
        "login",
        context! {
            title: "Login",
            config: get_public_config(),
            logged_in: get_logged_in(cookies),
        },
    )
}

#[post("/login", data = "<input>")]
async fn login_post(
    cookies: &CookieJar<'_>,
    db: &State<Surreal<Db>>,
    input: Form<LoginForm<'_>>,
) -> Template {
    rocket::info!("rocket login: {:?}", input.email);

    let config = get_public_config();
    let logged_in = get_logged_in(cookies);

    let email = input.email.to_lowercase().trim().to_owned();
    if !validator::validate_email(&email) {
        return Template::render(
            "message",
            context! {title: "Invalid email address", message: format!("Invalid email address <b>{}</b>. Please try again", input.email), config, logged_in},
        );
    }

    let user = match get_user_by_email(db, &email).await {
        Ok(user) => user,
        Err(err) => {
            rocket::error!("Error: {err}");
            return Template::render(
                "message",
                context! {title: "No such user", message: format!("No user with address <b>{}</b>. Please try again", input.email), config,logged_in},
            );
        }
    };

    let Some(user) = user else {
        return Template::render(
            "message",
            context! {title: "No such user", message: format!("No user with address <b>{}</b>. Please try again", input.email), config,logged_in},
        );
    };

    rocket::info!("email: {}", user.email);

    let password = input.password.trim().as_bytes();

    let parsed_hash = match PasswordHash::new(&user.password) {
        Ok(val) => val,
        Err(err) => {
            rocket::error!("Error: {err}");
            return Template::render(
                "message",
                context! {title: "Internal error", message: "Internal error", config, logged_in},
            );
        }
    };

    if Pbkdf2.verify_password(password, &parsed_hash).is_ok() {
        cookies.add_private(("meet-os", user.email)); // TODO this should be the user ID, right?
        Template::render(
            "message",
            context! {title: "Welcome back", message: r#"Welcome back. <a href="/profile">profile</a>"#, config, logged_in: CookieUser {email}},
        )
    } else {
        Template::render(
            "message",
            context! {title: "Invalid password", message: "Invalid password", config, logged_in},
        )
    }
}

// #[post("/reset-password", data = "<input>")]
// async fn reset_password_post(
//     cookies: &CookieJar<'_>,
//     db: &State<Surreal<Db>>,
//     input: Form<LoginForm<'_>>,
//     myconfig: &State<MyConfig>,
// ) -> Template {
//     rocket::info!("rocket login: {:?}", input.email);

//     let email = input.email.to_lowercase().trim().to_owned();
//     if !validator::validate_email(&email) {
//         return Template::render(
//             "message",
//             context! {title: "Invalid email address", message: format!("Invalid email address <b>{}</b>. Please try again", input.email), config: get_public_config(), logged_in: logged_in(cookies),},
//         );
//     }

//     let user: User = match get_user_by_email(db, &email).await {
//         Ok(user) => match user {
//             Some(user) => user,
//             None => {
//                 return Template::render(
//                     "message",
//                     context! {title: "No such user", message: format!("No user with address <b>{}</b>. Please try again", input.email), config: get_public_config(),logged_in: logged_in(cookies),},
//                 )
//             }
//         },
//         Err(err) => {
//             rocket::error!("Error: {err}");
//             return Template::render(
//                 "message",
//                 context! {title: "No such user", message: format!("No user with address <b>{}</b>. Please try again", input.email), config: get_public_config(),logged_in: logged_in(cookies),},
//             );
//         }
//     };

//     let process = "login";
//     let code = Uuid::new_v4();

//     match add_login_code_to_user(db, &email, process, code.to_string().as_str()).await {
//         Ok(_result) => (),
//         Err(err) => {
//             rocket::info!("Error while trying to add user {err}");
//             return Template::render(
//                 "message",
//                 context! {title: "Internal error", message: "Oups", config: get_public_config(), logged_in: logged_in(cookies),},
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
//     If it was not you, we would like to apolozie. You don't need to do anything..
//     ";
//     "#
//     );

//     // TODO: read from some config file
//     let from = EmailAddress {
//         name: String::from("Meet OS"),
//         email: String::from("gabor@szabgab.com"),
//     };
//     let to_address = &EmailAddress {
//         name: user.name.clone(),
//         email: input.email.to_owned(),
//     };

//     if let Ok(email_file) = env::var("EMAIL_FILE") {
//         rocket::info!("email_file: {email_file}");
//         let mut file = File::create(email_file).unwrap();
//         writeln!(&mut file, "{}", &text).unwrap();
//     } else {
//         sendgrid(&myconfig.sendgrid_api_key, &from, to_address, subject, &text).await;
//     }

//     Template::render(
//         "message",
//         context! {title: "We sent you an email", message: format!("We sent you an email to <b>{}</b> Please click on the link to finish the login process.", to_address.email), config: get_public_config(), logged_in: logged_in(cookies),},
//     )
// }

#[get("/register")]
fn register_get(cookies: &CookieJar<'_>) -> Template {
    Template::render(
        "register",
        context! {
            title: "Register",
            config: get_public_config(),
            logged_in: get_logged_in(cookies),
        },
    )
}

#[post("/register", data = "<input>")]
async fn register_post(
    cookies: &CookieJar<'_>,
    db: &State<Surreal<Db>>,
    input: Form<RegistrationForm<'_>>,
    myconfig: &State<MyConfig>,
) -> Template {
    rocket::info!("rocket input: {:?} {:?}", input.email, input.name);

    let config = get_public_config();
    let logged_in = get_logged_in(cookies);

    // email: lowerase, remove spaces from sides
    // validate format @
    let email = input.email.to_lowercase().trim().to_owned();
    if !validator::validate_email(&email) {
        return Template::render(
            "message",
            context! {title: "Invalid email address", message: format!("Invalid email address <b>{}</b> Please try again", input.email), config, logged_in},
        );
    }

    let password = input.password.trim().as_bytes();
    let pw_min_length = 6;
    if password.len() < pw_min_length {
        return Template::render(
            "message",
            context! {title: "Invalid password", message: format!("The password must be at least {pw_min_length} characters long."), config, logged_in},
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
                context! {title: "Invalid password", message: format!("The password must be at least {pw_min_length} characters long."), config, logged_in},
            );
        }
    };

    let user = User {
        name: input.name.to_owned(),
        email,
        password: hashed_password,
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
                context! {title: "Registration failed", message: format!("Could not register <b>{}</b>.", user.email), config, logged_in},
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

    Template::render(
        "message",
        context! {title: "We sent you an email", message: format!("We sent you an email to <b>{}</b> Please check your inbox and verify your email address.", to_address.email), config, logged_in},
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

    let config = get_public_config();
    let logged_in = get_logged_in(cookies);

    // TODO take the process into account at the verification
    if let Ok(Some(user)) = verify_code(db, process, code).await {
        rocket::info!("verified: {}", user.email);
        cookies.add_private(("meet-os", user.email.clone())); // TODO this should be the user ID, right?
        let (title, message) = match process {
            "register" => ("Thank you for registering", "Your email was verified."),
            "login" => ("Welcome back", "Welcome back"),
            _ => ("Oups", "Big opus and TODO"),
        };
        return Template::render(
            "message",
            context! {title: title, message: message, config, logged_in: CookieUser {email: user.email},},
        );
    }
    Template::render(
        "message",
        context! {title: "Invalid code", message: format!("Invalid code <b>{code}</b>"), config, logged_in},
    )
}

#[get("/profile")]
async fn show_profile(db: &State<Surreal<Db>>, cookies: &CookieJar<'_>) -> Template {
    let config = get_public_config();
    let logged_in = get_logged_in(cookies);

    if let Some(cookie) = cookies.get_private("meet-os") {
        let email = cookie.value();
        rocket::info!("cookie value received from user: {email}");
        if let Ok(Some(user)) = get_user_by_email(db, email).await {
            rocket::info!("email: {}", user.email);
            return Template::render(
                "profile",
                context! {title: "Profile", user: user, config, logged_in},
            );
        }
    }

    Template::render(
        "message",
        context! {title: "Missing cookie", message: format!("It seems you are not logged in"), config, logged_in},
    )
}

#[get("/event/<id>")]
fn event_get(cookies: &CookieJar<'_>, id: usize) -> Template {
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
            logged_in: get_logged_in(cookies),
        },
    )
}

#[get("/group/<gid>")]
async fn group_get(db: &State<Surreal<Db>>, cookies: &CookieJar<'_>, gid: usize) -> Template {
    rocket::info!("group_get: {gid}");
    let config = get_public_config();
    let logged_in = get_logged_in(cookies);

    let group = match get_group_by_gid(db, gid).await {
        Ok(group) => match group {
            Some(group) => group,
            None => {
                return Template::render(
                    "message",
                    context! {title: "No such group", message: "No such group", config, logged_in},
                )
            } // TODO 404
        },
        Err(err) => {
            rocket::error!("Error: {err}");
            return Template::render(
                "message",
                context! {title: "Internal error", message: "Internal error", config, logged_in},
            );
        }
    };

    let events = get_events_by_group_id(db, gid).await;

    let description = markdown2html(&group.description).unwrap();

    Template::render(
        "group",
        context! {
            title: &group.name,
            group: &group,
            description: description,
            events: events,
            config,
            logged_in,
        },
    )
}

#[get("/js/<file..>")]
async fn js_files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/js/").join(file))
        .await
        .ok()
}

#[get("/groups")]
async fn groups_get(db: &State<Surreal<Db>>, cookies: &CookieJar<'_>) -> Template {
    let config = get_public_config();
    let logged_in = get_logged_in(cookies);

    match get_groups_from_database(db).await {
        Ok(groups) => Template::render(
            "groups",
            context! {title: "Groups", groups: groups, config, logged_in},
        ),
        Err(err) => {
            rocket::error!("Error {err}");
            Template::render(
                "message",
                context! {title: "Internal error", message: "Internal error", config, logged_in},
            )
        }
    }

    // if let Ok(groups) = get_groups_from_database(db).await {
    //     return Template::render(
    //         "groups",
    //         context! {title: "Groups", groups: groups, config: get_public_config(), logged_in: logged_in(cookies),},
    //     );
    // }
    // Template::render(
    //     "message",
    //     context! {title: "Internal error", message: "Internal error", config: get_public_config(), logged_in: logged_in(cookies),},
    // )
}

async fn is_admin(
    db: &State<Surreal<Db>>,
    myconfig: &State<MyConfig>,
    email: &str,
) -> Option<User> {
    if let Ok(Some(user)) = get_user_by_email(db, email).await {
        rocket::info!("email: {}", user.email);
        if myconfig.admins.contains(&email.to_owned()) {
            return Some(user);
        }
    }

    None
}

#[get("/create-group")]
async fn create_group_get(
    db: &State<Surreal<Db>>,
    cookies: &CookieJar<'_>,
    myconfig: &State<MyConfig>,
) -> Template {
    let config = get_public_config();
    let logged_in = get_logged_in(cookies);

    if logged_in.is_none() {
        return Template::render(
            "message",
            context! {title: "Not logged in", message: format!("It seems you are not logged in"), config, logged_in},
        );
    };
    let logged_in = logged_in.unwrap();

    rocket::info!("cookie value received from user: {}", logged_in.email);
    let Some(user) = is_admin(db, myconfig, &logged_in.email).await else {
        return Template::render(
            "message",
            context! {title: "Unauthorized", message: "Unauthorized", config, logged_in},
        );
    };

    Template::render(
        "create_group",
        context! {title: "Create Group", user: user, config, logged_in},
    )
}

#[post("/create-group", data = "<input>")]
async fn create_group_post(
    cookies: &CookieJar<'_>,
    db: &State<Surreal<Db>>,
    input: Form<GroupForm<'_>>,
    myconfig: &State<MyConfig>,
) -> Template {
    rocket::info!("create_group_post: {:?}", input.name);
    let config = get_public_config();

    let logged_in = get_logged_in(cookies);

    if logged_in.is_none() {
        return Template::render(
            "message",
            context! {title: "Not logged in", message: format!("It seems you are not logged in"), config, logged_in},
        );
    };
    let logged_in = logged_in.unwrap();

    rocket::info!("cookie value received from user: {}", logged_in.email);

    if is_admin(db, myconfig, &logged_in.email).await.is_none() {
        return Template::render(
            "message",
            context! {title: "Unauthorized", message: "Unauthorized", config, logged_in},
        );
    }

    let gid = match get_groups_from_database(db).await {
        Ok(groups) => groups.len().saturating_add(1),
        Err(err) => {
            rocket::info!("Error while trying to add group {err}");
            1 // TODO return internal error message
        }
    };

    rocket::info!("group_id: {gid}");
    let group = Group {
        name: input.name.to_owned(),
        location: input.location.to_owned(),
        description: input.description.to_owned(),
        gid,
    };

    match add_group(db, &group).await {
        Ok(_result) => Template::render(
            "message",
            context! {title: "Group created", message: format!(r#"Group <b><a href="/group/{}/{}</a></b>created"#, gid, group.name), config, logged_in},
        ),
        Err(err) => {
            rocket::info!("Error while trying to add group {err}");
            Template::render(
                "message",
                context! {title: "Failed", message: format!("Could not add <b>{}</b>.", group.name), config, logged_in},
            )
        }
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount(
            "/",
            routes![
                about,
                admin,
                create_group_get,
                create_group_post,
                event_get,
                groups_get,
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
        .attach(AdHoc::config::<MyConfig>())
}
