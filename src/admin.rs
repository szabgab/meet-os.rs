use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::Route;
use rocket::State;

use rocket_dyn_templates::{context, Template};

use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;

use crate::db::add_group;
use crate::web::Visitor;
use crate::{get_public_config, get_users_from_database, MyConfig};
use meetings::{increment, Group};

#[derive(FromForm)]
struct GroupForm<'r> {
    name: &'r str,
    location: &'r str,
    description: &'r str,
    owner: usize,
}

pub fn routes() -> Vec<Route> {
    routes![admin, admin_users, create_group_get, create_group_post,]
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

#[get("/create-group")]
async fn create_group_get(
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

    let user = visitor.user.clone().unwrap();

    rocket::info!("cookie value received from user: {}", user.email);
    if !visitor.admin {
        return Template::render(
            "message",
            context! {title: "Unauthorized", message: "Unauthorized", config, visitor},
        );
    };

    let users = get_users_from_database(db).await.unwrap();

    Template::render(
        "create_group",
        context! {title: "Create Group", users, user: user, config, visitor},
    )
}

#[post("/create-group", data = "<input>")]
async fn create_group_post(
    cookies: &CookieJar<'_>,
    db: &State<Surreal<Client>>,
    myconfig: &State<MyConfig>,
    input: Form<GroupForm<'_>>,
) -> Template {
    rocket::info!("create_group_post: {:?}", input.name);
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

    let gid = increment(db, "group").await.unwrap();
    // TODO verify that the given owner is a valid user-id (FOREIGN KEY should handle this)
    // //let owner = get_user_by_email(db, input.owner).await.unwrap();
    // if owner.is_none() {
    //     return Template::render(
    //         "message",
    //         context! {title: "Invalid email", message: "Invalid email", config, visitor},
    //     );
    // }
    //let owner_id = owner.unwrap().uid;

    rocket::info!("group_id: {gid}");
    let group = Group {
        name: input.name.to_owned(),
        location: input.location.to_owned(),
        description: input.description.to_owned(),
        owner: input.owner,
        gid,
    };

    match add_group(db, &group).await {
        Ok(_result) => Template::render(
            "message",
            context! {title: "Group created", message: format!(r#"Group <b><a href="/group/{}/{}</a></b>created"#, gid, group.name), config, visitor},
        ),
        Err(err) => {
            rocket::info!("Error while trying to add group {err}");
            Template::render(
                "message",
                context! {title: "Failed", message: format!("Could not add <b>{}</b>.", group.name), config, visitor},
            )
        }
    }
}
