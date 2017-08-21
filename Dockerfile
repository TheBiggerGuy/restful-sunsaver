FROM arm32v7/rust:latest


COPY * /build/
COPY src /build/src
WORKDIR /build

RUN cargo build

ENTRYPOINT ["cargo", "run", "--"]
