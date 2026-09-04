#!/bin/bash
#set -xv

SCRIPT_DIR=$(dirname $0)
GENERATE_TOKEN_KEYS="true"
GENERATE_STATE_KEYS="true"

VARS_TO_REPLACE=(
  '$BASE_URL'
  '$GLOBUS_CLIENT_ID'
  '$GLOBUS_CLIENT_SECRET'
  '$TACC_RP_CLIENT_ID'
  '$TACC_RP_CLIENT_SECRET'
  '$TMS_TOKEN_KID'
  '$TMS_TOKEN_PUB_KEY'
  '$TMS_TOKEN_PRIV_KEY'
  '$TMS_STATE_KID'
  '$TMS_STATE_PUB_KEY'
  '$TMS_STATE_PRIV_KEY'
  '$TAPIS_TEST_TENANT_CLIENT_ID',
  '$TAPIS_TEST_TENANT_CLIENT_SECRET'
  '$TAPIS_AUTH_CLIENT_ID'
  '$CONFIG_DIR'
)

OUTPUT_FILE_NAME="output.sql"
BASE_URL="http://localhost:8080"
CONFIG_DIR="application/resources/config"

function usage() {
  echo "$0 [-p port] [-u user] [-w password] [-d db]"

  echo "OPTIONS:"
  echo "     -o --out"
  echo "        Output file name"
  echo 
  echo "     -v --vars"
  echo "        Filename of file to source.  File should contain variable exports"
  echo 
  echo "     -u --base-url"
  echo "        base url - e.g. http://localhost:8080 ... NOTE: no trailing slash"
  echo 
  echo "     --token-priv-key-file"
  echo "        private key file for token signing"
  echo 
  echo "     --token-kid"
  echo "        key id for token signing"
  echo 
  echo "     --state-priv-key-file"
  echo "        private key file for state signing"
  echo 
  echo "     --state-kid"
  echo "        key id for token signing"
  echo 
  exit 1
}

function announce() {
  echo ---==== $@ ====---
}

while [[ $# -gt 0 ]]; do
  case $1 in
    -o|--out)
      OUTPUT_FILE_NAME="$2"
      shift # past argument
      shift # past value
      ;;
    -v|--vars)
      VARS_FILE="$2"
      shift # past argument
      shift # past value
      ;;
    -u|--base-url)
      BASE_URL="$2"
      shift # past argument
      shift # past value
      ;;
    --token-priv-key-file)
      TOKEN_PRIV_KEY_FILE="$2"
      GENERATE_TOKEN_KEYS="false"
      echo Using token private key file : ${TOKEN_PRIV_KEY_FILE}
      shift # past argument
      shift # past value
      ;;
    --state-priv-key-file)
      STATE_PRIV_KEY_FILE="$2"
      GENERATE_STATE_KEYS="false"
      echo Using state private key file : ${STATE_PRIV_KEY_FILE}
      shift # past argument
      shift # past value
      ;;
    --tms-token-kid)
      TMS_TOKEN_KID_TMP="$2"
      echo Using token kid: ${TMS_TOKEN_KID_TMP}
      shift # past argument
      shift # past value
      ;;
    --tms-state-kid)
      TMS_STATE_KID_TMP="$2"
      echo Using state kid: ${TMS_STATE_KID_TMP}
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

# source additional file with variables
if [[ -n "${VARS_FILE}" ]] ; then
  if [[ -f "${VARS_FILE}" ]] ; then
      echo "sourcing file: ${VARS_FILE}"
      source ${VARS_FILE}
  else 
      echo Error: vars file not found: ${VARS_FILE}
      exit 1
  fi
fi

# after sourcing files apply parameter variables
if [[ -n "${TMS_TOKEN_KID_TMP}" ]] ; then
  TMS_TOKEN_KID=${TMS_TOKEN_KID_TMP}
  echo Using token kid: ${TMS_TOKEN_KID}
fi
if [[ -n "${TMS_STATE_KID_TMP}" ]] ; then
  TMS_STATE_KID=${TMS_STATE_KID_TMP}
  echo Using state kid: ${TMS_STATE_KID}
