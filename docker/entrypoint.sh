#!/bin/bash

if [ -z ${SERVICE_NAME} ]; then
  echo "SERVICE_NAME is not specified"
  exit 1
fi;

./${SERVICE_NAME}