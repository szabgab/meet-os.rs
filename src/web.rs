use serde::{Deserialize, Serialize};

use rocket::http::CookieJar;
use rocket::http::Status;
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

#[expect(clippy::struct_field_names)]
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LoggedIn {
    pub logged_in: bool,
    pub admin: bool,
    pub user: Option<User>,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for LoggedIn {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, ()> {
        rocket::info!("from_request");
        match Visitor::from_request(request).await {
            Outcome::Success(visitor) => {
                rocket::info!("from_request visitor: {visitor:?}");
                if visitor.logged_in {
                    let user = Self {
                        logged_in: visitor.logged_in,
                        admin: visitor.admin,
                        user: visitor.user,
                    };
                    Outcome::Success(user)
                } else {
                    Outcome::Error((Status::Unauthorized, ()))
                }
            }
            // This should never happen
            Outcome::Error(_) | Outcome::Forward(_) => {
                Outcome::Error((Status::InternalServerError, ()))
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AdminUser {
    pub logged_in: bool,
    pub admin: bool,
    pub user: Option<User>,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminUser {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, ()> {
        match LoggedIn::from_request(request).await {
            Outcome::Success(visitor) => {
                if visitor.admin {
                    let user = Self {
                        logged_in: visitor.logged_in,
                        admin: visitor.admin,
                        user: visitor.user,
                    };
                    Outcome::Success(user)
                } else {
                    Outcome::Error((Status::Forbidden, ()))
                }
            }
            Outcome::Error(err) => Outcome::Error(err),
            // This should never happen
            Outcome::Forward(_) => Outcome::Error((Status::InternalServerError, ())),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Visitor {
    pub logged_in: bool,
    pub admin: bool,
    pub user: Option<User>,
}

impl Visitor {
    pub async fn new(cookies: &CookieJar<'_>, dbh: &Surreal<Client>, myconfig: &MyConfig) -> Self {
        let mut me = Self {
            logged_in: false,
            admin: false,
            user: None,
        };

        if let Some(cookie_user) = get_logged_in(cookies) {
            rocket::info!("Email from cookie: {}", &cookie_user.email);
            if let Some(user) = db::get_user_by_email(dbh, &cookie_user.email)
                .await
                .unwrap()
            {
                me.logged_in = true;
                me.user = Some(user);
                //rocket::info!("email: {}", user.email);
                if myconfig.admins.contains(&cookie_user.email.clone()) {
                    me.admin = true;
                }
            } else {
                rocket::warn!(
                    "Could not find user with email: {} in the database",
                    &cookie_user.email
                );
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
            logged_in: false,
            admin: false,
            user: None,
        };
        rocket::info!("new_after_login");

        if let Some(user) = db::get_user_by_email(dbh, email).await.unwrap() {
            me.logged_in = true;
            me.user = Some(user);
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
impl<'r> FromRequest<'r> for Visitor {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, ()> {
        let cookies = request.cookies();
        let dbh = request.rocket().state::<Surreal<Client>>().unwrap();
        let myconfig = request.rocket().state::<MyConfig>().unwrap();

        Outcome::Success(Visitor::new(cookies, dbh, myconfig).await)
    }
}

fn get_logged_in(cookies: &CookieJar<'_>) -> Option<CookieUser> {
    if let Some(cookie) = cookies.get_private("meet-os") {
        let email = cookie.value();
        rocket::info!("get_logged_in: cookie value received from user: {email}");
        return Some(CookieUser {
            email: email.to_owned(),
        });
    }
    None
}
