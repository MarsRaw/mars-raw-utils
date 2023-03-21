FROM rust:latest

WORKDIR /mars

RUN apt install libssl-dev imagemagick
COPY . .

RUN cargo test --manifest-path sciimg/Cargo.toml
RUN cargo test 
RUN cargo install --path .

RUN mkdir ~/.marsdata/
RUN cp -vr mars-raw-utils-data/caldata/* ~/.marsdata/

RUN mkdir -p /data/bin
RUN cp examples/*sh /data/bin
RUN chmod +x /data/bin/*sh
ENV PATH="$PATH:/data/bin"


RUN mkdir -p /data/MSL
RUN mkdir -p /data/M20
RUN mkdir -p /data/NSYT

WORKDIR /data