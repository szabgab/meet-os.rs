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

#[derive(Deserialize, Debug)]
struct PrivateConfig {
    sendgrid_api_key: String,
    admins: Vec<String>,
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

fn logged_in(cookies: &CookieJar<'_>) -> Option<CookieUser> {
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
async fn index(cookies: &CookieJar<'_>, db: &State<Surreal<Db>>) -> Result<Template, Template> {
    rocket::info!("home");

    let events = get_events_from_database(db).await.map_err(|err| {
        rocket::error!("Error: {err}");
        Template::render("message", context! {title: "Internal error", message: "Internal error", config: get_public_config(), logged_in: logged_in(cookies),},)
    })?;

    let groups = get_groups_from_database(db).await.map_err(|err| {
        rocket::error!("Error: {err}");
        Template::render("message", context! {title: "Internal error", message: "Internal error", config: get_public_config(), logged_in: logged_in(cookies),},)
    })?;

    Ok(Template::render(
        "index",
        context! {
            title: "Meet-OS",
            events: events,
            groups: groups,
            config: get_public_config(),
            logged_in: logged_in(cookies),
        },
    ))
}

#[get("/about")]
fn about(cookies: &CookieJar<'_>) -> Template {
    Template::render(
        "about",
        context! {
            title: "About Meet-OS",
            config: get_public_config(),
            logged_in: logged_in(cookies),
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
            logged_in: logged_in(cookies),
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
            logged_in: logged_in(cookies),
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
            logged_in: logged_in(cookies),
        },
    )
}

#[get("/logout")]
fn logout_get(cookies: &CookieJar<'_>) -> Template {
    // TODO shall we check if the cookie was even there?
    cookies.remove_private("meet-os");
    Template::render(
        "message",
        context! {title: "Logged out", message: "We have logged you out from the system", config: get_public_config(), logged_in: logged_in(cookies),},
    )
}

#[get("/login")]
fn login_get(cookies: &CookieJar<'_>) -> Template {
    Template::render(
        "login",
        context! {
            title: "Login",
            config: get_public_config(),
            logged_in: logged_in(cookies),
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

    let email = input.email.to_lowercase().trim().to_owned();
    if !validator::validate_email(&email) {
        return Template::render(
            "message",
            context! {title: "Invalid email address", message: format!("Invalid email address <b>{}</b>. Please try again", input.email), config: get_public_config(), logged_in: logged_in(cookies),},
        );
    }

    let user: User = match get_user_by_email(db, &email).await {
        Ok(user) => match user {
            Some(user) => user,
            None => {
                return Template::render(
                    "message",
                    context! {title: "No such user", message: format!("No user with address <b>{}</b>. Please try again", input.email), config: get_public_config(),logged_in: logged_in(cookies),},
                )
            }
        },
        Err(err) => {
            rocket::error!("Error: {err}");
            return Template::render(
                "message",
                context! {title: "No such user", message: format!("No user with address <b>{}</b>. Please try again", input.email), config: get_public_config(),logged_in: logged_in(cookies),},
            );
        }
    };

    rocket::info!("email: {}", user.email);

    let password = input.password.trim().as_bytes();

    let parsed_hash = match PasswordHash::new(&user.password) {
        Ok(val) => val,
        Err(err) => {
            rocket::error!("Error: {err}");
            return Template::render(
                "message",
                context! {title: "Internal error", message: "Internal error", config: get_public_config(), logged_in: logged_in(cookies),},
            );
        }
    };

    if Pbkdf2.verify_password(password, &parsed_hash).is_ok() {
        cookies.add_private(("meet-os", user.email)); // TODO this should be the user ID, right?
        Template::render(
            "message",
            context! {title: "Welcome back", message: r#"Welcome back. <a href="/profile">profile</a>"#, config: get_public_config(), logged_in: CookieUser {email}},
        )
    } else {
        Template::render(
            "message",
            context! {title: "Invalid password", message: "Invalid password", config: get_public_config(), logged_in: logged_in(cookies),},
        )
    }
}

// #[post("/reset-password", data = "<input>")]
// async fn reset_password_post(
//     cookies: &CookieJar<'_>,
//     db: &State<Surreal<Db>>,
//     input: Form<LoginForm<'_>>,
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

//     let base_url = rocket::Config::figment()
//         .extract_inner::<String>("base_url")
//         .unwrap_or_default();

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
//         let private = get_private_config();
//         sendgrid(&private.sendgrid_api_key, &from, to_address, subject, &text).await;
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
            logged_in: logged_in(cookies),
        },
    )
}

#[post("/register", data = "<input>")]
async fn register_post(
    cookies: &CookieJar<'_>,
    db: &State<Surreal<Db>>,
    input: Form<RegistrationForm<'_>>,
) -> Template {
    rocket::info!("rocket input: {:?} {:?}", input.email, input.name);

    // email: lowerase, remove spaces from sides
    // validate format @
    let email = input.email.to_lowercase().trim().to_owned();
    if !validator::validate_email(&email) {
        return Template::render(
            "message",
            context! {title: "Invalid email address", message: format!("Invalid email address <b>{}</b> Please try again", input.email), config: get_public_config(), logged_in: logged_in(cookies),},
        );
    }
    let password = input.password.trim().as_bytes();
    let pw_min_length = 6;
    if password.len() < pw_min_length {
        return Template::render(
            "message",
            context! {title: "Invalid password", message: format!("The password must be at least {pw_min_length} characters long."), config: get_public_config(), logged_in: logged_in(cookies),},
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
                context! {title: "Invalid password", message: format!("The password must be at least {pw_min_length} characters long."), config: get_public_config(), logged_in: logged_in(cookies),},
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
                context! {title: "Registration failed", message: format!("Could not register <b>{}</b>.", user.email), config: get_public_config(), logged_in: logged_in(cookies),},
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
        context! {title: "We sent you an email", message: format!("We sent you an email to <b>{}</b> Please check your inbox and verify your email address.", to_address.email), config: get_public_config(), logged_in: logged_in(cookies),},
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
            context! {title: title, message: message, config: get_public_config(),logged_in: logged_in(cookies),},
        );
    }
    Template::render(
        "message",
        context! {title: "Invalid code", message: format!("Invalid code <b>{code}</b>"), config: get_public_config(), logged_in: logged_in(cookies),},
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
                context! {title: "Profile", user: user, config: get_public_config(), logged_in: logged_in(cookies),},
            );
        }
    }

