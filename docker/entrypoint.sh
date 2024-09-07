#!/bin/bash

if [ -z ${SERVICE_NAME} ]; then
  echo "SERVICE_NAME is not specified"
  exit 1
fi;

if [ ! -z ${TEST_MODE} ]; then
  if [ "${SERVICE_NAME}" = "proxy" ]; then
    /usr/bin/solana -u http://solana:8899 airdrop 100 /opt/proxy-sender.json
  elif [ "${SERVICE_NAME}" = "rhea" ]; then
    /usr/bin/solana -u http://solana:8899 airdrop 100 /opt/rhea-sender.json
  fi;
fi;

./${SERVICE_NAME}