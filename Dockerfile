FROM rust:1-alpine3.23 AS builder
ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN apk add --no-cache musl-dev \
    # Required for git-version
    git

WORKDIR /pumpkin
COPY . /pumpkin

# Railway builds from a source archive, which does not populate Git submodules.
# Fetch the pinned WIT definitions when they are missing from the build context.
RUN test -f pumpkin-plugin-wit/v0.1/world.wit || ( \
    rm -rf pumpkin-plugin-wit && \
    git init pumpkin-plugin-wit && \
    git -C pumpkin-plugin-wit remote add origin https://github.com/Pumpkin-MC/pumpkin-plugin-wit && \
    git -C pumpkin-plugin-wit fetch --depth=1 origin 3773e86ec7ce68eb53e879f613aeb3b2198d9522 && \
    git -C pumpkin-plugin-wit checkout --detach FETCH_HEAD \
    )

RUN rustup show active-toolchain || rustup toolchain install
RUN rustup component add rustfmt

# build release
RUN cargo build --release && cp target/release/pumpkin ./pumpkin.release

FROM alpine:3.24

COPY --from=builder /pumpkin/pumpkin.release /bin/pumpkin

# set workdir to /pumpkin, this is required to influence the PWD environment variable
# it allows for bind mounting the server files without overwriting the pumpkin
# executable (without requiring an `docker cp`-ing the binary to the host folder)
WORKDIR /pumpkin

RUN apk add --no-cache libgcc && chown 2613:2613 .

ENV RUST_BACKTRACE=1
EXPOSE 25565
USER 2613:2613
ENTRYPOINT [ "/bin/pumpkin" ]
HEALTHCHECK CMD nc -z 127.0.0.1 25565 || exit 1