    Template::render(
        "message",
        context! {title: "Missing cookie", message: format!("It seems you are not logged in"), config: get_public_config(), logged_in: logged_in(cookies),},
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
            logged_in: logged_in(cookies),
        },
    )
}

#[get("/group/<gid>")]
async fn group_get(db: &State<Surreal<Db>>, cookies: &CookieJar<'_>, gid: usize) -> Template {
    rocket::info!("group_get: {gid}");
    let group = match get_group_by_gid(db, gid).await {
        Ok(group) => match group {
            Some(group) => group,
            None => {
                return Template::render(
                    "message",
                    context! {title: "No such group", message: "No such group", config: get_public_config(), logged_in: logged_in(cookies),},
                )
            } // TODO 404
        },
        Err(err) => {
            rocket::error!("Error: {err}");
            return Template::render(
                "message",
                context! {title: "Internal error", message: "Internal error", config: get_public_config(), logged_in: logged_in(cookies),},
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
            config: get_public_config(),
            logged_in: logged_in(cookies),
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
    match get_groups_from_database(db).await {
        Ok(groups) => Template::render(
            "groups",
            context! {title: "Groups", groups: groups, config: get_public_config(), logged_in: logged_in(cookies),},
        ),
        Err(err) => {
            rocket::error!("Error {err}");
            Template::render(
                "message",
                context! {title: "Internal error", message: "Internal error", config: get_public_config(), logged_in: logged_in(cookies),},
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

async fn is_admin(db: &State<Surreal<Db>>, email: &str) -> Option<User> {
    if let Ok(Some(user)) = get_user_by_email(db, email).await {
        rocket::info!("email: {}", user.email);
        let private = get_private_config();
        if private.admins.contains(&email.to_owned()) {
            return Some(user);
        }
    }

    None
}

#[get("/create-group")]
async fn create_group_get(db: &State<Surreal<Db>>, cookies: &CookieJar<'_>) -> Template {
    if let Some(login) = logged_in(cookies) {
        rocket::info!("cookie value received from user: {}", login.email);
        if let Some(user) = is_admin(db, &login.email).await {
            return Template::render(
                "create_group",
                context! {title: "Create Group", user: user, config: get_public_config(), logged_in: logged_in(cookies),},
            );
        }

        return Template::render(
            "message",
            context! {title: "Unauthorized", message: "Unauthorized", config: get_public_config(), logged_in: logged_in(cookies),},
        );
    }

    Template::render(
        "message",
        context! {title: "Not logged in", message: format!("It seems you are not logged in"), config: get_public_config(), logged_in: logged_in(cookies),},
    )
}

#[post("/create-group", data = "<input>")]
async fn create_group_post(
    cookies: &CookieJar<'_>,
    db: &State<Surreal<Db>>,
    input: Form<GroupForm<'_>>,
) -> Template {
    rocket::info!("create_group_post: {:?}", input.name);

    if let Some(login) = logged_in(cookies) {
        rocket::info!("cookie value received from user: {}", login.email);
        if let Some(_user) = is_admin(db, &login.email).await {
            let gid = match get_groups_from_database(db).await {
                Ok(groups) => groups.len().saturating_add(1),
                Err(err) => {
                    rocket::info!("Error while trying to add group {err}");
                    1
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
                Ok(result) => result,
                Err(err) => {
                    rocket::info!("Error while trying to add group {err}");
                    return Template::render(
                        "message",
                        context! {title: "Failed", message: format!("Could not add <b>{}</b>.", group.name), config: get_public_config(), logged_in: logged_in(cookies),},
                    );
                }
            };

            return Template::render(
                "message",
                context! {title: "Group created", message: format!(r#"Group <b><a href="/group/{}/{}</a></b>created"#, gid, group.name), config: get_public_config(), logged_in: logged_in(cookies),},
            );
        }
        return Template::render(
            "message",
            context! {title: "Unauthorized", message: "Unauthorized", config: get_public_config(), logged_in: logged_in(cookies),},
        );
    }

    Template::render(
        "message",
        context! {title: "Not logged in", message: format!("It seems you are not logged in"), config: get_public_config(), logged_in: logged_in(cookies),},
    )
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
}
