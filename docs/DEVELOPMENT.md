# Development

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
