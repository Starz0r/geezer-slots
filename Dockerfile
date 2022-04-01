# hastily derived from https://gist.github.com/ihrwein/1f11efc568601055f2c78eb471a41d99

# stage 1
FROM ubuntu:latest

ENV TARGET=x86_64-unknown-linux-musl
ENV BUILD_DIR=/src/target/x86_64-unknown-linux-musl/release/
ENV RUSTC_VERSION=1.58.0

RUN apt-get update && \
    apt-get install \
        curl \
        gcc \
        -y

RUN curl https://sh.rustup.rs -sSf -o /tmp/rustup-init.sh
RUN sh /tmp/rustup-init.sh -y

RUN ~/.cargo/bin/rustup target add ${TARGET}
RUN ~/.cargo/bin/rustup install ${RUSTC_VERSION}

ONBUILD COPY . /src
ONBUILD WORKDIR /src

ONBUILD RUN ~/.cargo/bin/cargo build --release --target=${TARGET}

# Build artifacts will be available in /app.
RUN mkdir /app
# Copy the "interesting" files into /app.
ONBUILD RUN find ${BUILD_DIR} \
                -regextype egrep \
                # The interesting binaries are all directly in ${BUILD_DIR}.
                -maxdepth 1 \
                # Well, binaries are executable.
                -executable \
                # Well, binaries are files.
                -type f \
                # Filter out tests.
                ! -regex ".*\-[a-fA-F0-9]{16,16}$" \
                # Copy the matching files into /app.
                -exec cp {} /app \;
				
# stage 2
FROM alpine:3.12.0

RUN apk update && apk upgrade && apk add ca-certificates
COPY --from=0 /app/ /app/

CMD ["/app/geezer-slots"]