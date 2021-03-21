#! /bin/bash

PROTO_DIR="${PWD}/proto"

OUT_PYTHON_DIR="${PWD}/models"

mkdir -p build
cd build

python3 -m venv gen
source "gen/bin/activate"

if [ ! -d python-betterproto ]
then
  git clone https://github.com/danielgtaylor/python-betterproto python-betterproto
  cd python-betterproto
else
  cd python-betterproto
  git pull
fi


python -m pip install poetry
poetry install
poetry shell

python -m grpc_tools.protoc \
    --proto_path="${PROTO_DIR}" \
    --python_betterproto_out="${OUT_PYTHON_DIR}" \
    ${PROTO_DIR}/*.proto
