use rocket::Route;
use rocket_dyn_templates::{context, Template};

use crate::get_public_config;
use crate::web::VisitorGuard;

pub fn routes() -> Vec<Route> {
    routes![about, privacy, soc,]
}

#[get("/about")]
fn about(visitor: VisitorGuard) -> Template {
    Template::render(
        "about",
        context! {
            title: "About Meet-OS",
            config: get_public_config(),
            visitor,
        },
    )
}

#[get("/privacy")]
fn privacy(visitor: VisitorGuard) -> Template {
    Template::render(
        "privacy",
        context! {
            title: "Privacy Policy",
            config: get_public_config(),
            visitor,
        },
    )
}

#[get("/soc")]
fn soc(visitor: VisitorGuard) -> Template {
    Template::render(
        "soc",
        context! {
            title: "Standard of Conduct",
            config: get_public_config(),
            visitor,
        },
    )
}
