#!/bin/bash

if [ -z "${CHAIN_ID}" ]; then
  echo "CHAIN_ID is not defined"
  exit 1
fi;

if [ -z "${PROGRAM_ID}" ]; then
  echo "PROGRAM_ID is not defined"
  exit 1
fi;

if [ -z "${SOLANA_RPC}" ]; then
  echo "SOLANA_RPC is not defined"
  exit 1
fi;

if [ -z "${COMMAND}" ]; then
  echo "COMMAND is not defined"
  exit 1
fi;

if [[ "$COMMAND" == "reg-rollup" ]]; then
  if [ -z "${ROLLUP_OWNER_ID}" ]; then
    echo "ROLLUP_OWNER_ID is not defined"
    exit 1
  fi;

  if [ -z "${UPGRADE_AUTHORITY}" ]; then
    echo "UPGRADE_AUTHORITY is not defined"
    exit 1
  fi;

  ./cli --program-id $PROGRAM_ID --chain-id $CHAIN_ID --url $SOLANA_RPC $COMMAND $ROLLUP_OWNER_ID $UPGRADE_AUTHORITY

elif [[ "$COMMAND" == "create-balance" ]]; then
  if [ -z "${ADDRESS}" ]; then
    echo "ADDRESS is not defined"
    exit 1
  fi;

  if [ -z "${BALANCE}" ]; then
    echo "BALANCE is not defined"
    exit 1
  fi;

  if [ -z "${KEYPAIR}" ]; then
    echo "KEYPAIR is not defined"
    exit 1
  fi;

  ./cli --program-id $PROGRAM_ID --chain-id $CHAIN_ID --url $SOLANA_RPC $COMMAND $ADDRESS $BALANCE $KEYPAIR

elif [[ "$COMMAND" == "reg-gas-recipient" ]]; then
  if [ -z "${ADDRESS}" ]; then
    echo "ADDRESS is not defined"
    exit 1
  fi;

  if [ -z "${KEYPAIR}" ]; then
    echo "KEYPAIR is not defined"
    exit 1
  fi;

  ./cli --program-id $PROGRAM_ID --chain-id $CHAIN_ID --url $SOLANA_RPC $COMMAND $ADDRESS $KEYPAIR
else
  echo "Unknown cli command $COMMAND"
  exit 1
fi;
