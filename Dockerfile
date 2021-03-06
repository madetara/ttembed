FROM clux/muslrust:1.58.1-stable as chef
USER root
RUN cargo install cargo-chef
WORKDIR /app

FROM chef as planner
COPY Cargo.toml Cargo.lock ./
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl --bin ttembedder

FROM mcr.microsoft.com/playwright:focal as runtime
RUN apt-get update && apt-get upgrade -y
RUN apt-get install -y python3-pip
RUN python -m pip install --upgrade youtube-dl TikTokApi
RUN python -m playwright install

WORKDIR /app
COPY --from=builder /app/vendor /app/vendor
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/ttembedder ttembedder
ENV RUST_LOG="info"
ENTRYPOINT [ "/app/ttembedder" ]
