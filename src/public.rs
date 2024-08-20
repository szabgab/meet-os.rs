use rocket::http::CookieJar;
use rocket::Route;
use rocket::State;
use rocket_dyn_templates::{context, Template};

use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;

use crate::web::Visitor;
use crate::{get_public_config, MyConfig};

pub fn routes() -> Vec<Route> {
    routes![about, privacy, soc,]
}

#[get("/about")]
async fn about(
    cookies: &CookieJar<'_>,
    dbh: &State<Surreal<Client>>,
    myconfig: &State<MyConfig>,
) -> Template {
    Template::render(
        "about",
        context! {
            title: "About Meet-OS",
            config: get_public_config(),
            visitor: Visitor::new(cookies, dbh, myconfig).await,
        },
    )
}

#[get("/privacy")]
async fn privacy(
    cookies: &CookieJar<'_>,
    dbh: &State<Surreal<Client>>,
    myconfig: &State<MyConfig>,
) -> Template {
    Template::render(
        "privacy",
        context! {
            title: "Privacy Policy",
            config: get_public_config(),
            visitor: Visitor::new(cookies, dbh, myconfig).await,
        },
    )
}

#[get("/soc")]
async fn soc(
    cookies: &CookieJar<'_>,
    dbh: &State<Surreal<Client>>,
    myconfig: &State<MyConfig>,
) -> Template {
    Template::render(
        "soc",
        context! {
            title: "Standard of Conduct",
            config: get_public_config(),
            visitor: Visitor::new(cookies, dbh, myconfig).await,
        },
    )
}
