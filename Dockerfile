FROM rust:1.62.1

WORKDIR /mars

RUN cargo install cargo-deb
COPY . .

RUN cargo deb

RUN apt install ./target/debian/mars_raw_utils_0.5.2_amd64.deb 

WORKDIR /data



