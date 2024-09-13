# Stage 1: Build stage using a lightweight image to generate the configuration file
FROM alpine:3.18 AS builder

# Install necessary tools (like sh) for generating the config file
RUN apk add --no-cache bash

# Set environment variables
ENV GARAGE_RPC_SECRET=${GARAGE_RPC_SECRET}

# Create a script to generate the garage.toml file
RUN echo '#!/bin/sh' > /create_garage_config.sh && \
    echo 'cat <<EOF > /garage.toml' >> /create_garage_config.sh && \
    echo 'metadata_dir = "/var/lib/garage/meta"' >> /create_garage_config.sh && \
    echo 'data_dir = "/var/lib/garage/data"' >> /create_garage_config.sh && \
    echo 'replication_mode = "none"' >> /create_garage_config.sh && \
    echo 'rpc_bind_addr = "[::]:3901"' >> /create_garage_config.sh && \
    echo 'rpc_secret = "${GARAGE_RPC_SECRET}"' >> /create_garage_config.sh && \
    echo 'bootstrap_peers = []' >> /create_garage_config.sh && \
    echo '[s3_api]' >> /create_garage_config.sh && \
    echo 's3_region = "gibber"' >> /create_garage_config.sh && \
    echo 'api_bind_addr = "[::]:3900"' >> /create_garage_config.sh && \
    echo '[s3_web]' >> /create_garage_config.sh && \
    echo 'bind_addr = "[::]:3902"' >> /create_garage_config.sh && \
    echo 'root_domain = "localhost"' >> /create_garage_config.sh && \
    echo 'EOF' >> /create_garage_config.sh

# Make the script executable
RUN chmod +x /create_garage_config.sh

# Stage 2: Production image based on scratch
FROM scratch

# Set environment variables
ENV RUST_BACKTRACE=1
ENV RUST_LOG=garage=info

# Copy the binary from the build context
COPY result-bin/bin/garage /

# Copy the generated garage.toml file from the builder stage
COPY --from=builder /create_garage_config.sh /create_garage_config.sh

# When the container starts, first generate the garage.toml, then run the server
CMD [ "/bin/sh", "-c", "/create_garage_config.sh && /garage", "server" ]
