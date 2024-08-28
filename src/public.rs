use rocket::Route;
use rocket_dyn_templates::{context, Template};

use crate::get_public_config;
use crate::web::Visitor;

pub fn routes() -> Vec<Route> {
    routes![about, privacy, soc, faq]
}

#[get("/about")]
fn about(visitor: Visitor) -> Template {
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
fn privacy(visitor: Visitor) -> Template {
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
fn soc(visitor: Visitor) -> Template {
    Template::render(
        "soc",
        context! {
            title: "Standard of Conduct",
            config: get_public_config(),
            visitor,
        },
    )
}

#[get("/faq")]
fn faq(visitor: Visitor) -> Template {
    Template::render(
        "faq",
        context! {
            title: "FAQ - Frequently Asked Questions",
            config: get_public_config(),
            visitor,
        },
    )
}
