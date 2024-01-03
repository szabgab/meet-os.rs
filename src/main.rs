#[macro_use]
extern crate rocket;

use rocket::form::Form;
use rocket_dyn_templates::{context, Template};
use serde::{Deserialize, Serialize};
use std::fs::read_to_string;

#[derive(Deserialize, Serialize, Debug)]
struct Event {
    title: String,
}

#[derive(FromForm)]
struct RegistrationForm<'r> {
    name: &'r str,
    email: &'r str,
}

// TODO load n events to display on the front page, which n events?
fn load_events() -> Vec<Event> {
    let filename = "data/events/1.json";
    let json_string = read_to_string(filename).unwrap();
    let data: Event = serde_json::from_str(&json_string).expect("JSON parsing error");
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

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index, register_get, register_post])
        .attach(Template::fairing())
}

#[cfg(test)]
mod tests;
