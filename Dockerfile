FROM arm32v7/rust:latest

COPY /usr/bin/qemu-arm-static /usr/bin/qemu-arm-static

RUN apt-get update && \
    apt-get install -y --no-install-recommends build-essential autoconf automake cmake \
                                               llvm-3.9-dev libclang-3.9-dev clang-3.9 \
                                               libgit2-24 ca-certificates

COPY ./ /build/
COPY ./web/ /web/
WORKDIR /build
RUN cargo build --release
RUN cargo install

RUN apt-get purge -y build-essential autoconf automake cmake \
                     llvm-3.9-dev libclang-3.9-dev clang-3.9 \
                     libgit2-24 ca-certificates \
                     qemu-user-static binfmt-support && \
    rm -rf /build && \
    rm -rf /var/lib/apt/lists/*

EXPOSE 4000
ENTRYPOINT [ "restful-sunsaver", "--port=4000", "--webroot=/web/" ]
