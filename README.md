# Meeting Rust


* A platform to organize meetings at [Meet-OS](https://meet-os.com/)


## Plan

* First implement whatever is necessary to organize the Rust-Maven online meetings.
* Then extend it to organize the Rust Israel meetings and maybe some other meetings I organize.
* Then talk to other Rust groups and see if I can win them over one by one.
* Then talk to Python groups and see if I can win them over one by one.
* Then extend it to other technology related group.


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


## Community

There is now a [Zulip stream](https://osdc.zulipchat.com/#narrow/stream/422181-meet-os) for discussion.

## Development

* Start the Docker that Runs the SurrealDB

```
docker volume create my-surreal-db
docker run --name surrealdb --detach --restart always --name surreal -p 127.0.0.1:8000:8000 --user root -v my-surreal-db:/database surrealdb/surrealdb:latest start --log trace file://database
```

* At the end of the development session you might want to stop the docker container.
```
docker stop surreal
```

* For the next session you can start it again.

```
docker restart surreal
```

* Install [Rust](https://www.rust-lang.org/tools/install).

* Install [cargo-watch](https://github.com/watchexec/cargo-watch).

* Get the Source code:

```
git clone https://github.com/szabgab/meet-os.rs
cd meet-os.rs
```

* Install [pre-commit](https://pre-commit.com/)

* Set up the pre-commit:

```
pre-commit install
```

Copy `Rocket.skeleton.toml` to `Rocket.toml`

The system sends emails. You can either setup a [Sendgrid](https://sendgrid.com/) account (the free account might be enough during development)
or you might use the "file-based delivery" which basically means that every email is saved as a file. The file-based system is used for testing.

* If using sendgrid then create an account on Sendgrid and add your API key to the `[debug]` section of `Rocket.toml`:

```
email            = "Sendgrid"
sendgrid_api_key = "SG.blabla"
```

* If using the file-based delivery system then create and empty folder (e.g. emails/ in the current folder)  edit `Rocket.toml` and add
the full path to this folder.

```
email            = "Folder"
email_folder     = "/path/to/email_folder"
```


TODO: This needs rework:

Create `config.yaml` adding the Google Analytics code:

```
google_analytics: G-SOME-CODE
```

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

* Set up meet-os.com on https://forwardemail.net/
* Set up meet-os.com on https://sendgrid.com/

* We have nginx server configured as a reverse proxy in-front of the application.

* We use [SurrealDB](https://surrealdb.com/) in a Docker container.

```
docker volume create my-surreal-db
docker run --name surrealdb --detach --restart always --name surreal -p 127.0.0.1:8000:8000 --user root -v my-surreal-db:/database surrealdb/surrealdb:latest start --log trace file://database
```

I have a folder called `/home/gabor/work` with all of the projects. The deployment described here is relative to that.

```
cd /home/gabor/work
git clone git@github.com:szabgab/meet-os.rs.git meet-os.com
```

```
sudo cp /home/gabor/work/meet-os.com/meet-os.service /etc/systemd/system/meet-os.service
sudo systemctl daemon-reload
sudo systemctl enable meet-os.service
sudo systemctl start meet-os.service
```



## Release and deployment

```
ssh s7
cd /home/gabor/work/meet-os.com/
git pull
cargo build --release
sudo systemctl restart meet-os.service
```

## Deploy the development and testing service



```
docker volume create my-surreal-db
docker run --name surrealdb --detach --restart always --name surreal -p 127.0.0.1:8000:8000 --user root -v my-surreal-db:/database surrealdb/surrealdb:latest start --log trace file://database
```

git clone git@github.com:szabgab/meet-os.rs.git dev.meet-os.com
cd dev.meet-os.com
cargo build --release
```

Upload the Rocket.toml with the configuration of the dev server.

```
sudo cp dev.meet-os.com.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable dev.meet-os.com.service
sudo systemctl start dev.meet-os.com.service
```

### Upgrade the development and testing service

```
cd dev.meet-os.com
git pull
cargo build --release
sudo systemctl restart dev.meet-os.com.service
```


