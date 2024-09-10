FROM docker.io/library/rust:alpine AS builder

## Add os build dependencies
## podman run --rm --tty --interactive rust:alpine /bin/sh
## apk update; apk info <package>
RUN apk add --no-cache musl-dev=1.2.5-r0 sqlite-static=3.45.3-r1 sqlite-dev=3.45.3-r1

## Copy the source files for the project
WORKDIR /actix-data-receiver
COPY ./Cargo.toml ./Cargo.toml
COPY ./src ./src

## Build the release
RUN cargo build --release

## Use a small base image to run the compiled application binary
## TODO: distroless encountered permission issues when writing, see if there is a work around
FROM docker.io/library/alpine:3 AS final

## Copy the compiled application binary from the builder
COPY --from=builder /actix-data-receiver/target/release/actix-data-receiver \
    /usr/local/bin/actix-data-receiver

## Setup the entrypoint with the binary
ENTRYPOINT ["/usr/local/bin/actix-data-receiver"]
## Example usage:
## podman build --tag actix-data-receiver:${TAG:=v1} .
## 
## podman run --rm --name actix-data-receiver --detach \
## --volume ./db/:/var/db/:rw --publish 8888:8888/tcp \
## actix-data-receiver:v1 --database-files /var/db
## 
## See available appliaction options:
## podman run --rm --name actix-data-receiver-help actix-data-receiver:${TAG:=v1} --help