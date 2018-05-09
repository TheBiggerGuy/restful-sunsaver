FROM rust:1.25-slim-stretch as build

# Install non rust things
RUN apt-get update && \
    apt-get install -y --no-install-recommends build-essential autoconf automake libtool cmake \
                                               llvm-3.9-dev libclang-3.9-dev clang-3.9 \
                                               libgit2-24 ca-certificates

# create a new empty shell project
RUN USER=root cargo new --bin restful-sunsaver
WORKDIR /restful-sunsaver

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# this build step will cache your dependencies
RUN cargo build --release
RUN rm src/*.rs

# copy your source tree
COPY ./src ./src

# build for release
RUN cargo build --release

# Start fresh
FROM debian:stretch-slim

# copy the build artifact from the build stage
COPY --from=build /restful-sunsaver/target/release/restful-sunsaver .

# Set up static files
COPY ./web/ /web/

# Set up service
EXPOSE 4000
ENTRYPOINT [ "./restful-sunsaver", "--port=4000", "--webroot=/web/" ]
