FROM rust:1.51.0

WORKDIR /mars

RUN cargo install cargo-deb
COPY . .

RUN cargo deb

RUN apt install ./target/debian/mars_raw_utils_0.1.5_amd64.deb 

WORKDIR /data



