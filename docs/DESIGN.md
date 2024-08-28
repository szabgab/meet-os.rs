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
