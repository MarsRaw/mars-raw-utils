FROM rust:latest

WORKDIR /mars

RUN apt install libssl-dev
COPY . .

RUN cargo install --path .

RUN mkdir ~/.marsdata/
RUN cp -vr mars-raw-utils-data/caldata/* ~/.marsdata/

RUN mkdir -p /data/MSL
RUN mkdir -p /data/M20
RUN mkdir -p /data/NSYT

WORKDIR /data