.PHONY: run docker-latest docker-dev

run:
	RUST_LOG=main=info cargo run -- --device=test_file

docker-latest:
	docker build --tag="thebiggerguy/restful-sunsaver:latest" .

docker-dev:
	docker build --tag="thebiggerguy/restful-sunsaver:dev" --file=Dockerfile.dev .

docker-dev-build: docker-dev
	docker run --rm -it -u ${UID}:$(id -g ${USER}) -v /etc/group:/etc/group:ro -v /etc/passwd:/etc/passwd:ro -v "$(pwd):/build" -w="/build" -e "CARGO_HOME=/build/.cargo" thebiggerguy/restful-sunsaver:dev cargo build

docker-dev-run: docker-dev
	docker run --rm -it -u ${UID}:$(id -g ${USER}) -v /etc/group:/etc/group:ro -v /etc/passwd:/etc/passwd:ro -v "$(pwd):/build" -w="/build" -e "CARGO_HOME=/build/.cargo" -e "RT_LOG=restful_sunsaver=info" --device=/dev/ttyUSB0 --group-add dialout thebiggerguy/restful-sunsaver:dev cargo run -- --device=/dev/ttyUSB
