## Plan

* First implement whatever is necessary to organize the Rust-Maven online meetings. Also setup groups for Python-Maven, Perl-Maven and the generic Code-Maven.
* Then extend it to organize the Rust Israel and Python Israel meetings and maybe some other meetings I organize.
* Then talk to other Rust groups and see if any of them would be interested in using the system. They can try it on the dev server and give feedback even before setting up an account on the production server.
* Then talk to Python groups and see if any of those would be interested.
* Then extend it to other technology related groups.


## Source code, License, re-usability

* The license of the source code is MIT OR Apache-2.0.
* Some of the text and the branding will have a different license.
* This will allow reuse of the application while making sure others can't legally pretend to run the same service.

## Costs

* Time spent on developing and maintaining the platform instead of providing training and consulting to clients.
* Support for the users.
* UI Design, graphics.
* Hosting cost.
* Email sending cost.
* Marketing.

## Permission levels
* Site Admin
* Group owner
* TODO: group admin
* TODO: event admin
* User
* Visitor (not logged in)

Mark registered users as **Site Admin** by adding their names to the `admins` field in the `Rocket.toml` configuration file.

The site admin can create a new group and assign a user to be the **GRoup owner**.


A **Visitor** can see the groups and the events. A visitor can register to the site and the login to become a **User**. A Visitor can ask to reset a password.

A **User** can also list the registered and verified users. A User can join a group and leave a group. A User can RSVP to an event or remove the RSVP from the event. 


## Requirements

### For the Rust-Maven online meetings

* Visitor
    * Register user
    * Reset password (forgotten password)
    * Login
    * Logout

* User
    * Edit user profile (name, description)
    * Join group / leave group
    * Join event / leave event
    * logout

* Owner:
    * create event
    * update all the fields of an event
    * Send notification to all the members of the group.
    * Send notification to all the users registered to a specific event.
    * Send notification to all the users NOT registered to a specific event.


* Admin:
    * create group assign to owner


## Processes

**Registration**
* form: name, email, password
* generate id
* generate email-validation code
* generated timestamp
* store in the database
* send email with a link containing the validation code to user.

* When the user clicks on link:
* mark the user in the db as validated
* remove the code
* save validation timestamp
* TODO: set an expiration on the code


**Reset password process**
* User fills email, we generate validation code; save the code in the db and send email to user.
* User clicks on the link with the code in the email and arrives to the site, we check if we have a user with that code and get the user, create form with the `uid` the `code` and a place for the new `password`.
* User fills the new password we verify that the supplied `id` and `code` belong together and if the password is good. Then save the password and remove the code from the db.

**Login**
* User types in email and password, we compare it to the hashed password in the database.
* Set a cookie.

**Logout**
* On click on the logout link.
* Remove the cookie.

**Create group**
* Only the Site Admin can create groups.
* Create a new group and add a user to a group to be the **owner**
* notify new owner

**Send email notification to group members**
* Log in
* On /profile user can find the owned groups 
* Go to page of the group or go to the page of an event in your group
* TODO Select the recipients
    * all the members
    * members who joined in the last N days
    * members active in the last (N days, N months)
    * members inactive in the last (N days, N months)
    * selected members (and allow the selection of members one-by-one)
    * members RSVP-ed YES to a specific event
    * members RSVP-ed NO to a specific event
    * members not RSVP-ed to a specific event
* Fill the subject and the content (use markdown)
* TODO: save the message so later we can see the messages we sent.
* TODO: preview
* TODO: send preview to group owner
* TODO: save as draft
* Send



New user
* register on the web site
* register to a group -> also to the web site
* register to an even -> also to the group -> also to the web site

Registered users who is not logged in
* login
* register to group -> login
* register to event -> 

**User leaving a group**
* When a user leaves a group, remove that person from all the future events of the group.
* notify owner



* I can assume a few hundred people for the first few months.

* I can send email notifications "manually" from the web interface.


* TODO: Automatic messages: When a new event is created, 1 day before a scheduled event etc.

* Users need to be able to register on the web-site with email address. We need to verify the email address. (keep the email address lowercase)
    * Name
    * Email
    * Should we ask the user for a password as well or should we let the user login by getting a token to their email address?
    * Should we ask for a username?
* Users who have registered on the web site can mark themselves that they would like to attend an event or not. We probably should have at least 3 states for this field.
    * By default the user is "has not replied yet"
    * then the user can "attend"
    * or "not attend".
    * Users who are not in the group will have none of these.
* As this is an online event and we don't need to limit the number of attendees.
