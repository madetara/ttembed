FROM clux/muslrust:1.80.1-stable as chef
USER root
RUN cargo install cargo-chef
WORKDIR /app

FROM chef as planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl --bin ttembedder

FROM alpine:3.18 as runtime
RUN apk -U add py3-pip bash
RUN python3 -m pip install -U --pre yt-dlp[default]
# some of yt-dlp features don't work in sh for some reason
ENV SHELL="/bin/bash"

WORKDIR /app
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/ttembedder ttembedder
ENV RUST_LOG="info"
ENTRYPOINT [ "/app/ttembedder" ]
