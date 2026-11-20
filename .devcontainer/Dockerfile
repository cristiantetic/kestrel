# syntax=docker/dockerfile:1
#
# Kestrel dev container: Rust (from rust-toolchain.toml), seL4 17.0.0 build
# dependencies and QEMU 11.1.3 built from source.
#
# Pinned: the base image (by digest), the QEMU tarball (sha256), the seL4 tree
# (tag and commit), the seL4 Python packages (sel4-python-constraints.txt),
# rustup (sha256), the Rust toolchain (rust-toolchain.toml) and cargo-deny
# (sha256). Debian packages come from the bookworm stable archive at build time.
#
# Build from the repository root (the image copies rust-toolchain.toml):
#   docker build -t kestrel-dev -f .devcontainer/Dockerfile .

# debian:bookworm, multi-arch index digest.
ARG DEBIAN_IMAGE=debian:bookworm@sha256:268fb24c39bca92d326dd83ea3abebe4a3f7751c804ffef23bb6d857dd343ff8

# ---------------------------------------------------------------------------
# Stage 1: build QEMU. Debian ships no 11.1.x package, so it is built here and
# only /opt/qemu is copied into the final image.
# ---------------------------------------------------------------------------
FROM ${DEBIAN_IMAGE} AS qemu-build

ARG QEMU_VERSION=11.1.3
ARG QEMU_SHA256=5a8289c7940dc77d3873c81a6c29ba3afe1446fc9d36e77fa1658cdbecb84b22

RUN apt-get update -q \
    && apt-get install -y --no-install-recommends \
        bison \
        build-essential \
        ca-certificates \
        curl \
        flex \
        libfdt-dev \
        libglib2.0-dev \
        libpixman-1-dev \
        libslirp-dev \
        ninja-build \
        pkg-config \
        python3 \
        python3-venv \
        xz-utils \
        zlib1g-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /src
RUN set -eux; \
    curl -fsSLo qemu.tar.xz "https://download.qemu.org/qemu-${QEMU_VERSION}.tar.xz"; \
    echo "${QEMU_SHA256}  qemu.tar.xz" | sha256sum -c -; \
    tar -xJf qemu.tar.xz; \
    cd "qemu-${QEMU_VERSION}"; \
    ./configure \
        --prefix=/opt/qemu \
        --target-list=x86_64-softmmu,aarch64-softmmu \
        --disable-docs \
        --disable-werror \
        --enable-fdt=system \
        --enable-slirp; \
    make -j"$(nproc)"; \
    make install

# ---------------------------------------------------------------------------
# Stage 2: the dev image.
# ---------------------------------------------------------------------------
FROM ${DEBIAN_IMAGE}

