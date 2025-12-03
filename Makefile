.PHONY: build docker-build docker-push clean help docker

REGISTRY := registry.justalan.ru
IMAGE_NAME := amd-uprof-exporter
TAG := latest
FULL_IMAGE := justagod/$(IMAGE_NAME):$(TAG)

help:
	@echo "Available targets:"
	@echo "  build        - Build Rust binary"
	@echo "  docker-build - Build Docker image"
	@echo "  docker-push  - Push to registry"
	@echo "  clean        - Clean build artifacts"

build:
	cargo build --release

docker-build:
	docker build -t $(FULL_IMAGE) .

docker-push: docker-build
	docker push $(FULL_IMAGE)

docker: docker-push

clean:
	cargo clean
	docker rmi $(FULL_IMAGE) 2>/dev/null || true