#!/bin/bash
#set -xv

SCRIPT_DIR=$(dirname $0)

DB_ROLE="tms"
DB_PASSWORD="tms_password"
DB_HOST="localhost"
DB_PORT="5432"
DB_NAME="tms_db"
TMS_SERVER_GIT_BRANCH="main"
USE_CACHE=0

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
  echo "     -b --branch"
  echo "        TMS Server github repository branch name"
  echo 
  echo "     --cached"
  echo "        Use cached files - this may fail or result in an incomplete db if the files are not present, or not up to date"
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
    -b|--branch)
      TMS_SERVER_GIT_BRANCH="$2"
      shift # past argument
      shift # past value
      ;;
    --cached)
      USE_CACHE=1
      shift # past argument
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

MIGRATION_DIR=${SCRIPT_DIR}/migrations
DB_URL=postgres://${DB_ROLE}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}

if [ ${USE_CACHE} -eq 0 ] ; then 
  #install sqlx command line util
  cargo install sqlx-cli

  # make directory for migration files, and change to that directory
  mkdir -p ${MIGRATION_DIR}
  pushd ${MIGRATION_DIR}

  # get the list of migration files (based on branch)
  MIGRATION_FILES=$(http https://api.github.com/repos/tms-trust-project/tms_server/contents/resources/migrations?ref=${TMS_SERVER_GIT_BRANCH} | jq .[].download_url -r)

  # download each migration file
  for MFILE in ${MIGRATION_FILES} ; do 
        LOCAL_NAME=$(basename ${MFILE})
        echo downloading ${MFILE} as ${LOCAL_NAME}
        # httpie download, quiet, overwrite by specifying name
        http -dqo ${LOCAL_NAME} ${MFILE}
  done
  popd
fi

#process the files in the migration directory
echo dburl = ${DB_URL}
sqlx migrate run --database-url $DB_URL --source ${MIGRATION_DIR}
