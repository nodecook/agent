FROM clux/muslrust:stable AS chef
USER root
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
ARG ARCH=x86_64-unknown-linux-musl
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --target ${ARCH} --recipe-path recipe.json
COPY . .
RUN cargo build --release --target ${ARCH} --bin nodecook-agent

FROM alpine AS runtime
ARG ARCH=x86_64-unknown-linux-musl
RUN addgroup -S nodecook && adduser -S nodecook -G nodecook
COPY --from=builder /app/target/${ARCH}/release/nodecook-agent /usr/local/bin/
USER nodecook
CMD ["/usr/local/bin/nodecook-agent"]
