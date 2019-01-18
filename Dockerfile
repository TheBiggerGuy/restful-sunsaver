FROM multiarch/debian-debootstrap:armhf-buster-slim as base

LABEL maintainer="restful.sunsaver@thebiggerguy.net"

RUN apt-get update && \
    apt-get install -y --no-install-recommends curl && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*


FROM base as build

# Install non rust things
RUN apt-get update && \
    apt-get install -y --no-install-recommends gcc autoconf automake libtool cmake \
                                               llvm-dev libclang-7-dev clang-7 \
                                               libgit2-27 curl ca-certificates
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

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
RUN touch src/main.rs

# build for release
RUN cargo build --release
RUN ./target/release/restful-sunsaver --version

# Start fresh
FROM base

# copy the build artifact from the build stage
COPY --from=build /restful-sunsaver/target/release/restful-sunsaver /usr/local/bin

# Set up static files
COPY ./web/ /web/

# Set up the runner script
COPY docker-runner.sh /usr/local/bin/docker-runner

# Set up service
EXPOSE 4000

HEALTHCHECK --start-period=30s --interval=5m --timeout=3s --retries=2 \
    CMD curl -fsS http://localhost:4000/ || exit 1

ENTRYPOINT ["docker-runner"]
