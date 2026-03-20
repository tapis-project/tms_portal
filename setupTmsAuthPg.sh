#!/bin/bash
#set -xv

SCRIPT_DIR=$(dirname $0)

PG_PORT="5432"
PG_CONTAINER="tms_auth_postgres"
PG_USER="tms_auth_user"
PG_DATABASE="tms_auth_db"
PG_USER_PASSWORD="tms_auth_password"
PG_ADMIN_USER="postgres"
PG_ADMIN_PASS="pg_admin_pass"

function usage() {
  echo "$0 [-p port] [-u user] [-w password] [-d db]"

  echo "OPTIONS:"
  echo "     -p --port"
  echo "        The port to run postgres on"
  echo 
  echo "     -u --pguser"
  echo "        The postgres user for the service"
  echo 
  echo "     -w --pgpass"
  echo "        The postgres password for the service"
  echo 
  echo "     -d --db"
  echo "        The postgres database name for the service"
  echo 
  exit 1
}

function announce() {
  echo ---==== $@ ====---
}

while [[ $# -gt 0 ]]; do
  case $1 in
    -p|--port)
      PG_PORT="$2"
      shift # past argument
      shift # past value
      ;;
    -u|--pguser)
      PG_USER="$2"
      shift # past argument
      shift # past value
      ;;
    -w|--pgpass)
      PG_USER_PASSWORD="$2"
      shift # past argument
      shift # past value
      ;;
    -n|--db)
      PG_DATABASE="$2"
      shift # past argument
      shift # past value
      ;;
    -*|--*)
      echo "Unknown option $1"
      usage
      ;;
    *)
      echo "Unknown positional arguement $1"
      usage
  esac
done

announce "database container on port ${PG_PORT}"
export PG_PORT
export PG_ADMIN_PASS

announce "running docker compose up"
docker compose -f ${SCRIPT_DIR}/docker-compose.yml up --wait

announce "pausing for startup"
sleep 5

announce "create tms_auth database"
set -xv
docker exec -i ${PG_CONTAINER} psql -U ${PG_ADMIN_USER} <<EOD
SELECT 'CREATE DATABASE ${PG_DATABASE} ENCODING="UTF8" LC_COLLATE="en_US.utf8" LC_CTYPE="en_US.utf8" ' WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = '${PG_DATABASE}')\gexec
EOD

announce "create user ${PG_USER}"
docker exec -i ${PG_CONTAINER} psql -U ${PG_ADMIN_USER} <<EOD
DO \$\$
BEGIN
  CREATE USER ${PG_USER} with encrypted password '${PG_USER_PASSWORD}';
  EXCEPTION WHEN DUPLICATE_OBJECT THEN
  RAISE NOTICE 'User already exists. User name: "${PG_USER}"';
END
\$\$
EOD

docker exec -i ${PG_CONTAINER} psql -U ${PG_ADMIN_USER} <<EOD
alter database ${PG_DATABASE} OWNER TO ${PG_USER} ;
EOD

