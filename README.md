# restful-sunsaver
HTTP REST interface to the SunSaver MPPT via the serial ModBus

```bash
docker run --rm -it -u ${UID}:$(id -g ${USER}) -v /etc/group:/etc/group:ro -v /etc/passwd:/etc/passwd:ro -v "$(pwd):/build" -w="/build" -e "CARGO_HOME=/build/.cargo" -e "RUST_LOG=restful_sunsaver=debug" --device=/dev/SunSaver --group-add dialout --expose="4000" --publish="0.0.0.0:4000:4000" --env="PORT=4000" thebiggerguy/restful-sunsaver:dev cargo run -- --device=/dev/SunSaver
```
