-- ------------------------------
-- OPTION
-- ------------------------------

OPTION IMPORT;

-- ------------------------------
-- TABLE: audit
-- ------------------------------

DEFINE TABLE audit TYPE ANY SCHEMALESS PERMISSIONS NONE;

-- ------------------------------
-- TABLE: counter
-- ------------------------------

DEFINE TABLE counter TYPE ANY SCHEMALESS PERMISSIONS NONE;

DEFINE INDEX counter_name ON counter FIELDS name UNIQUE;

-- ------------------------------
-- TABLE: event
-- ------------------------------

DEFINE TABLE event TYPE ANY SCHEMALESS PERMISSIONS NONE;

-- ------------------------------
-- TABLE: group
-- ------------------------------

DEFINE TABLE group TYPE ANY SCHEMALESS PERMISSIONS NONE;

DEFINE INDEX group_gid ON group FIELDS gid UNIQUE;

-- ------------------------------
-- TABLE: membership
-- ------------------------------

DEFINE TABLE membership TYPE ANY SCHEMALESS PERMISSIONS NONE;

DEFINE INDEX member_ship ON membership FIELDS uid, gid UNIQUE;

-- ------------------------------
-- TABLE: rsvp
-- ------------------------------

DEFINE TABLE rsvp TYPE ANY SCHEMALESS PERMISSIONS NONE;

DEFINE INDEX rsvp_index ON rsvp FIELDS uid, eid UNIQUE;

-- ------------------------------
-- TABLE: user
-- ------------------------------

DEFINE TABLE user TYPE ANY SCHEMALESS PERMISSIONS NONE;

DEFINE INDEX user_email ON user FIELDS email UNIQUE;
DEFINE INDEX user_uid ON user FIELDS uid UNIQUE;

-- ------------------------------
-- TRANSACTION
-- ------------------------------

BEGIN TRANSACTION;

-- ------------------------------
-- TABLE DATA: audit
-- ------------------------------

