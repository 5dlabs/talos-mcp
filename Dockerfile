# Use a minimal base image for running the binary
FROM debian:bookworm-slim

# Install essential packages and clean up in one layer
RUN apt-get update && \
    apt-get install -y \
        ca-certificates \
        curl \
        --no-install-recommends && \
    rm -rf /var/lib/apt/lists/*

# Create a non-root user for security
RUN useradd --create-home --shell /bin/bash --user-group --uid 1000 talos

# Copy the pre-built binary from the CI pipeline
COPY talos-mcp-server-linux /usr/local/bin/talos-mcp-server

# Make sure the binary is executable
RUN chmod +x /usr/local/bin/talos-mcp-server

# Create directory for Talos configuration
RUN mkdir -p /home/talos/.talos && \
    chown -R talos:talos /home/talos/.talos

# Switch to non-root user
USER talos
WORKDIR /home/talos

# Set default environment variables
ENV TALOSCONFIG=/home/talos/.talos/config

# Expose no ports (MCP uses stdio)
# Health check to ensure the binary is working
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD talos-mcp-server --help > /dev/null || exit 1

# Default command
CMD ["talos-mcp-server"]