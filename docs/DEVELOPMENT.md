# Local Development

* Install [Docker](https://docs.docker.com/engine/install/)
* Start the Docker container that runs the SurrealDB

```
docker volume create my-surreal-db
docker run --detach --restart always --name surrealdb -p 127.0.0.1:8000:8000 --user root -v$(pwd):/external -v my-surreal-db:/database surrealdb/surrealdb:v2.0.1 start --user root --pass root --log trace file://database
```

* Note: At the end of the development session you might want to stop the docker container.

```
docker stop surrealdb
```

* For the next session you can start it again.

```
docker restart surrealdb
```

The database is store in the Docker volume. If you'd like to get rid of it you can do so
by shutting down the container and then running:

```
docker volume remove my-surreal-db
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

The system sends emails. You can either setup a [Sendgrid](https://sendgrid.com/) account (the free account might be enough during development) or you might use the "file-based delivery" which basically means that every email is saved as a file. The file-based system is used for testing.

* If using sendgrid then create an account on Sendgrid and add your API key to the `[debug]` section of `Rocket.toml`:

```
email            = "Sendgrid"
sendgrid_api_key = "SG.blabla"
```

* If using the file-based delivery system then create and empty folder (e.g. emails/ in the current folder), edit `Rocket.toml` and add the full path to this folder:

```
email            = "Folder"
email_folder     = "/path/to/email_folder"
```

The system identifies its administrator(s) by their email.
In `Rocket.toml` replace the email addresses in the `admins`
field to designate one or more of the user you will register later
to have all the rights of an admin.


TODO: This needs rework:

Create `config.yaml` adding the Google Analytics code:

```
google_analytics: G-SOME-CODE
```


Run

```
cargo watch -x run
```

This will download all the dependencies, compile the code and start the server.
This will also monitor your filesystem for changes and will recompile and restart
the server on every change.


## Testing

The tests assume a running SurrealDB server. We setup a web server for each test run in a forked process and then send http requests using the reqwest crate. We use environment variables to pass test-related configuration options to the application.

The tests used to run in a single thread configured in the `.cargo/config.toml` by setting `RUST_TEST_THREADS`to `1`. Not any more.

The reason we test with a web server is probably historical. The in-process tests collided and I thought the solution will be setting up external processes. Only after converting the tests and realizing those also fail I understood that the problem is that the tests run in parallel threads and environment variables are per-process. So the use of the environment variables is what force us to run the tests in a single thread.

This issue has been fixed for the forked processes, but not yet for the in-process tests. So for now we only have one in-process test (test_simple).


Running the tests:

```
cargo test
```


Running the test filtered by the name of the test functions:
(e.g. all the tests with the word `admin` in their name)

```
cargo test admin
```

## Backup and restore

* dump / export

```
docker exec -it surrealdb /surreal export -e http://localhost:8000 --ns meet-os-ns --db meet-os-db > out.txt
```

* restore / import

```
docker exec surrealdb /surreal import -e http://localhost:8000 --namespace meet-os-ns --database meet-os-db /external/out.txt
```

## CLI

```
docker exec -it surrealdb /surreal sql -e http://localhost:8000 --ns meet-os-ns --db meet-os-ns --pretty
```

## Measure elapsed time of tests

* If you don't have it yet, install the nightly toolchain of Rust:

```
rustup toolchain install nightly
```

Then run the test with these flags:

```
cargo +nightly test -- -Z unstable-options --report-time
```

## SurrealDB cleanup (regular and for upgrade)

* Due to [a bug](https://github.com/surrealdb/surrealdb/issues/3904) or  [two](https://github.com/surrealdb/surrealdb/issues/3903)
we don't remove the namespaces and databases that we create during testing. It seems that many such entries might cause connection
failures for SurrealDB so once in a while we might want to clean up the database.

* When changing schema (which we don't strictly defined yet) we will have some code that makes changes to the database.
In order to be able to practice this we might want to remove the whole database once in a while.

Here is the procedure for the local development setup.

* Export the data from the database (before trying to upgrade)
* Stop the web application
* Stop the docker container with SurrealDB
* Remove the container `docker container rm surreal`
* Remove the Docker volume
* Create the Docker volume
* Start a new container
* Import the data
* Start the web application

## Generate test coverage report

* We generate a test coverage report in the GitHub Actions CI you download from GitHub.
* You can also generate the report locally by using [tarpaulin](https://github.com/xd009642/tarpaulin)

Install tarpaulin:

```
cargo install cargo-tarpaulin
```

Tarpaulin does not seem to work well with the forking tests so until we figure out how to fix that we need to temporarily remove
those tests:

```
rm -f tests/*.rs
```

Run tarpaulin:

```
time cargo tarpaulin --ignore-tests -o Html -o Lcov --timeout 240 --engine llvm
```

Alternatively you can run it for a subset of the tests. e.g. for the database specific test:

```
time cargo tarpaulin --ignore-tests -o Html -o Lcov --timeout 240 --engine llvm -- test_db
```

These commands will generate 2 files: `lcov.info` is really only needed for the CI to upload to [Coveralls](https://coveralls.io/) and `tarpaulin-report.html` that you can open with your browser.
