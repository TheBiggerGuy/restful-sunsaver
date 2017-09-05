.PHONY: docker-arm docker-x86 run

run:
	RUST_LOG=main=info cargo run -- --device=test_file

armv7hf-debian-qemu:
	git submodule update --init

docker: armv7hf-debian-qemu
	docker build -t thebiggerguy/restful-sunsaver:latest .

