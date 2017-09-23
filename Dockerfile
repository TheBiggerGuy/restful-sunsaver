FROM arm32v7/rust:latest

RUN apt-get update && \
    apt-get install -y --no-install-recommends build-essential autoconf automake cmake llvm-3.9-dev libclang-3.9-dev clang-3.9

RUN mkdir /build
COPY ./ /build/
WORKDIR /build
RUN cargo build
RUN cargo install

RUN apt-get purge -y build-essentail autoconf automake cmake llvm-3.9-dev libclang-3.9-dev clang-3.9 && \
    rm -rf /build && \
    rm -rf /var/lib/apt/lists/*

EXPOSE 4000
ENTRYPOINT [ "restful-sunsaver" ]
