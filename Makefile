.PHONY: docker-arm docker-x86

Dockerfile.x86: Dockerfile
	cat Dockerfile | sed -E 's/FROM.*/FROM rust:latest/' > Dockerfile.x86

docker-arm:
	docker build -t thebiggerguy/restful-sunsaver:latest .

docker-x86: Dockerfile.x86
	docker build -t thebiggerguy/restful-sunsaver:x86 -f Dockerfile.x86 .
