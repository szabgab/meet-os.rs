# Meeting Rust


* A platform to organize meetings


## Plan

* First implement whatever is necessary to organize the Rust-Maven online meetings.
* Then extend it to organize the Rust Israel meetings.
* Then talk to other Rust groups and see if I can win them over one by one.
* then talk to Python groups and see if I can win them over one by one.
* Then extend it to other technology related group.


## Fees

* The organization that I invite initially will get a lifetime free hosting.
* Later, if the platform is successful I'll think if other free structures for new groups.

## Financing

* Ask for donation from the users.
* Use the platform to offer training courses.

## Development


```
git clone https://github.com/szabgab/meeting.rs
cd meeting.rs
```

* Install [pre-commit](https://pre-commit.com/)

```
pre-commit install
```

## Requirements

### For the Rust-Maven online meetings

* I can assume only a few tens maybe a few hundred people for the first few months so I think I can start with a filesystem based "database".

* The event can be defined using a YAML file, necessary images can be add on the filesystem.
    * Title
    * Date, starting hour (timezone), length
    * Description: Markdown without HTML tags
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

copy the `meetings.service` to the server e.g. to `/home/gabor/rust/`

```
sudo ln -s /home/gabor/rust/meetings.service /usr/lib/systemd/system/meetings.service
sudo systemctl daemon-reload

sudo systemctl start meetings.service
```


## Release and deployment

```
cargo build --release
scp target/release/meetings s7:rust/
ssh sudo systemctl start meetings.service

```