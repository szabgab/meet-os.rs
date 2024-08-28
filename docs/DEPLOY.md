# Deploy

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
