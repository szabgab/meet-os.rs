use rocket::http::CookieJar;
use rocket::Route;
use rocket::State;

use rocket_dyn_templates::{context, Template};

use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;

use crate::web::Visitor;
use crate::{get_public_config, MyConfig};

pub fn routes() -> Vec<Route> {
    routes![admin]
}

#[get("/")]
async fn admin(
    cookies: &CookieJar<'_>,
    db: &State<Surreal<Client>>,
    myconfig: &State<MyConfig>,
) -> Template {
    let config = get_public_config();

    let visitor = Visitor::new(cookies, db, myconfig).await;

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

    if !visitor.admin {
        return Template::render(
            "message",
            context! {title: "Unauthorized", message: "Unauthorized", config, visitor},
        );
    }

    Template::render(
        "admin",
        context! {
            title: "Admin",
            config ,
            visitor,
        },
    )
}
