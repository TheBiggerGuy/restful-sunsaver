FROM arm32v7/rust:latest

RUN apt-get update && \
    apt-get install -y --no-install-recommends build-essential autoconf automake libtool pkg-config git ca-certificates llvm-3.9-dev libclang-3.9-dev clang-3.9
RUN mkdir /build && \
    cd /build && \
    git clone https://github.com/stephane/libmodbus.git && \
    cd libmodbus && \
    git checkout f9358460ee1f62bcac716ad0444b3bbe7628b204 && \
    ./autogen.sh && \
    ./configure --disable-tests && \
    make -j 3 && \
    make install && \
    ldconfig && \
    cd / && \
    rm -rf build

RUN mkdir /build
COPY ./ /build/
WORKDIR /build

RUN cargo build

EXPOSE 4000
ENTRYPOINT [ "cargo", "run", "--" ]
