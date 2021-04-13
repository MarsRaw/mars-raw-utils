FROM ubuntu:20.04

RUN apt-get update
RUN apt-get install -y build-essential curl

RUN bash -c "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y"

ENV PATH="/root/.cargo/bin:${PATH}"

RUN DEBIAN_FRONTEND="noninteractive" apt-get -y install libopencv-dev libssl-dev libclang1 libclang-dev librust-clang-sys-dev librust-clang-sys+runtime-dev clang-tools 
WORKDIR /mars

RUN cargo install cargo-deb
COPY . .

RUN cargo deb

RUN apt install ./target/debian/mars_raw_utils_0.1.0_amd64.deb 

WORKDIR /data



