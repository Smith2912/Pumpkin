# syntax=docker/dockerfile:1

FROM eclipse-temurin:25-jdk-noble@sha256:3eb81ed94d8c1a34422f19f8188548bdf02cae69c91d0328afdbb7abed90f617 AS builder

ARG PATCHBUKKIT_COMMIT=fcadfea17adf8ccde166b9e524f6d7029ead5a0e
ARG RUST_TOOLCHAIN=nightly-2026-03-05
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    CARGO_TARGET_DIR=/cargo-target \
    RUSTUP_TOOLCHAIN=${RUST_TOOLCHAIN} \
    PATH=/usr/local/cargo/bin:${PATH}

RUN apt-get update && apt-get install -y --no-install-recommends \
        build-essential \
        ca-certificates \
        curl \
        git \
        pkg-config \
    && rm -rf /var/lib/apt/lists/* \
    && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
        | sh -s -- -y --profile minimal --default-toolchain ${RUST_TOOLCHAIN}

# Pin PatchBukkit so upstream changes cannot silently alter a Railway rebuild.
RUN git init /patchbukkit \
    && git -C /patchbukkit remote add origin https://github.com/Pumpkin-MC/PatchBukkit.git \
    && git -C /patchbukkit fetch --depth=1 origin ${PATCHBUKKIT_COMMIT} \
    && git -C /patchbukkit checkout --detach FETCH_HEAD \
    && test "$(git -C /patchbukkit rev-parse HEAD)" = "${PATCHBUKKIT_COMMIT}"

WORKDIR /pumpkin
COPY . /pumpkin

# Replay the reviewed baseline and public-server batches with normal context.
# Every patch is checked immediately before it is applied so a stale or
# partially matching source tree fails the build instead of producing an
# unreviewed adapter.
RUN git -C /patchbukkit apply --check /pumpkin/docker/patchbukkit-26.2.patch \
    && git -C /patchbukkit apply /pumpkin/docker/patchbukkit-26.2.patch \
    && git -C /patchbukkit apply --check /pumpkin/docker/patchbukkit-public-server-metadata.patch \
    && git -C /patchbukkit apply /pumpkin/docker/patchbukkit-public-server-metadata.patch \
    && git -C /patchbukkit apply --check /pumpkin/docker/patchbukkit-public-server-interaction.patch \
    && git -C /patchbukkit apply /pumpkin/docker/patchbukkit-public-server-interaction.patch \
    && git -C /patchbukkit apply --check /pumpkin/docker/patchbukkit-public-server-conformance.patch \
    && git -C /patchbukkit apply /pumpkin/docker/patchbukkit-public-server-conformance.patch \
    && git -C /patchbukkit diff --check

# Build the Java bridge only after applying the compatibility/lifecycle patch.
RUN --mount=type=cache,id=s/c4f5b7dc-c554-4f52-a998-ab086c9613f2-/root/.gradle,target=/root/.gradle \
    cd /patchbukkit/java \
    && ./gradlew --no-daemon jar

# Railway builds from a source archive, which does not populate Git submodules.
# Fetch the pinned WIT definitions when they are missing from the build context.
RUN test -f pumpkin-plugin-wit/v0.1/world.wit || ( \
    rm -rf pumpkin-plugin-wit && \
    git init pumpkin-plugin-wit && \
    git -C pumpkin-plugin-wit remote add origin https://github.com/Pumpkin-MC/pumpkin-plugin-wit && \
    git -C pumpkin-plugin-wit fetch --depth=1 origin 3773e86ec7ce68eb53e879f613aeb3b2198d9522 && \
    git -C pumpkin-plugin-wit checkout --detach FETCH_HEAD \
    )

# Build Pumpkin and PatchBukkit with one toolchain and the exact same local
# Pumpkin packages. Native plugins are ABI-sensitive, so this version matching
# is required even when their numeric plugin API version is unchanged.
RUN --mount=type=cache,id=s/c4f5b7dc-c554-4f52-a998-ab086c9613f2-/usr/local/cargo/registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=s/c4f5b7dc-c554-4f52-a998-ab086c9613f2-/usr/local/cargo/git,target=/usr/local/cargo/git \
    --mount=type=cache,id=s/c4f5b7dc-c554-4f52-a998-ab086c9613f2-/cargo-target,target=/cargo-target \
    cargo build --locked --release --package pumpkin \
    && cp /cargo-target/release/pumpkin /pumpkin.release \
    && printf '\n[patch."https://github.com/Pumpkin-MC/Pumpkin.git"]\n\
pumpkin = { path = "/pumpkin/pumpkin" }\n\
pumpkin-api-macros = { path = "/pumpkin/pumpkin-api-macros" }\n\
pumpkin-data = { path = "/pumpkin/pumpkin-data" }\n\
pumpkin-protocol = { path = "/pumpkin/pumpkin-protocol" }\n\
pumpkin-util = { path = "/pumpkin/pumpkin-util" }\n' \
        >> /patchbukkit/rust/Cargo.toml \
    && cargo -Z bindeps update --manifest-path /patchbukkit/rust/Cargo.toml \
        -p pumpkin \
        -p pumpkin-api-macros \
        -p pumpkin-data \
        -p pumpkin-protocol \
        -p pumpkin-util \
    && cargo -Z bindeps build --locked --release --manifest-path /patchbukkit/rust/Cargo.toml \
    && cp /cargo-target/release/libpatchbukkit.so /libpatchbukkit.so

FROM eclipse-temurin:25-jre-noble@sha256:2f1da100788559b397bcf48c736169ea5b070bde84e55f203bbee8e83d87a175

ARG PUFFERPANEL_VERSION=3.0.9
ARG PUFFERPANEL_SHA256=ae6b74b00c0383a3f0f9fb81a99b37bb64e3028288d46b39617ff6945bc01379

RUN apt-get update && apt-get install -y --no-install-recommends \
        curl \
        gosu \
        libgcc-s1 \
        libstdc++6 \
        netcat-openbsd \
    && rm -rf /var/lib/apt/lists/* \
    && curl --proto '=https' --tlsv1.2 -fsSL \
        "https://github.com/pufferpanel/pufferpanel/releases/download/v${PUFFERPANEL_VERSION}/pufferpanel_${PUFFERPANEL_VERSION}_amd64.deb" \
        -o /tmp/pufferpanel.deb \
    && echo "${PUFFERPANEL_SHA256}  /tmp/pufferpanel.deb" | sha256sum -c - \
    && dpkg-deb --extract /tmp/pufferpanel.deb /tmp/pufferpanel \
    && install -m 0755 /tmp/pufferpanel/usr/sbin/pufferpanel /usr/local/bin/pufferpanel \
    && mkdir -p /var/www \
    && cp -a /tmp/pufferpanel/var/www/pufferpanel /var/www/pufferpanel \
    && rm -rf /tmp/pufferpanel /tmp/pufferpanel.deb \
    && groupadd --gid 2613 pumpkin \
    && useradd --uid 2613 --gid pumpkin --home-dir /pumpkin --shell /usr/sbin/nologin pumpkin \
    && mkdir -p /pumpkin /opt/pumpkin/plugins /opt/pumpkin/pufferpanel \
    && chown 2613:2613 /pumpkin

COPY --from=builder /pumpkin.release /bin/pumpkin
COPY --from=builder /libpatchbukkit.so /opt/pumpkin/plugins/libpatchbukkit.so
COPY docker/pufferpanel-config.json /opt/pumpkin/pufferpanel/config.json
COPY docker/pufferpanel-pumpkin.json /opt/pumpkin/pufferpanel/pumpkin.json
COPY --chmod=755 docker/pumpkin-entrypoint.sh /usr/local/bin/pumpkin-entrypoint

WORKDIR /pumpkin
ENV RUST_BACKTRACE=1
EXPOSE 25565 5657 8080
USER 2613:2613
ENTRYPOINT ["/usr/local/bin/pumpkin-entrypoint"]
HEALTHCHECK --interval=30s --timeout=5s --start-period=120s --retries=5 \
    CMD nc -z 127.0.0.1 25565 && nc -z 127.0.0.1 8080 || exit 1