UPDATE audit:0ja3h1l9kxqgjdirdel7 CONTENT { date: s'2024-09-10T06:23:45.884302535Z', id: audit:0ja3h1l9kxqgjdirdel7, text: 'User 3 joined group 1' };
UPDATE audit:18jr5kec6m6ervrrmcz0 CONTENT { date: s'2024-08-25T07:30:06.795678529Z', id: audit:18jr5kec6m6ervrrmcz0, text: "Group 4 name: 'Group of the Admin' created." };
UPDATE audit:3pz4tglmwy1ry15qe6vv CONTENT { date: s'2024-08-22T01:47:07.543616259Z', id: audit:3pz4tglmwy1ry15qe6vv, text: 'User 2 joined group 2' };
UPDATE audit:45ieteia4vzk3j1bnolr CONTENT { date: s'2024-08-21T12:45:28.870044447Z', id: audit:45ieteia4vzk3j1bnolr, text: "Group 1 name: 'Code Maven' created." };
UPDATE audit:7qb2k2bo58qg0c3s86nf CONTENT { date: s'2024-08-21T13:00:44.759227876Z', id: audit:7qb2k2bo58qg0c3s86nf, text: "Group 2 name: 'Group of  Foo1' created." };
UPDATE audit:8jvvkpwgkwa1hscsu1mw CONTENT { date: s'2024-08-26T03:32:08.709969189Z', id: audit:8jvvkpwgkwa1hscsu1mw, text: "Group 5 name: 'new group in new style' created." };
UPDATE audit:ckvz5f52j6wc58r5jw13 CONTENT { date: s'2024-08-25T07:30:55.264680894Z', id: audit:ckvz5f52j6wc58r5jw13, text: 'User 2 joined group 4' };
UPDATE audit:cusp0et7pqo24wyd1r5w CONTENT { date: s'2024-08-21T13:21:06.437850328Z', id: audit:cusp0et7pqo24wyd1r5w, text: 'User 2 joined group 2' };
UPDATE audit:cy7vg9nxw2u2hc37kb8l CONTENT { date: s'2024-08-25T06:56:16.330600215Z', id: audit:cy7vg9nxw2u2hc37kb8l, text: "Group 3 name: 'Send email to owner' created." };
UPDATE audit:dfjnhqm249mmfy4mmybv CONTENT { date: s'2024-09-10T06:23:30.539348234Z', id: audit:dfjnhqm249mmfy4mmybv, text: 'User 3 RSVPed to event 7' };
UPDATE audit:e16jjyj4xmyeum8je4dn CONTENT { date: s'2024-08-26T14:52:13.783053523Z', id: audit:e16jjyj4xmyeum8je4dn, text: 'User 8 joined group 4' };
UPDATE audit:fyp36c69h4wf81zcjh4g CONTENT { date: s'2024-08-26T07:39:10.538317330Z', id: audit:fyp36c69h4wf81zcjh4g, text: 'User 2 joined group 2' };
UPDATE audit:ke9qhh6ti3rfuj3iz0f0 CONTENT { date: s'2024-08-22T01:47:38.448694367Z', id: audit:ke9qhh6ti3rfuj3iz0f0, text: 'User 2 left group 2' };
UPDATE audit:l0vxe6585ot36c78zidu CONTENT { date: s'2024-08-22T01:31:10.813589883Z', id: audit:l0vxe6585ot36c78zidu, text: 'User 2 left group 2' };
UPDATE audit:l3j0027t2r2bsz1j8rxb CONTENT { date: s'2024-09-01T14:10:45.763648133Z', id: audit:l3j0027t2r2bsz1j8rxb, text: 'User 1 joined group 5' };
UPDATE audit:li7wm51r1w6x3736orf7 CONTENT { date: s'2024-08-24T11:13:59.197404463Z', id: audit:li7wm51r1w6x3736orf7, text: 'User 7 joined group 1' };
UPDATE audit:m9h1is63r74bmyfnuurw CONTENT { date: s'2024-08-22T01:33:53.853516206Z', id: audit:m9h1is63r74bmyfnuurw, text: 'User 2 joined group 2' };
UPDATE audit:nw0gk4hszxw2vyt0anj6 CONTENT { date: s'2024-09-02T06:19:57.529855931Z', id: audit:nw0gk4hszxw2vyt0anj6, text: 'User 1 RSVPed again to event 7' };
UPDATE audit:pe48xo55wz0q0f6kzaw3 CONTENT { date: s'2024-09-01T14:10:47.523856059Z', id: audit:pe48xo55wz0q0f6kzaw3, text: 'User 1 RSVPed to event 7' };
UPDATE audit:pzi5wmg2sxcbyz682q0v CONTENT { date: s'2024-09-02T06:34:12.108414620Z', id: audit:pzi5wmg2sxcbyz682q0v, text: 'User 1 RSVPed again to event 7' };
UPDATE audit:qlc1sp5kdyqkfft1bi5j CONTENT { date: s'2024-09-02T06:36:17.778667404Z', id: audit:qlc1sp5kdyqkfft1bi5j, text: 'User 1 left the event 7' };
UPDATE audit:s8izvt7wdp00k8cor4cm CONTENT { date: s'2024-09-10T06:23:29.316060670Z', id: audit:s8izvt7wdp00k8cor4cm, text: 'User 3 joined group 5' };
UPDATE audit:uk361xhxgapjbrry1ze6 CONTENT { date: s'2024-08-22T01:37:02.059254404Z', id: audit:uk361xhxgapjbrry1ze6, text: 'User 2 left group 2' };

-- ------------------------------
-- TABLE DATA: counter
-- ------------------------------

UPDATE counter:0uv354hvuoj1t99y0zh4 CONTENT { count: 5, id: counter:0uv354hvuoj1t99y0zh4, name: 'group' };
UPDATE counter:fk47vx25h5zq3lkqkjzw CONTENT { count: 7, id: counter:fk47vx25h5zq3lkqkjzw, name: 'event' };
UPDATE counter:op60nh91as1z32lqe9xq CONTENT { count: 11, id: counter:op60nh91as1z32lqe9xq, name: 'user' };

-- ------------------------------
-- TABLE DATA: event
-- ------------------------------

