use rocket::http::CookieJar;
use rocket::Route;
use rocket::State;

use rocket_dyn_templates::{context, Template};

use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;

use crate::web::Visitor;
use crate::{get_public_config, get_users_from_database, MyConfig};

pub fn routes() -> Vec<Route> {
    routes![admin, admin_users]
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

#[get("/users")]
async fn admin_users(
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

    let users = get_users_from_database(db).await.unwrap();

    Template::render(
        "admin_users",
        context! {
            title: "List Users",
            config ,
            visitor,
            users,
        },
    )
}
