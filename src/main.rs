#[macro_use]
extern crate rocket;

use rocket::form::Form;
use rocket::log;
use rocket_dyn_templates::{context, Template};
use sendgrid::SGClient;
use sendgrid::{Destination, Mail};
use serde::{Deserialize, Serialize};
use std::fs::read_to_string;

#[derive(Deserialize, Debug)]
struct PrivateConfig {
    sendgrid_api_key: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct PublicConfig {
    google_analytics: String,
}

// TODO is there a better way to set the id of the event to the filename?
#[derive(Deserialize, Serialize, Debug)]
struct Event {
    #[serde(default = "get_empty_string")]
    id: String,
    title: String,
    date: String,
    location: String,
    group_id: usize,
    body: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct Group {
    #[serde(default = "get_empty_string")]
    id: String,
    name: String,
    location: String,
    description: String,
}

fn get_empty_string() -> String {
    String::new()
}

#[derive(FromForm)]
struct RegistrationForm<'r> {
    name: &'r str,
    email: &'r str,
}

fn load_event(id: usize) -> Event {
    let filename = format!("data/events/{id}.yaml");
    let raw_string = read_to_string(filename).unwrap();
    let mut data: Event = serde_yaml::from_str(&raw_string).expect("YAML parsing error");
    data.id = String::from("1");
    data
}

fn get_events_by_group_id(id: usize) -> Vec<Event> {
    let events = load_events();
    events
        .into_iter()
        .filter(|event| event.group_id == id)
        .collect()
}

// TODO load n events to display on the front page, which n events?
fn load_events() -> Vec<Event> {
    let data = load_event(1);
    vec![data]
}

fn load_group(id: usize) -> Group {
    let filename = format!("data/groups/{id}.yaml");
    let raw_string = read_to_string(filename).unwrap();
    let mut data: Group = serde_yaml::from_str(&raw_string).expect("YAML parsing error");
    data.id = String::from("1");
    data
}

// TODO load n groups to display on the front page
fn load_groups() -> Vec<Group> {
    let data = load_group(1);
    vec![data]
}

#[get("/")]
fn index() -> Template {
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

#[post("/register", data = "<input>")]
async fn register_post(input: Form<RegistrationForm<'_>>) -> Template {
    log::info_!("rocket input: {:?} {:?}", input.email, input.name);
    // email: lowerase, remove spaces from sides
    // validate format @
    let subject = "Verify your Meet-OS registration!";
    let text = "Hi,
    Someone used your email to register on the Meet-OS web site.
    If it was you, please click on this link to verify your email address.


    If it was not you, we would like to apolozie. You don't need to do anything. We'll discard your registration if it is not validated.
    ";

    let private = get_private_config();

    // TODO: read from some config file
    let from = EmailAddress {
        name: String::from("Meet OS"),
        email: String::from("gabor@szabgab.com"),
    };
    let to_address = &EmailAddress {
        name: input.name.to_owned(),
        email: input.email.to_owned(),
    };

    sendgrid(&private.sendgrid_api_key, &from, to_address, subject, text).await;

    Template::render(
        "register",
        context! {title: "Register", config: get_public_config()},
    )
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

use rocket::fs::NamedFile;
use std::path::{Path, PathBuf};

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
                index,
                about,
                privacy,
                soc,
                register_get,
                register_post,
                event_get,
                group_get,
                js_files
            ],
        )
        .attach(Template::fairing())
}

#[derive(Debug)]
struct EmailAddress {
    name: String,
    email: String,
}

async fn sendgrid(
    api_key: &str,
    from: &EmailAddress,
    to: &EmailAddress,
    subject: &str,
    html: &str,
) {
    let sg = SGClient::new(api_key);

    let mut x_smtpapi = String::new();
    x_smtpapi.push_str(r#"{"unique_args":{"test":7}}"#);

    let mail_info = Mail::new()
        .add_to(Destination {
            address: &to.email,
            name: &to.name,
        })
        .add_from(&from.email)
        .add_from_name(&from.name)
        .add_subject(subject)
        .add_html(html)
        .add_header("x-cool".to_owned(), "indeed")
        .add_x_smtpapi(&x_smtpapi);

    sg.send(mail_info).await.ok();
}

#[cfg(test)]
mod tests;
