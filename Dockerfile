FROM docker.io/library/rust:alpine AS builder

## Add os build dependencies
RUN apk add --no-cache musl-dev=1.2.5-r0

## Copy the source files for the project
WORKDIR /actix-data-receiver
COPY ./Cargo.toml ./Cargo.toml
COPY ./src ./src

## Build the release
RUN cargo build --release

## Use a dirstroless image to run the compiled application binary
## https://github.com/GoogleContainerTools/distroless
## https://github.com/GoogleContainerTools/distroless/blob/main/cc/README.md
## https://github.com/GoogleContainerTools/distroless/blob/main/examples/rust/Dockerfile
FROM gcr.io/distroless/cc-debian12:nonroot AS final

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