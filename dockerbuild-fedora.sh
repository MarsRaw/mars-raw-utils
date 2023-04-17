#!/bin/bash

# Dockerized build method for Fedora rpms
MRU_VERSION=`cargo pkgid | cut -d @ -f 2`
. docker/config.sh

# Build RPMS for Fedora
DOCKER_IMAGE_NAME=${VENDOR}/build_rpms_${PROJECT}

docker build -t ${DOCKER_IMAGE_NAME} --build-arg MRU_VERSION=$MRU_VERSION -f docker/Dockerfile.fedora . 

CONTAINER_ID=$(docker run -d ${DOCKER_IMAGE_NAME})

if [ ! -d target/release ]; then
    mkdir -p target/release
fi

docker cp ${CONTAINER_ID}:/build/target/generate-rpm/ target/release/

docker rm -f ${CONTAINER_ID}  || true
docker rmi -f ${DOCKER_IMAGE_NAME} || true