UPDATE event:axuqfvhjpl5ecks946c9 CONTENT { date: s'2024-08-27T15:00:00Z', description: '* one
* two', eid: 6, group_id: 4, id: event:axuqfvhjpl5ecks946c9, location: 'We now have a place', status: 'Published', title: 'First event new3 name' };
UPDATE event:q6jb2ywe5yepskwdmjde CONTENT { date: s'2024-09-22T14:00:00Z', description: '', eid: 7, group_id: 5, id: event:q6jb2ywe5yepskwdmjde, location: 'Somewhere', status: 'Published', title: 'Intro to Meet-OS 🎉  ' };

-- ------------------------------
-- TABLE DATA: group
-- ------------------------------

UPDATE group:dlsqb4e9xj62q49ib13r CONTENT { creation_date: s'2024-08-21T13:00:44.759214952Z', description: '', gid: 2, id: group:dlsqb4e9xj62q49ib13r, location: '', name: 'Group of  Foo1', owner: 3 };
UPDATE group:igaaj48yqrzu6qyz91k2 CONTENT { creation_date: s'2024-08-26T03:32:06.143933308Z', description: '', gid: 5, id: group:igaaj48yqrzu6qyz91k2, location: '', name: 'new group in new style', owner: 2 };
UPDATE group:kpad1694sumzkz0jasuf CONTENT { creation_date: s'2024-08-21T12:45:28.870031393Z', description: 'text and link to [Code Maven](https://code-maven.com/)', gid: 1, id: group:kpad1694sumzkz0jasuf, location: 'Virtual', name: 'Gabor Maven', owner: 2 };
UPDATE group:np1mad90scvh6keadnyf CONTENT { creation_date: s'2024-08-25T07:30:05.080943931Z', description: '', gid: 4, id: group:np1mad90scvh6keadnyf, location: '', name: 'Group of the Admin', owner: 1 };
UPDATE group:yym5godjy34vjufh666n CONTENT { creation_date: s'2024-08-25T06:56:14.670777330Z', description: '', gid: 3, id: group:yym5godjy34vjufh666n, location: '', name: 'Send email to owner', owner: 2 };

-- ------------------------------
-- TABLE DATA: membership
-- ------------------------------

UPDATE membership:7of96mtn4um9t7z15gvs CONTENT { admin: false, gid: 4, id: membership:7of96mtn4um9t7z15gvs, join_date: s'2024-08-25T07:30:55.263644791Z', uid: 2 };
UPDATE membership:ghzca2lcyi1254t4xmv0 CONTENT { admin: false, gid: 5, id: membership:ghzca2lcyi1254t4xmv0, join_date: s'2024-09-01T14:10:45.762522612Z', uid: 1 };
UPDATE membership:h9s3g45t24kup5yfhs1m CONTENT { admin: false, gid: 1, id: membership:h9s3g45t24kup5yfhs1m, join_date: s'2024-08-24T11:13:59.196461916Z', uid: 7 };
UPDATE membership:i3aoew67h7e0wiaeaaom CONTENT { admin: false, gid: 1, id: membership:i3aoew67h7e0wiaeaaom, join_date: s'2024-09-10T06:23:45.883540005Z', uid: 3 };
UPDATE membership:iurl5ktgnmmx1dg7zs6v CONTENT { admin: false, gid: 5, id: membership:iurl5ktgnmmx1dg7zs6v, join_date: s'2024-09-10T06:23:29.315256251Z', uid: 3 };
UPDATE membership:nmejrdi778w50g1hvbb6 CONTENT { admin: false, gid: 4, id: membership:nmejrdi778w50g1hvbb6, join_date: s'2024-08-26T14:52:13.781692170Z', uid: 8 };
UPDATE membership:rgryq19fdvwaeqm5zpt8 CONTENT { admin: false, gid: 2, id: membership:rgryq19fdvwaeqm5zpt8, join_date: s'2024-08-26T07:39:10.535754434Z', uid: 2 };

-- ------------------------------
-- TABLE DATA: rsvp
-- ------------------------------

UPDATE rsvp:3r8cds2jn0ap7375yfqf CONTENT { date: s'2024-09-02T06:36:17.775877832Z', eid: 7, id: rsvp:3r8cds2jn0ap7375yfqf, status: false, uid: 1 };
UPDATE rsvp:sd0hbfpzewa98jw7diu7 CONTENT { date: s'2024-09-10T06:23:30.537576511Z', eid: 7, id: rsvp:sd0hbfpzewa98jw7diu7, status: true, uid: 3 };

-- ------------------------------
-- TABLE DATA: user
-- ------------------------------

UPDATE user:21hrau9uocakrfz8ifqc CONTENT { about: NONE, code: '',                                      email: 'spam@szabgab.com',  github: 'szabgab', gitlab: '',        id: user:21hrau9uocakrfz8ifqc, linkedin: 'https://www.linkedin.com/in/szabgab/', name: '<b>spam</b>', password: '$pbkdf2-sha256$i=600000,l=32$mtkaIlI0Jh57a95kCNXLnw$tw4JYgnQ51AQAKeiTSUEQMkFksNJzFjsThOfkRSDaPM', process: 'register', registration_date: s'2024-08-26T09:23:45.298673439Z', uid: 8, verification_date: s'2024-08-26T09:24:10.056715572Z', verified: true };
UPDATE user:5pgi7l2fj2o0l4njhla4 CONTENT { about: NONE, code: '',                                      email: 'foo8@szabgab.com',  github: NONE,      gitlab: NONE,      id: user:5pgi7l2fj2o0l4njhla4, linkedin: NONE,                                   name: 'Foo8',        password: '$pbkdf2-sha256$i=600000,l=32$+5A5JtQkvhA2oD4k3O/HRw$V3fS6L1QvN4kimRrv8i8wHbDjQ1OPvShn/vBl7+adPs', process: 'register', registration_date: s'2024-08-30T03:41:30.308742373Z', uid: 11, verification_date: s'2024-08-30T03:41:53.262859838Z', verified: true };
UPDATE user:6vdax2oj9etg8hotugzs CONTENT { about: NONE, code: '',                                      email: 'foo3@szabgab.com',  github: NONE,      gitlab: NONE,      id: user:6vdax2oj9etg8hotugzs, linkedin: NONE,                                   name: 'Foo3',        password: '$pbkdf2-sha256$i=600000,l=32$/hqBLeUPawPSD5hRAzz+/g$vUd26eEinUhJ3WbaaDZUTXG/woFfzfBhkNjU8kmNXZQ', process: 'register', registration_date: s'2024-08-22T11:49:12.100945163Z', uid: 5, verification_date: s'2024-08-22T11:49:39.080030110Z', verified: true };
UPDATE user:8l7upgfgm5n7ut1wddxj CONTENT { about: NONE, code: '',                                      email: 'foo1@szabgab.com',  github: NONE,      gitlab: NONE,      id: user:8l7upgfgm5n7ut1wddxj, linkedin: NONE,                                   name: 'Foo1',        password: '$pbkdf2-sha256$i=600000,l=32$wyvu51Eck3qx03EzoRD+WA$ICT9l40fk9iyqbNGv4uMTF1v5PTViVGgh1pv0AURvJc', process: 'reset',    registration_date: s'2024-08-21T12:41:32.873434967Z', uid: 3, verification_date: s'2024-08-27T10:23:54.616399071Z', verified: true };
UPDATE user:d41h010rkuee9092b0ie CONTENT { about: NONE, code: '',                                      email: 'foo5@szabgab.com',  github: NONE,      gitlab: NONE,      id: user:d41h010rkuee9092b0ie, linkedin: NONE,                                   name: 'Foo5',        password: '$pbkdf2-sha256$i=600000,l=32$IBBgFg9XcGzr6qz4CfGMNA$6jwEZVKw4exzuqvgKiS6K9UoiGIYzTwGaujqSf6GtCQ', process: 'register', registration_date: s'2024-08-24T10:57:27.944793751Z', uid: 7, verification_date: s'2024-08-24T10:58:18.226169921Z', verified: true };
UPDATE user:f4r5sxua27ulxwm9dmrl CONTENT { about: NONE, code: '',                                      email: 'foo6@szabgab.com',  github: NONE,      gitlab: NONE,      id: user:f4r5sxua27ulxwm9dmrl, linkedin: NONE,                                   name: 'Foo6',        password: '$pbkdf2-sha256$i=600000,l=32$l6JgqBIuReGlJbaEiD/KjA$h+wZehapesMeDryzH2yYqrA/jECeL6xeFrPBk/IW/H4', process: 'resetxxx', registration_date: s'2024-08-30T03:03:59.092417888Z', uid: 9, verification_date: s'2024-09-15T09:36:52.741417772Z', verified: true };
UPDATE user:k112ypl9ijebmfki4tof CONTENT { about: NONE, code: '',                                      email: 'foo4@szabgab.com',  github: NONE,      gitlab: NONE,      id: user:k112ypl9ijebmfki4tof, linkedin: NONE,                                   name: 'Foo4',        password: '$pbkdf2-sha256$i=600000,l=32$xTZHehR3/XdmIIUxd2vkGg$+VjcAz18WWU7ZkGE+iZ+OEcjLkW0zcS1X86DpdSXWB8', process: 'register', registration_date: s'2024-08-23T10:54:16.549169155Z', uid: 6, verification_date: s'2024-08-23T11:06:00.419857424Z', verified: true };
UPDATE user:ltq9gex0935q3d07lav1 CONTENT { about: NONE, code: '',                                      email: 'foo2@szabgab.com',  github: NONE,      gitlab: NONE,      id: user:ltq9gex0935q3d07lav1, linkedin: NONE,                                   name: 'Foo2',        password: '$pbkdf2-sha256$i=600000,l=32$n8st6sZRKmjJX/5O4hQWvw$RI9mS9YSgOcdVnnEnMIez9z96eR0MPnXon4yMDgTZEY', process: 'register', registration_date: s'2024-08-21T12:42:15.853885992Z', uid: 4, verification_date: s'2024-08-21T12:42:35.093192450Z', verified: true };
UPDATE user:oc4um42whose11mtz28zf CONTENT { about: NONE, code: s'6aabfb2b-220d-418a-8365-5721c3177f3c', email: 'gabor@szabgab.com', github: 'szabgab', gitlab: 'szabgab', id: user:oc4um42whose11mtz28zf, linkedin: 'https://www.linkedin.com/in/szabgab/', name: 'Gabor Szabo', password: '$pbkdf2-sha256$i=600000,l=32$aNNXUmLcW6OPK1enzEjlGA$bXCUMcYF968KDqKBVNqQbuZMb8Ol2ZIZkAJNHWLegN4', process: 'reset',    registration_date: s'2024-08-21T07:33:12.087101391Z', uid: 2, verification_date: s'2024-08-21T07:35:38.698535841Z', verified: true };
UPDATE user:ut72kjayichbhy5qg2cm CONTENT { about: NONE, code: '',                                      email: 'admin@szabgab.com', github: NONE,      gitlab: NONE,      id: user:ut72kjayichbhy5qg2cm, linkedin: NONE,                                   name: 'Admin',       password: '$pbkdf2-sha256$i=600000,l=32$SdhWw/qA+vDn9pTUkFh7cQ$ussKZtuNJshjvY+lzGLMEA4lrgsLsMzqPJ8RVFJoBZ4', process: 'register', registration_date: s'2024-08-21T07:32:09.440556798Z', uid: 1, verification_date: s'2024-08-21T07:32:40.075781397Z', verified: true };
UPDATE user:v0jb4lyx5py21ad24eb9 CONTENT { about: NONE, code: '',                                      email: 'foo7@szabgab.com',  github: NONE,      gitlab: NONE,      id: user:v0jb4lyx5py21ad24eb9, linkedin: NONE,                                   name: 'Foo7',        password: '$pbkdf2-sha256$i=600000,l=32$mTWDif9p1S5N6gRYx/e5rQ$XlyjY3rd6qiARSM0wUVmQ909vI9RWBVbgL223X/PtAs', process: 'register', registration_date: s'2024-08-30T03:05:39.201069372Z', uid: 10, verification_date: s'2024-08-30T03:09:13.298518004Z', verified: true };

-- ------------------------------
-- TRANSACTION
-- ------------------------------

COMMIT TRANSACTION;

