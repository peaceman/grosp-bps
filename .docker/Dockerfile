FROM rust:1.46 as compile-image

RUN USER=root cargo new --bin /usr/src/balancing-playlist-spreader
WORKDIR /usr/src/balancing-playlist-spreader

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release
RUN rm src/*.rs

COPY ./src ./src
RUN cargo build --release

FROM debian:buster-slim as runtime-image

RUN apt-get update \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y ca-certificates tzdata libssl-dev \
    && rm -rf /var/lib/apt/lists*

ENV APP_USER=appuser \
    APP_DIR=/opt/app

RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p $APP_DIR

COPY --from=compile-image /usr/src/balancing-playlist-spreader/target/release/balancing-playlist-spreader $APP_DIR/balancing-playlist-spreader

RUN chown -R $APP_USER:$APP_USER $APP_DIR

USER $APP_USER
WORKDIR $APP_DIR

CMD ["./balancing-playlist-spreader"]
