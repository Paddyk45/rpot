FROM rust:1-slim-bullseye
WORKDIR /opt/rpot
COPY . .
RUN cargo build --release
CMD [ "/opt/rpot/target/release/rpot" ]