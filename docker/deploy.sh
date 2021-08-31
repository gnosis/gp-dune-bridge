#!/bin/bash

set -euo pipefail

echo "$DOCKER_PASSWORD" | docker login -u "$DOCKER_NAME" --password-stdin
# Build and tag api-image
docker build -t "${DOCKERHUB_PROJECT}-api" -f docker/Dockerfile.binary .
docker tag "${DOCKERHUB_PROJECT}-api" gnosispm/"${DOCKERHUB_PROJECT}-api":$1
docker push gnosispm/"${DOCKERHUB_PROJECT}-api":$1

# Build and tag dune-data-fetcher-image
cd dune_data_api
docker build -t "${DOCKERHUB_PROJECT}-fetcher" -f docker/Dockerfile.binary .
docker tag "${DOCKERHUB_PROJECT}-fetcher" gnosispm/"${DOCKERHUB_PROJECT}-fetcher":$1
docker push gnosispm/"${DOCKERHUB_PROJECT}-fetcher":$1