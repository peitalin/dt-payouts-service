#!/bin/sh
DOCKER_BUILDKIT=1 docker build -f ./local.dockerfile -t gm-payment-service .
