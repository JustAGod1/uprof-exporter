FROM rust:1.91.1-slim as builder

WORKDIR /build

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source
COPY src ./src

# Build release binary
RUN cargo build --release

# Runtime stage
FROM debian:sid-slim

# Install required runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates libelf-dev wget bzip2 && \
    rm -rf /var/lib/apt/lists/*

RUN wget 'https://nc.justalan.ru/public.php/dav/files/cKEJDoxC6WWRX4c/?accept=zip' -O '/tmp/AMDuProf_Linux_x64_5.1.701.tar.bz2' && \
    tar -xjf /tmp/AMDuProf_Linux_x64_5.1.701.tar.bz2 -C /opt/ && \
    rm /tmp/AMDuProf_Linux_x64_5.1.701.tar.bz2

# Copy binary from builder
COPY --from=builder /build/target/release/uprof-exporter /usr/local/bin/

EXPOSE 9100

CMD ["/usr/local/bin/uprof-exporter"]