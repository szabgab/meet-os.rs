set -e
mkdir -p backup

if [ ! -f Rocket.toml ]; then
    echo "Rocket.toml not found!"
    exit 1
fi

database=$(cat Rocket.toml | grep "database_name " | cut -d'=' -f2 | sed  's/ //g' | sed  's/"//g')
namespace=$(cat Rocket.toml | grep "database_namespace" | cut -d'=' -f2 | sed  's/ //g' | sed  's/"//g')
username=$(cat Rocket.toml | grep "database_username" | cut -d'=' -f2 | sed  's/ //g' | sed  's/"//g')
password=$(cat Rocket.toml | grep "database_password" | cut -d'=' -f2 | sed  's/ //g' | sed  's/"//g')

if [ "$database" == "" ]; then
    echo "database_name is missing from Rocket.toml"
    exit 1
fi

if [ "$namespace" == "" ]; then
    echo "database_namespace is missing from Rocket.toml"
    exit 1
fi

if [ "$username" == "" ]; then
    echo "database_username is missing from Rocket.toml"
    exit 1
fi

if [ "$password" == "" ]; then
    echo "database_password is missing from Rocket.toml"
    exit 1
fi


echo $namespace
echo $database
#echo $username
#echo $password

set -x
docker volume create my-surreal-db
docker run --detach --restart always --name surrealdb -p 127.0.0.1:8000:8000 --user root -v"$(pwd)":/external -v my-surreal-db:/database surrealdb/surrealdb:v2.0.4 start --user $username --pass $password --log trace file://database

