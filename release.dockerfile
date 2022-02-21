# syntax=docker/dockerfile:experimental

############################################################
# NOTE: executables are in /target/release, instead of /target/debug
# For more info on multi-stage builds see:
# https://blog.jawg.io/docker-multi-stage-build/
############################################################

FROM rust:1.42.0-buster as builder

# Set working directory in Docker Image
WORKDIR /
# Copy cargo: contains list of dependencies
# Doing this first downloads and caches the libraries
COPY ./Cargo.toml    /Cargo.toml
COPY ./diesel.toml   /diesel.toml
# Copy over source files incrementally, to take advantage of cacheing
COPY ./src/db        /src/db
COPY ./src/utils     /src/utils
COPY ./src/models    /src/models
COPY ./src/lib.rs    /src/lib.rs
# Build lib: This also gets the dependencies cached
RUN cargo update
RUN --mount=type=cache,target=/usr/local/cargo,from=rust,source=/usr/local/cargo \
    --mount=type=cache,target=target \
    cargo build --lib

######### Build #########
# Copy over app files for each binary
COPY ./src/bin/payment   /src/bin/payment
RUN cargo build --bin payment
# Build app (executables will be in /target/debug/)

### You can strip the binary to make it much smaller,
### but it also removes debug information.
RUN strip /target/debug/payment

# print and inspect compiled binaries
RUN  ls /target/debug/
RUN  ls /

#######################################
####### END OF RUST BUILD STAGE #######
#######################################

### NOTE: the images must match the image used in the build stage
#### Debian
# Use stretch-slim debian, match image used in build stage.
FROM debian:buster-slim
RUN apt-get update && apt-get install -y libpq-dev curl

# Copy binaries from builder to this new image
COPY --from=builder /target/debug/payment   /bin/gm_payment

####### MUST INCLUDE FOR TLS requests to verify
COPY --from=builder /etc  /etc

# Indicate that this image expects to accept traffic internally on this port.
# NOTE: Expose doesn't do anything, it's just documenting that this port is hardcoded internally
# and you'll want to map a host port to this value.
EXPOSE 8898

# Define the health check
HEALTHCHECK --start-period=30s CMD curl --fail http://localhost:8898/_health || exit 1
