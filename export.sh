mkdir -p backup
database=$(cat Rocket.toml | grep "database_name " | cut -d'=' -f2 | sed  's/ //g' | sed  's/"//g')
namespace=$(cat Rocket.toml | grep "database_namespace" | cut -d'=' -f2 | sed  's/ //g' | sed  's/"//g')
filename=$(date +%F_%H%M%S)
echo $namespace
echo $database
echo $filename
docker exec -it surreal /surreal export -e http://localhost:8000 --ns $namespace --db $database > backup/backup-$filename.sql
