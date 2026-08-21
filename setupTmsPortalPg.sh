#!/bin/bash
#set -xv

SCRIPT_DIR=$(dirname $0)

PG_PORT="5432"
PG_CONTAINER="tms_portal_postgres"
PG_ROLE="tms"
PG_DATABASE="tms_db"
PG_SCHEMA="tms"
PG_ROLE_PASSWORD="tms_password"
PG_ADMIN_USER="postgres"
PG_ADMIN_PASS="pg_admin_pass"

function usage() {
  echo "$0 [-p port] [-r role] [-w password] [-d db] [-s schema]"

  echo "OPTIONS:"
  echo "     -p --port"
  echo "        The port to run postgres on"
  echo 
  echo "     -r --pgrole"
  echo "        The postgres role for the service"
  echo 
  echo "     -w --pgpass"
  echo "        The postgres password for the service"
  echo 
  echo "     -d --db"
  echo "        The postgres database name for the service"
  echo 
  echo "     -s --pgschema"
  echo "        The postgres schema name for the service"
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
    -r|--pgrole)
      PG_ROLE="$2"
      shift # past argument
      shift # past value
      ;;
    -w|--pgpass)
      PG_ROLE_PASSWORD="$2"
      shift # past argument
      shift # past value
      ;;
    -n|--db)
      PG_DATABASE="$2"
      shift # past argument
      shift # past value
      ;;
    -s|--pgschema)
      PG_SCHEMA="$2"
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

announce "create tms_portal database"
#set -xv
docker exec -i ${PG_CONTAINER} psql -U ${PG_ADMIN_USER} <<EOD
SELECT 'CREATE DATABASE ${PG_DATABASE} ENCODING="UTF8" LC_COLLATE="en_US.utf8" LC_CTYPE="en_US.utf8" ' WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = '${PG_DATABASE}')\gexec
EOD

announce "create role ${PG_ROLE}"
docker exec -i ${PG_CONTAINER} psql -U ${PG_ADMIN_USER} <<EOD
DO \$\$
BEGIN
  CREATE ROLE ${PG_ROLE} LOGIN password '${PG_ROLE_PASSWORD}';
  EXCEPTION WHEN DUPLICATE_OBJECT THEN
  RAISE NOTICE 'Role already exists. Role name: "${PG_ROLE}"';
END
\$\$
EOD

docker exec -i ${PG_CONTAINER} psql -U ${PG_ADMIN_USER} <<EOD
alter database ${PG_DATABASE} OWNER TO ${PG_ROLE} ;
EOD

docker exec -i ${PG_CONTAINER} psql -U ${PG_ROLE} ${PG_DATABASE} <<EOD
  CREATE SCHEMA IF NOT EXISTS ${PG_SCHEMA} AUTHORIZATION ${PG_ROLE};
  ALTER ROLE ${PG_ROLE} SET search_path = '${PG_SCHEMA}';
  SET search_path TO ${PG_SCHEMA};
EOD

