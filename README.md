# Meeting Rust


* A platform to organize meetings at [Meet-OS](https://meet-os.com/)


## Plan

* First implement whatever is necessary to organize the Rust-Maven online meetings.
* Then extend it to organize the Rust Israel meetings and maybe some other meetings I organize.
* Then talk to other Rust groups and see if I can win them over one by one.
* Then talk to Python groups and see if I can win them over one by one.
* Then extend it to other technology related group.

## Fees

* The organizations that I invite will get a lifetime free hosting. These will probably include some sponsored links. e.g. link to my own Rust, Python, etc. courses and job offers of sponsors.
* Later, if the platform is successful and the costs start to rise I'll think of a fee structure for new groups and we might add some features that will be enabled only for corporations.

## Source code, License, reusability

* The license of the source code is MIT OR Apache-2.0.
* Some of the text and the branding will have a different license.
* This will allow reuse of the application while making sure others can't legally pretend to run the same service.

## Costs

* The biggest cost will be the time spent on developing and maintaining the platform.
* Support for the users.
* UI Design, graphics.
* Hosting is probably less than $50 / month for a long time.
* Email sending - depending on the service, but it won't go over $50/month for quite some time either.

## Financing

* I hope that many people, especially in the Rust community, but also in the wider Open Source community will find the project valuable to support it financially.
* The platform will help promoting the training courses of Gabor Szabo, the main author. That can generate income.
* Income from companies that would like to promote their job offers through the system.


## Community

There is now a [Zulip stream](https://osdc.zulipchat.com/#narrow/stream/422181-meet-os) for discussion.

## Development

```
docker volume create my-surreal-db
docker run --name surrealdb --rm -p 127.0.0.1:8001:8000 --user root -v my-surreal-db:/database surrealdb/surrealdb:latest start --log trace file://database
```

```
git clone https://github.com/szabgab/meeting.rs
cd meeting.rs
```

* Install [pre-commit](https://pre-commit.com/)

```
pre-commit install
```

Copy `Rocket.skeleton.toml` to `Rocket.toml`

Create an account on Sendgrid and add your API key to the `[debug]` section of `Rocket.toml`:

```
sendgrid_api_key = "SG.blabla"
```

Create `config.yaml` adding the Google Analytics code:

```
google_analytics: G-SOME-CODE
```

Install [cargo-watch](https://github.com/watchexec/cargo-watch)

Run

```
cargo watch -x "run --bin meetings"
```

## Testing

The tests assume a running SurrealDB server. We setup a web server for each test run in a forked process and then
send http requests using the reqwest crate. We use environment variables to pass test-related configuration
options to the application.

The test run in a single thread configured in the `.cargo/config.toml` by setting `RUST_TEST_THREADS`to `1`.

The reason we text with a web server is probably historical. The in-process tests collided and I thought the solution
will be setting up external processes. Only after converting the tests and realizing those also fail I understood that
the problem is that the tests run in parallel threads and environment variables are per-process. So the use of the
environment variables is what force us to run the tests in a single thread.


```
cargo test
```


## Requirements

### For the Rust-Maven online meetings

Registration form, record the user name, email, generate id, generated and save validation code, timestamp send email with validation code.
When user clicks on link, mark as validated and remove the code.



New user
* register on the web site
* register to a group -> also to the web site
* register to an even -> also to the group -> also to the web site

Registered users who is not logged in
* login
* register to group -> login
* register to event -> 




* I can assume only a few tens maybe a few hundred people for the first few months so I think I can start with a filesystem based "database".

* I can send email notifications "manually" from the command line. There is no need for scheduling.
* Users need to be able to register on the web-site with email address. We need to verify the email address. (keep the email address lowercase)
    * Name
    * Email
    * Should we ask the user for a password as well or should we let the user login by getting a token to their email address?
    * Should we ask for a username?
* Users who have registered on the web site can mark themselves that they would like to attend an event or not. We probably should have at least 3 startes for this field.
    * By default the user is "has not replied yet"
    * then the user can "attend"
    * or "not attend".
    * Users who are not in the group will have none of these.
* As this is an online event and we don't need to limit the number of attendees.

* The user-id can bee a uuid
* The user information can be saved in a json file using the uuid as the name.


### Starting to organize the Rust-Israel meetings as well

* Create "groups".
* Allow the user to join and leave a group.
* When a user leaves a group, remove that person from all the future events of the group.


### Invite other group owners

* Create a new group on the command line
* Add an "owner" to a group.

* Web interface for the "owner" to create and update a new event.
* Web interface for the "owner" to write and email and send to
    * all the people in the group
    * all the people who registered to a specific event
    * all the people in the group who have not registered to a specific event



## Deploy

* We have nginx server configured as a reverse proxy in-front of the application.


I have a folder called `/home/gabor/work` with all of the projects. The deployment described here is relative to that.

```
cd /home/gabor/work
git clone git@github.com:szabgab/meet-os.rs.git
```

```
sudo cp /home/gabor/work/meet-os.rs/meet-os.service /etc/systemd/system/meet-os.service
sudo systemctl daemon-reload
sudo systemctl enable meet-os.service
sudo systemctl start meet-os.service
```

```
docker run --name surrealdb --detach --restart always --name surreal -p 127.0.0.1:8001:8000 --user root -v my-surreal-db:/database surrealdb/surrealdb:latest start --log trace file://database

docker stop surreal
docker restart surreal
```


## Release and deployment

```
ssh s7
cd /home/gabor/work/meet-os.rs/
git pull
cargo build --release
sudo systemctl restart meet-os.service
```

## Deploy the development and testing service


```
docker volume create my-surreal-db
docker run --name surrealdb --detach --restart always --name surreal -p 127.0.0.1:8000:8000 --user root -v my-surreal-db:/database surrealdb/surrealdb:latest start --log trace file://database

git clone git@github.com:szabgab/meet-os.rs.git dev.meet-os.com
git clone git@github.com:szabgab/meet-os.com-secrets.git
cd dev.meet-os.com
cp ../meet-os.com-secrets/dev/Rocket.toml .

sudo cp dev.meet-os.com.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable meet-os.service
sudo systemctl start meet-os.service

```
