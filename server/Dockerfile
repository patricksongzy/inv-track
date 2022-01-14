FROM rustlang/rust:nightly
WORKDIR /usr/src/app
RUN cargo install sqlx-cli --no-default-features --features native-tls,postgres
RUN cargo install cargo-watch
EXPOSE ${ROCKET_PORT}
COPY . .