# seL4 17.0.0 build dependencies (from seL4/rust-root-task-demo docker/Dockerfile),
# plus the shared libraries the QEMU build links against. Each host needs a cross
# gcc for the other architecture; on arm64 the native gcc is aarch64-linux-gnu-gcc.
RUN set -eux; \
    case "$(dpkg --print-architecture)" in \
        amd64) cross_gcc=gcc-aarch64-linux-gnu ;; \
        arm64) cross_gcc=gcc-x86-64-linux-gnu ;; \
        *) echo "unsupported architecture" >&2; exit 1 ;; \
    esac; \
    apt-get update -q; \
    apt-get install -y --no-install-recommends \
        "${cross_gcc}" \
        build-essential \
        ca-certificates \
        cmake \
        cpio \
        curl \
        device-tree-compiler \
        git \
        libclang-dev \
        libxml2-utils \
        ninja-build \
        python3-dev \
        python3-venv \
        libfdt1 \
        libglib2.0-0 \
        libpixman-1-0 \
        libslirp0 \
        zlib1g; \
    rm -rf /var/lib/apt/lists/*

# Python dependencies of the seL4 17.0.0 build, from the tagged kernel tree, with
# every transitive package held to the versions in sel4-python-constraints.txt.
ARG SEL4_VERSION=17.0.0
ARG SEL4_COMMIT=62e345c240a15671484530b4e6e069ce266c28d7
COPY .devcontainer/sel4-python-constraints.txt /tmp/sel4-python-constraints.txt
RUN set -eux; \
    git clone --quiet --depth 1 --branch "${SEL4_VERSION}" https://github.com/seL4/seL4.git /tmp/seL4; \
    test "$(git -C /tmp/seL4 rev-parse HEAD)" = "${SEL4_COMMIT}"; \
    python3 -m venv /opt/sel4-venv; \
    /opt/sel4-venv/bin/pip install --no-cache-dir \
        -c /tmp/sel4-python-constraints.txt /tmp/seL4/tools/python-deps; \
    rm -rf /tmp/seL4 /tmp/sel4-python-constraints.txt

ENV PATH=/opt/sel4-venv/bin:/opt/qemu/bin:${PATH}

# Non-root user.
RUN groupadd --gid 1000 dev \
    && useradd --uid 1000 --gid 1000 --create-home --shell /bin/bash dev

# Rust: a pinned rustup, then the toolchain named in rust-toolchain.toml.
ARG RUSTUP_VERSION=1.29.3
ARG RUSTUP_SHA256_AMD64=b1c8cb14e30889e7e2999e6b69e62d9e9b9ffc0f4df8180564d306d9680e6c55
ARG RUSTUP_SHA256_ARM64=9f608c142e0e38a78957d0661ba1dfaf0b6fb37184cfaa4b9a0aa57af6184282
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:${PATH}
COPY rust-toolchain.toml /tmp/toolchain/rust-toolchain.toml
RUN set -eux; \
    case "$(dpkg --print-architecture)" in \
        amd64) triple=x86_64-unknown-linux-gnu; sha="${RUSTUP_SHA256_AMD64}" ;; \
        arm64) triple=aarch64-unknown-linux-gnu; sha="${RUSTUP_SHA256_ARM64}" ;; \
        *) echo "unsupported architecture" >&2; exit 1 ;; \
    esac; \
    curl -fsSLo /tmp/rustup-init "https://static.rust-lang.org/rustup/archive/${RUSTUP_VERSION}/${triple}/rustup-init"; \
    echo "${sha}  /tmp/rustup-init" | sha256sum -c -; \
    chmod +x /tmp/rustup-init; \
    /tmp/rustup-init -y --no-modify-path --profile minimal --default-toolchain none; \
    rm /tmp/rustup-init; \
    cd /tmp/toolchain; \
    rustup toolchain install; \
    rustup default "$(rustup show active-toolchain | cut -d' ' -f1)"; \
    cd /; \
    rm -rf /tmp/toolchain; \
    # Writable by any uid: Dev Containers may remap dev's uid, and `docker run --user`
    # may use any uid; cargo must still write its registry cache.
    chmod -R a+rwX "${RUSTUP_HOME}" "${CARGO_HOME}"

# cargo-deny, pinned prebuilt release.
ARG CARGO_DENY_VERSION=0.22.0
ARG CARGO_DENY_SHA256_AMD64=98c56e6a408bfa5c3c60744b6015aaa6aa08f743fdeb88386d06fca035bb1786
ARG CARGO_DENY_SHA256_ARM64=4818d44cdcac285776336504fca722493ab634d3377b94a462e98aad3f41484a
RUN set -eux; \
    case "$(dpkg --print-architecture)" in \
        amd64) triple=x86_64-unknown-linux-musl; sha="${CARGO_DENY_SHA256_AMD64}" ;; \
        arm64) triple=aarch64-unknown-linux-musl; sha="${CARGO_DENY_SHA256_ARM64}" ;; \
        *) echo "unsupported architecture" >&2; exit 1 ;; \
    esac; \
    name="cargo-deny-${CARGO_DENY_VERSION}-${triple}"; \
    curl -fsSLo /tmp/cargo-deny.tar.gz "https://github.com/EmbarkStudios/cargo-deny/releases/download/${CARGO_DENY_VERSION}/${name}.tar.gz"; \
    echo "${sha}  /tmp/cargo-deny.tar.gz" | sha256sum -c -; \
    tar -xzf /tmp/cargo-deny.tar.gz -C /tmp; \
    install -m 0755 "/tmp/${name}/cargo-deny" /usr/local/bin/cargo-deny; \
    rm -rf /tmp/cargo-deny.tar.gz "/tmp/${name}"

# Copied last so that the QEMU build stage runs in parallel with the steps above.
COPY --from=qemu-build /opt/qemu /opt/qemu

# The repository is bind-mounted from the host and may be owned by another uid.
RUN git config --system --add safe.directory /work

# Debian's /etc/profile resets PATH for login shells, which would hide the
# tools above. VS Code reads the container environment through a login shell,
# so restore the image's tool paths there too.
RUN printf '%s\n' \
        'export PATH="/usr/local/cargo/bin:/opt/qemu/bin:/opt/sel4-venv/bin:${PATH}"' \
        > /etc/profile.d/kestrel-path.sh

USER dev
WORKDIR /work
