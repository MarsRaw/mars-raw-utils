FROM rust:latest

WORKDIR /mars

# libssl needed for build. ImageMagick needed for image processing
RUN apt install libssl-dev imagemagick

# Copy project resources
COPY . .

# Test sciimg core library
RUN cargo test --manifest-path sciimg/Cargo.toml

# Test MRU
RUN cargo test 

# Build and install MRU
RUN cargo install --path .

# Create and populate calibration data folder
RUN mkdir ~/.marsdata/
RUN cp -vr mars-raw-utils-data/caldata/* ~/.marsdata/

# Create and populate scripts
RUN mkdir -p /data/bin
RUN cp examples/*sh /data/bin
RUN chmod +x /data/bin/*sh

# Create and populate userdir bin scripts
RUN mkdir ~/bin
RUN cp examples/go.sh ~/bin/
RUN cp examples/get.sh ~/bin/
RUN chmod +x ~/bin/*sh

# Expand $PATH
ENV PATH="$PATH:/data/bin:~/bin"

# Symlink data path shortcut
RUN ln -s /data ~/data

# Shortcut aliases
RUN echo 'alias go=". /data/bin/go.sh"' >> ~/.bashrc
RUN echo 'alias get=". /data/bin/get.sh"' >> ~/.bashrc
RUN echo 'alias m20="cd /data/M20"' >> ~/.bashrc
RUN echo 'alias msl="cd /data/MSL"' >> ~/.bashrc
RUN echo 'alias insight="cd /data/NSYT"' >> ~/.bashrc

# Mission data root directories
RUN mkdir -p /data/MSL
RUN mkdir -p /data/M20
RUN mkdir -p /data/NSYT

# Starting shell $CWD
WORKDIR /data