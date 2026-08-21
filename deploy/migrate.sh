#!/bin/bash
#set -xv

SCRIPT_DIR=$(dirname $0)

DB_ROLE="tms"
DB_PASSWORD="tms_password"
DB_HOST="localhost"
DB_PORT="5432"
DB_NAME="tms_db"

function usage() {
  echo "$0 [-p port] [-u user] [-w password] [-d db]"

  echo "OPTIONS:"
  echo "     -p --port"
  echo "        Postgres port"
  echo 
  echo "     -r --role"
  echo "        Postgres role name"
  echo 
  echo "     -w --pass"
  echo "        Postgres password"
  echo 
  echo "     -d --db"
  echo "        Postgres database name"
  echo 
  echo "     -h --host"
  echo "        Postgres database host"
  echo 
  exit 1
}

function announce() {
  echo ---==== $@ ====---
}

while [[ $# -gt 0 ]]; do
  case $1 in
    -p|--port)
      DB_PORT="$2"
      shift # past argument
      shift # past value
      ;;
    -r|--role)
      DB_ROLE="$2"
      shift # past argument
      shift # past value
      ;;
    -w|--pass)
      DB_PASSWORD="$2"
      shift # past argument
      shift # past value
      ;;
    -d|--db)
      DB_NAME="$2"
      shift # past argument
      shift # past value
      ;;
    -h|--host)
      DB_HOST="$2"
      shift # past argument
      shift # past value
      ;;
    -*)
      echo "Unknown option $1"
      usage
      ;;
    *)
      echo "Unknown positional argument $1"
      usage
  esac
done


DB_URL=postgres://${DB_ROLE}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}
MIGRATION_DIR=${SCRIPT_DIR}/migrations
cargo install sqlx-cli
mkdir -p ${MIGRATION_DIR}
pushd ${MIGRATION_DIR}

MIGRATION_FILES=$(http https://api.github.com/repos/tms-trust-project/tms_server/contents/resources/migrations | jq .[].download_url -r)
for MFILE in ${MIGRATION_FILES} ; do 
        LOCAL_NAME=$(basename ${MFILE})
        echo downloading ${MFILE} as ${LOCAL_NAME}
        # httpie download, quiet, overwrite by specifying name
        http -dqo ${LOCAL_NAME} ${MFILE}
done
popd

echo dburl = ${DB_URL}
sqlx migrate run --database-url $DB_URL --source ${MIGRATION_DIR}
