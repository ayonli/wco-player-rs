#!/bin/bash

# Script to build the web application for OpenWrt using Docker
# and extract the results to target/dx/web/release/open-wrt

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
IMAGE_NAME="wco-player-web-builder"
CONTAINER_NAME="wco-player-web-temp"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
OUTPUT_DIR="$PROJECT_ROOT/target/dx/web/release/open-wrt"
DOCKERFILE_PATH="$SCRIPT_DIR/Dockerfile"

echo -e "${GREEN}Building web application for OpenWrt...${NC}"

# Change to project root (build context must be project root to access all files)
cd "$PROJECT_ROOT"

# Build the Docker image
echo -e "${YELLOW}Step 1/4: Building Docker image...${NC}"
docker build --platform linux/arm64 --target builder -f "$DOCKERFILE_PATH" -t "$IMAGE_NAME" . || {
    echo -e "${RED}Error: Docker build failed${NC}"
    exit 1
}

# Clean and create output directory
echo -e "${YELLOW}Step 2/4: Preparing output directory...${NC}"
if [ -d "$OUTPUT_DIR" ]; then
    echo "  - Removing old files from $OUTPUT_DIR"
    rm -rf "$OUTPUT_DIR"
fi
mkdir -p "$OUTPUT_DIR"

# Create a temporary container from the builder image
echo -e "${YELLOW}Step 3/4: Extracting build artifacts...${NC}"
CONTAINER_ID=$(docker create "$IMAGE_NAME")

# Extract the entire build output from target/dx/web/release/web
echo "  - Extracting build artifacts..."

# Try to find the actual output location
# Use docker run to check paths (since created container is stopped)
OUTPUT_PATH=""
POSSIBLE_PATHS=(
    "/app/target/dx/web/release/web"
    "/app/target/dx/web/release"
    "/app/target/dx/web"
    "/app/packages/web/dist"
)

for path in "${POSSIBLE_PATHS[@]}"; do
    if docker run --rm "$IMAGE_NAME" test -d "$path" 2>/dev/null; then
        echo "  Found output at: $path"
        OUTPUT_PATH="$path"
        break
    fi
done

if [ -z "$OUTPUT_PATH" ]; then
    echo -e "${RED}Error: Build output not found${NC}"
    echo ""
    echo "Searched in:"
    for path in "${POSSIBLE_PATHS[@]}"; do
        echo "  - $path"
    done
    echo ""
    echo "Checking container structure..."
    echo "Target directory contents:"
    docker run --rm "$IMAGE_NAME" find /app/target -maxdepth 4 -type d 2>/dev/null | head -20 || echo "  (target directory not found)"
    echo ""
    echo "Dx directory contents:"
    docker run --rm "$IMAGE_NAME" find /app -type d -name "dx" 2>/dev/null | head -10 || echo "  (dx directory not found)"
    echo ""
    echo "All files in /app/target:"
    docker run --rm "$IMAGE_NAME" find /app/target -type f 2>/dev/null | head -30 || echo "  (no files found)"
    echo ""
    echo "Run ./debug-build-output.sh for more detailed debugging"
    docker rm "$CONTAINER_ID" > /dev/null 2>&1 || true
    exit 1
fi

# Copy the entire directory to output
docker cp "$CONTAINER_ID:$OUTPUT_PATH/." "$OUTPUT_DIR/" || {
    echo -e "${RED}Error: Failed to extract build artifacts from $OUTPUT_PATH${NC}"
    docker rm "$CONTAINER_ID" > /dev/null 2>&1 || true
    exit 1
}

# Make all binaries executable
find "$OUTPUT_DIR" -type f -executable -exec chmod +x {} \; 2>/dev/null || true
# Also try to make common binary names executable
for binary in "$OUTPUT_DIR/web" "$OUTPUT_DIR/web-server" "$OUTPUT_DIR/server"; do
    if [ -f "$binary" ]; then
        chmod +x "$binary"
    fi
done

# Clean up the temporary container
echo -e "${YELLOW}Step 4/4: Cleaning up...${NC}"
docker rm "$CONTAINER_ID" > /dev/null 2>&1

# Display results
echo -e "${GREEN}✓ Build completed successfully!${NC}"
echo ""
echo "Output location: $OUTPUT_DIR"
if [ -d "$OUTPUT_DIR" ]; then
    echo ""
    echo "Contents:"
    ls -lh "$OUTPUT_DIR" | tail -n +2 || true
fi
echo ""
echo -e "${GREEN}You can now deploy these files to your OpenWrt server.${NC}"
