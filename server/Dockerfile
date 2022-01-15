FROM rustlang/rust:nightly
WORKDIR /usr/src/app/server
RUN cargo install sqlx-cli --no-default-features --features native-tls,postgres
RUN cargo install cargo-watch
EXPOSE ${ROCKET_PORT}