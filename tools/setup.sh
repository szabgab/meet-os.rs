mkdir -p backup
database=$(cat Rocket.toml | grep "database_name " | cut -d'=' -f2 | sed  's/ //g' | sed  's/"//g')
namespace=$(cat Rocket.toml | grep "database_namespace" | cut -d'=' -f2 | sed  's/ //g' | sed  's/"//g')
username=$(cat Rocket.toml | grep "database_username" | cut -d'=' -f2 | sed  's/ //g' | sed  's/"//g')
password=$(cat Rocket.toml | grep "database_password" | cut -d'=' -f2 | sed  's/ //g' | sed  's/"//g')
echo $namespace
echo $database
#echo $username
#echo $password
docker volume create my-surreal-db
docker run --detach --restart always --name surrealdb -p 127.0.0.1:8000:8000 --user root -v$(pwd):/external -v my-surreal-db:/database surrealdb/surrealdb:v2.0.4 start --user $username --pass $password --log trace file://database

