#[macro_use]
extern crate rocket;

use rocket::form::Form;
use rocket_dyn_templates::{context, Template};

#[derive(FromForm)]
struct RegistrationForm<'r> {
    name: &'r str,
    email: &'r str,
}

#[get("/")]
fn index() -> Template {
    Template::render("index", context! {})
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
