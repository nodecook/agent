# syntax=docker/dockerfile:1.7
ARG BUILDER_IMAGE=messense/rust-musl-cross:x86_64-musl

FROM ${BUILDER_IMAGE} AS chef
WORKDIR /home/rust/src
# messense images set CARGO_BUILD_TARGET to the cross target, which would
# make `cargo install` build cargo-chef for that target instead of the host.
RUN env -u CARGO_BUILD_TARGET cargo install cargo-chef --locked

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
ARG TARGET=x86_64-unknown-linux-musl
COPY --from=planner /home/rust/src/recipe.json recipe.json
RUN cargo chef cook --release --target ${TARGET} --recipe-path recipe.json
COPY . .
RUN cargo build --locked --release --target ${TARGET}

FROM scratch AS output
ARG TARGET=x86_64-unknown-linux-musl
COPY --from=builder /home/rust/src/target/${TARGET}/release/nodecook-agent /nodecook-agent

FROM alpine AS runtime
ARG TARGET=x86_64-unknown-linux-musl
RUN addgroup -S nodecook && adduser -S nodecook -G nodecook
COPY --from=builder /home/rust/src/target/${TARGET}/release/nodecook-agent /usr/local/bin/
USER nodecook
CMD ["/usr/local/bin/nodecook-agent"]
