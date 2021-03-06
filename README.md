# restful-sunsaver
[![Build Status](https://travis-ci.org/TheBiggerGuy/restful-sunsaver.svg?branch=master)](https://travis-ci.org/TheBiggerGuy/restful-sunsaver)
[![codecov](https://codecov.io/gh/TheBiggerGuy/restful-sunsaver/branch/master/graph/badge.svg)](https://codecov.io/gh/TheBiggerGuy/restful-sunsaver)
[![](https://img.shields.io/github/issues-raw/TheBiggerGuy/restful-sunsaver.svg)](https://github.com/TheBiggerGuy/restful-sunsaver/issues)
[![](https://tokei.rs/b1/github/TheBiggerGuy/restful-sunsaver)](https://github.com/TheBiggerGuy/restful-sunsaver)

HTTP REST interface to the SunSaver MPPT via the serial ModBus

```bash
docker run --rm -it -u ${UID}:$(id -g ${USER}) -v /etc/group:/etc/group:ro -v /etc/passwd:/etc/passwd:ro -v "$(pwd):/build" -w="/build" -e "CARGO_HOME=/build/.cargo" -e "RUST_LOG=restful_sunsaver=debug" --device=/dev/SunSaver --group-add dialout --expose="4000" --publish="0.0.0.0:4000:4000" --env="PORT=4000" thebiggerguy/restful-sunsaver:dev cargo run -- --device=/dev/SunSaver
```