fi
set -xv
# generate token keys if needed
if [[ "${GENERATE_TOKEN_KEYS}" == "true" ]] ; then
  if [[ -z "${TMS_TOKEN_PRIV_KEY}" ]] ; then 
    echo Generating token private key
    TMS_TOKEN_PRIV_KEY=$(openssl genrsa 2048)
  fi
else
  if [[ ! -f "${TOKEN_PRIV_KEY_FILE}" ]] ; then 
    echo "Token private key not found:" ${TOKEN_PRIV_KEY_FILE}
    exit 1
  fi
  TMS_TOKEN_PRIV_KEY=$(cat ${TOKEN_PRIV_KEY_FILE})
fi

# create the public key from the private key
TMS_TOKEN_PUB_KEY=$(echo "${TMS_TOKEN_PRIV_KEY}" | openssl rsa -pubout)

# generate a kid if required for state 
if [[ -z "${TMS_TOKEN_KID}" ]] ; then 
  TMS_TOKEN_KID=$(uuidgen)
  echo Using token kid: ${TMS_TOKEN_KID}
fi

# generate state keys if needed
if [[ "${GENERATE_STATE_KEYS}" == "true" ]] ; then
  if [[ -z "${TMS_STATE_PRIV_KEY}" ]] ; then 
    echo Generating state private key
    TMS_STATE_PRIV_KEY=$(openssl genrsa 2048)
  fi
else
  if [[ ! -f "${STATE_PRIV_KEY_FILE}" ]] ; then 
    echo "State private key not found:" ${STATE_PRIV_KEY_FILE}
    exit 1
  fi
  TMS_STATE_PRIV_KEY=$(cat ${STATE_PRIV_KEY_FILE})
fi

# create the public key from the private key
TMS_STATE_PUB_KEY=$(echo "${TMS_STATE_PRIV_KEY}" | openssl rsa -pubout)

# generate a kid if required for state 
if [[ -z "${TMS_STATE_KID}" ]] ; then 
  TMS_STATE_KID=$(uuidgen)
  echo Using state kid: ${TMS_STATE_KID}
fi
echo baseurl = $BASE_URL

# do some exports
export BASE_URL
export GLOBUS_CLIENT_ID
export GLOBUS_CLIENT_SECRET
export TACC_RP_CLIENT_ID
export TACC_RP_CLIENT_SECRET
export TMS_TOKEN_KID
export TMS_TOKEN_PUB_KEY
export TMS_TOKEN_PRIV_KEY
export TMS_STATE_KID
export TMS_STATE_PUB_KEY
export TMS_STATE_PRIV_KEY
export TAPIS_TEST_TENANT_CLIENT_ID
export TAPIS_TEST_TENANT_CLIENT_SECRET
export CONFIG_DIR

#echo BASE_URL $BASE_URL
#echo GLOBUS_CLIENT_ID $GLOBUS_CLIENT_ID
#echo GLOBUS_CLIENT_SECRET $GLOBUS_CLIENT_SECRET
#echo TACC_RP_CLIENT_ID $TACC_RP_CLIENT_ID
#echo TACC_RP_CLIENT_SECRET $TACC_RP_CLIENT_SECRET
#echo TMS_TOKEN_KID $TMS_TOKEN_KID
#echo TMS_TOKEN_PUB_KEY $TMS_TOKEN_PUB_KEY
#echo TMS_TOKEN_PRIV_KEY $TMS_TOKEN_PRIV_KEY
#echo TMS_STATE_KID $TMS_STATE_KID
#echo TMS_STATE_PUB_KEY $TMS_STATE_PUB_KEY
#echo TMS_STATE_PRIV_KEY $TMS_STATE_PRIV_KEY
#echo TAPIS_TEST_TENANT_CLIENT_ID $TAPIS_TEST_TENANT_CLIENT_ID
#echo TAPIS_TEST_TENANT_CLIENT_SECRET $TAPIS_TEST_TENANT_CLIENT_SECRET
#echo TAPIS_AUTH_CLIENT_ID $TAPIS_AUTH_CLIENT_ID
#echo CONFIG_DIR $CONFIG_DIR

# do envsubst
export VARS_PARAM=$(IFS=,;echo "${VARS_TO_REPLACE[*]}")
envsubst "'"${VARS_PARAM}"'" < ${SCRIPT_DIR}/../tms_portal_dev_config.sql > ${OUTPUT_FILE_NAME}


