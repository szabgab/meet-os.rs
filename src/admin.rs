use chrono::{DateTime, Utc};

use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::Route;
use rocket::State;

use rocket_dyn_templates::{context, Template};

use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;

use crate::db;
use crate::notify;
use crate::web::Visitor;
use crate::{get_public_config, MyConfig, User};
use meetings::Group;

#[derive(FromForm)]
struct GroupForm<'r> {
    name: &'r str,
    location: &'r str,
    description: &'r str,
    owner: usize,
}

#[derive(FromForm)]
struct SearchForm<'r> {
    query: &'r str,
    table: &'r str,
}

pub fn routes() -> Vec<Route> {
    routes![
        admin,
        admin_users,
        audit_get,
        create_group_get,
        create_group_post,
        search_get,
        search_post,
    ]
}

#[get("/")]
async fn admin(
    cookies: &CookieJar<'_>,
    dbh: &State<Surreal<Client>>,
    myconfig: &State<MyConfig>,
) -> Template {
    let config = get_public_config();

    let visitor = Visitor::new(cookies, dbh, myconfig).await;

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
    dbh: &State<Surreal<Client>>,
    myconfig: &State<MyConfig>,
) -> Template {
    let config = get_public_config();

    let visitor = Visitor::new(cookies, dbh, myconfig).await;

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

    let users = db::get_users(dbh).await.unwrap();

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

#[get("/search")]
async fn search_get(
    cookies: &CookieJar<'_>,
    dbh: &State<Surreal<Client>>,
    myconfig: &State<MyConfig>,
) -> Template {
    let config = get_public_config();
    let visitor = Visitor::new(cookies, dbh, myconfig).await;

    if !visitor.logged_in {
        return Template::render(
            "message",
            context! {title: "Not logged in", message: format!("It seems you are not logged in"), config, visitor},
        );
    };

    let user = visitor.user.clone().unwrap();

    if !visitor.admin {
        return Template::render(
            "message",
            context! {title: "Unauthorized", message: "Unauthorized", config, visitor},
        );
    };

    let users: Vec<User> = vec![];

    Template::render(
        "search",
        context! {title: "Search", query: "", table: "user", users, user: user, config, visitor},
    )
}

#[post("/search", data = "<input>")]
async fn search_post(
    cookies: &CookieJar<'_>,
    dbh: &State<Surreal<Client>>,
    myconfig: &State<MyConfig>,
    input: Form<SearchForm<'_>>,
) -> Template {
    rocket::info!("search_post: {:?}", input.query);
    let config = get_public_config();

    let visitor = Visitor::new(cookies, dbh, myconfig).await;

    if !visitor.logged_in {
        return Template::render(
            "message",
            context! {title: "Not logged in", message: format!("It seems you are not logged in"), config, visitor},
        );
    };

    let user = visitor.user.clone().unwrap();

    if !visitor.admin {
        return Template::render(
            "message",
            context! {title: "Unauthorized", message: "Unauthorized", config, visitor},
        );
    }

    let query = input.query.to_lowercase();

    let users = db::get_users(dbh)
        .await
        .unwrap()
        .into_iter()
        .filter(|usr| {
            usr.name.to_lowercase().contains(&query) || usr.email.to_lowercase().contains(&query)
        })
        .collect::<Vec<_>>();

    Template::render(
        "search",
        context! {title: "Search", query: input.query, table: input.table, users, user: user, config, visitor},
    )
}

#[get("/create-group?<uid>")]
async fn create_group_get(
    cookies: &CookieJar<'_>,
    dbh: &State<Surreal<Client>>,
    myconfig: &State<MyConfig>,
    uid: usize,
) -> Template {
    let config = get_public_config();
    let visitor = Visitor::new(cookies, dbh, myconfig).await;

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

    let owner = db::get_user_by_id(dbh, uid).await.unwrap().unwrap();

    Template::render(
        "create_group",
        context! {title: "Create Group", owner, user: user, config, visitor},
    )
}

#[post("/create-group", data = "<input>")]
async fn create_group_post(
    cookies: &CookieJar<'_>,
    dbh: &State<Surreal<Client>>,
    myconfig: &State<MyConfig>,
    input: Form<GroupForm<'_>>,
) -> Template {
    rocket::info!("create_group_post: {:?}", input.name);
    let config = get_public_config();

    let visitor = Visitor::new(cookies, dbh, myconfig).await;

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

    let gid = db::increment(dbh, "group").await.unwrap();
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
    let creation_date: DateTime<Utc> = Utc::now();
    let group = Group {
        name: input.name.to_owned(),
        location: input.location.to_owned(),
        description: input.description.to_owned(),
        owner: input.owner,
        gid,
        creation_date,
    };

    let owner = db::get_user_by_id(dbh, input.owner).await.unwrap().unwrap();

    match db::add_group(dbh, &group).await {
        Ok(_result) => {
            notify::owner_group_was_created(dbh, myconfig, &owner, &group).await;
            db::audit(dbh, format!("Group {gid} name: '{}' created.", group.name))
                .await
                .unwrap();
            Template::render(
                "message",
                context! {title: "Group created", message: format!(r#"Group <b><a href="/group/{}">{}</a></b>created"#, gid, group.name), config, visitor},
            )
        }
        Err(err) => {
            rocket::info!("Error while trying to add group {err}");
            Template::render(
                "message",
                context! {title: "Failed", message: format!("Could not add <b>{}</b>.", group.name), config, visitor},
            )
        }
    }
}

#[get("/audit")]
async fn audit_get(
    cookies: &CookieJar<'_>,
    dbh: &State<Surreal<Client>>,
    myconfig: &State<MyConfig>,
) -> Template {
    let config = get_public_config();
    let visitor = Visitor::new(cookies, dbh, myconfig).await;

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

    let audit = db::get_audit(dbh).await.unwrap();

    Template::render(
        "audit",
        context! {title: "Audit", audit, user: user, config, visitor},
    )
}
