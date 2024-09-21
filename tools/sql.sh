
# Start an interactive surrealDB shell
database=$(cat Rocket.toml | grep "database_name " | cut -d'=' -f2 | sed  's/ //g' | sed  's/"//g')
namespace=$(cat Rocket.toml | grep "database_namespace" | cut -d'=' -f2 | sed  's/ //g' | sed  's/"//g')
username=$(cat Rocket.toml | grep "database_username" | cut -d'=' -f2 | sed  's/ //g' | sed  's/"//g')
password=$(cat Rocket.toml | grep "database_password" | cut -d'=' -f2 | sed  's/ //g' | sed  's/"//g')
filename="/external/sql.sql"
echo $namespace
echo $database
#echo $username
#echo $password
echo $filename
docker exec -it surrealdb /surreal sql -e http://localhost:8000 --ns $namespace --db $database --user $username --pass $password
