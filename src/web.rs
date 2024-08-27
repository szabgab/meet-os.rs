use serde::{Deserialize, Serialize};

use rocket::http::CookieJar;
use rocket::outcome::Outcome;
use rocket::request::{self, FromRequest, Request};
use rocket::State;

use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;

use crate::db;
use meetings::{MyConfig, User};

#[derive(Serialize, Deserialize, Debug)]
pub struct CookieUser {
    email: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct VisitorGuard {
    pub logged_in: bool,
    pub admin: bool,
    pub user: Option<User>,
}

impl VisitorGuard {
    pub async fn new(cookies: &CookieJar<'_>, dbh: &Surreal<Client>, myconfig: &MyConfig) -> Self {
        let mut me = Self {
            logged_in: false,
            admin: false,
            user: None,
        };

        if let Some(cookie_user) = get_logged_in(cookies) {
            me.logged_in = true;
            if let Ok(user) = db::get_user_by_email(dbh, &cookie_user.email).await {
                me.user = user;
                //rocket::info!("email: {}", user.email);
                if myconfig.admins.contains(&cookie_user.email.clone()) {
                    me.admin = true;
                }
            }
        }

        me
    }

    pub async fn new_after_login(
        email: &str,
        dbh: &State<Surreal<Client>>,
        myconfig: &State<MyConfig>,
    ) -> Self {
        let mut me = Self {
            logged_in: true,
            admin: false,
            user: None,
        };

        if let Ok(user) = db::get_user_by_email(dbh, email).await {
            me.user = user;
            //rocket::info!("email: {}", user.email);
            if myconfig.admins.contains(&email.to_owned()) {
                me.admin = true;
            }
        }

        me
    }

    pub fn new_after_logout() -> Self {
        Self {
            logged_in: false,
            admin: false,
            user: None,
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for VisitorGuard {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, ()> {
        println!("from_request in GuardA '{request}'");

        let cookies = request.cookies();
        println!("cookies {cookies:?}");

        let dbh = request.rocket().state::<Surreal<Client>>().unwrap();
        println!("dbh {dbh:?}");

        let myconfig = request.rocket().state::<MyConfig>().unwrap();
        println!("myconfig {myconfig:?}");

        Outcome::Success(VisitorGuard::new(cookies, dbh, myconfig).await)
    }
}

fn get_logged_in(cookies: &CookieJar<'_>) -> Option<CookieUser> {
    if let Some(cookie) = cookies.get_private("meet-os") {
        let email = cookie.value();
        rocket::info!("cookie value received from user: {email}");
        return Some(CookieUser {
            email: email.to_owned(),
        });
    }
    None
}
