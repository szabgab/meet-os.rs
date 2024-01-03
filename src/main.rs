#[macro_use]
extern crate rocket;

use rocket::form::Form;
use rocket_dyn_templates::{context, Template};
use serde::{Deserialize, Serialize};
use std::fs::read_to_string;

// TODO is there a better way to set the id of the event to the filename?
#[derive(Deserialize, Serialize, Debug)]
struct Event {
    #[serde(default = "get_empty_string")]
    id: String,
    title: String,
    date: String,
    body: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct Group {
    #[serde(default = "get_empty_string")]
    id: String,
    name: String,
    description: String,
}

fn get_empty_string() -> String {
    "".to_string()
}

#[derive(FromForm)]
struct RegistrationForm<'r> {
    name: &'r str,
    email: &'r str,
}

// TODO load n events to display on the front page, which n events?
fn load_events() -> Vec<Event> {
    let filename = "data/events/1.yaml";
    let raw_string = read_to_string(filename).unwrap();
    let mut data: Event = serde_yaml::from_str(&raw_string).expect("YAML parsing error");
    data.id = "1".to_string();
    vec![data]
}

fn load_group(id: usize) -> Group {
    let filename = format!("data/groups/{}.yaml", id);
    let raw_string = read_to_string(filename).unwrap();
    let mut data: Group = serde_yaml::from_str(&raw_string).expect("YAML parsing error");
    data.id = "1".to_string();
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
            events: events,
            groups: groups,
        },
    )
}

#[get("/register")]
fn register_get() -> Template {
    Template::render("register", context! {})
}

#[post("/register", data = "<input>")]
fn register_post(input: Form<RegistrationForm<'_>>) -> Template {
    println!("input: {:?} {:?}", input.email, input.name);
    // email: lowerase, remove spaces from sides
    // validate format @
    Template::render("register", context! {})
}

#[get("/e/<id>")]
fn event_get(id: usize) -> Template {
    let events = load_events();

    let body = markdown::to_html_with_options(
        &events[id - 1].body,
        &markdown::Options {
            compile: markdown::CompileOptions {
                allow_dangerous_html: true,
                ..markdown::CompileOptions::default()
            },
            ..markdown::Options::gfm()
        },
    )
    .unwrap();

    Template::render(
        "event",
        context! {
            event: &events[id-1],
            body: body,
        },
    )
}

#[get("/g/<id>")]
fn group_get(id: usize) -> Template {
    let group = load_group(id);

    let description = markdown::to_html_with_options(
        &group.description,
        &markdown::Options {
            compile: markdown::CompileOptions {
                allow_dangerous_html: true,
                ..markdown::CompileOptions::default()
            },
            ..markdown::Options::gfm()
        },
    )
    .unwrap();

    Template::render(
        "group",
        context! {
            group: &group,
            description: description,
        },
    )
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount(
            "/",
            routes![index, register_get, register_post, event_get, group_get],
        )
        .attach(Template::fairing())
}

#[cfg(test)]
mod tests;
