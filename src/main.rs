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

#[get("/")]
fn index() -> Template {
    let events = load_events();

    Template::render(
        "index",
        context! {
            events: events,
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
fn event_get(id: &str) -> Template {
    let events = load_events();
    let id = id.parse::<usize>().unwrap();
    Template::render(
        "event",
        context! {
            event: &events[id-1],
        },
    )
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index, register_get, register_post, event_get])
        .attach(Template::fairing())
}

#[cfg(test)]
mod tests;
