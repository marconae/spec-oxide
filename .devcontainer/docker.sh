#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
IMAGE_NAME="spec-oxide-dev"
CONTAINER_NAME="spec-oxide-dev-container"

usage() {
    echo "Usage: $0 [build|start|run|stop|shell]"
    echo ""
    echo "Commands:"
    echo "  build    Build the Docker image"
    echo "  start    Start the container (builds if image doesn't exist)"
    echo "  run      Build and start the container"
    echo "  stop     Stop and remove the container"
    echo "  shell    Open a shell in the running container"
    echo ""
    echo "Options:"
    echo "  -h, --help    Show this help message"
    exit 1
}

build() {
    echo "Building Docker image: $IMAGE_NAME"
    docker build \
        --build-arg TZ="$(cat /etc/timezone 2>/dev/null || echo UTC)" \
        -t "$IMAGE_NAME" \
        -f "$SCRIPT_DIR/Dockerfile" \
        "$SCRIPT_DIR"
    echo "Build complete."
}

start() {
    if ! docker image inspect "$IMAGE_NAME" &>/dev/null; then
        echo "Image not found. Building first..."
        build
    fi

    if docker ps -q -f name="$CONTAINER_NAME" | grep -q .; then
        echo "Container is already running."
        return 0
    fi

    if docker ps -aq -f name="$CONTAINER_NAME" | grep -q .; then
        echo "Removing stopped container..."
        docker rm "$CONTAINER_NAME"
    fi

    echo "Starting container: $CONTAINER_NAME"
    docker run -d \
        --name "$CONTAINER_NAME" \
        --hostname dev-sandbox \
        -v "$PROJECT_DIR:/workspace" \
        -p 2222:22 \
        "$IMAGE_NAME"
    echo "Container started."
    echo "SSH access: ssh -p 2222 coder@localhost (password: coder)"
}

stop() {
    if docker ps -q -f name="$CONTAINER_NAME" | grep -q .; then
        echo "Stopping container: $CONTAINER_NAME"
        docker stop "$CONTAINER_NAME"
    fi

    if docker ps -aq -f name="$CONTAINER_NAME" | grep -q .; then
        echo "Removing container: $CONTAINER_NAME"
        docker rm "$CONTAINER_NAME"
    fi

    echo "Container stopped and removed."
}

open_shell() {
    if ! docker ps -q -f name="$CONTAINER_NAME" | grep -q .; then
        echo "Container is not running. Start it first with: $0 start"
        exit 1
    fi

    docker exec -it "$CONTAINER_NAME" /bin/bash
}

if [[ $# -eq 0 ]]; then
    usage
fi

case "$1" in
    build)
        build
        ;;
    start)
        start
        ;;
    run)
        build
        start
        ;;
    stop)
        stop
        ;;
    shell)
        open_shell
        ;;
    -h|--help)
        usage
        ;;
    *)
        echo "Unknown command: $1"
        usage
        ;;
esac
