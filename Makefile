.PHONY: run docker-build-latest docker-run-latest

run:
	RUST_LOG=main=info cargo run -- --device=test_file

docker-build-latest:
	docker build --tag="thebiggerguy/restful-sunsaver:latest" .

docker-run-latest: docker-build-latest
	docker run --rm -it -e "RT_LOG=restful_sunsaver=info" --device=/dev/ttyUSB0 --group-add dialout thebiggerguy/restful-sunsaver:latest --device=/dev/ttyUSB
