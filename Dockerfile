FROM arm32v7/rust:latest

RUN mkdir /build
COPY * /build/
WORKDIR /build

RUN cargo build

ENTRYPOINT [ "cargo", "run", "--" ]
