#!/bin/bash

# Dockerized build method for Fedora rpms

: ${PROJECT:=mars_raw_utils}
: ${VENDOR:=kevinmgill}


# Build RPMS for Fedora
DOCKER_IMAGE_NAME=${VENDOR}/build_rpms_${PROJECT}

docker build -t ${DOCKER_IMAGE_NAME} -f Dockerfile.fedora . 

CONTAINER_ID=$(docker run -d ${DOCKER_IMAGE_NAME})

if [ ! -d target/release ]; then
    mkdir -p target/release
fi

docker cp ${CONTAINER_ID}:/build/target/release/rpmbuild target/release/

docker rm -f ${CONTAINER_ID}  || true
docker rmi -f ${DOCKER_IMAGE_NAME} || true

