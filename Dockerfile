FROM rust:1.58.1

WORKDIR /mars

RUN cargo install cargo-deb
COPY . .

RUN cargo deb

RUN apt install ./target/debian/mars_raw_utils_0.5.1_amd64.deb 

WORKDIR /data



