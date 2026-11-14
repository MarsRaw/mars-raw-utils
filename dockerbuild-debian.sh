#!/bin/bash

MRU_VERSION=`cargo pkgid -p mru-cli -q | cut -d \# -f 2`
. docker/config.sh

# Build debs for Debian
DOCKER_IMAGE_NAME=${VENDOR}/build_debs_${PROJECT}

docker build -t ${DOCKER_IMAGE_NAME} --build-arg MRU_VERSION=$MRU_VERSION -f docker/Dockerfile.debian . 

CONTAINER_ID=$(docker run -d ${DOCKER_IMAGE_NAME})

if [ ! -d dist/release ]; then
    mkdir -p dist/release
fi

docker cp ${CONTAINER_ID}:/mars/target/debian/mru_${MRU_VERSION}-1_amd64.deb dist/release/

docker rm -f ${CONTAINER_ID}  || true
docker rmi -f ${DOCKER_IMAGE_NAME} || true

