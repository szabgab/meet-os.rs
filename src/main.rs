#[macro_use]
extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Welcome to the Rust meeting server"
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index])
}

#[cfg(test)]
mod tests;
