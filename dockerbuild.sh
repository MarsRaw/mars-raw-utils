
#!/bin/bash

MRU_VERSION=`cargo pkgid | cut -d @ -f 2`
. docker/config.sh


docker build -t ${VENDOR}/${PROJECT}:${MRU_VERSION} -t ${VENDOR}/${PROJECT}:latest -f docker/Dockerfile .