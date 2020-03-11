# miniquad-mandelbrot
A simple mandelbrot set explorer made with [miniquad](https://github.com/not-fl3/miniquad) to demonstrate it's power.

This project demonstrates how easy it is to code cross-platform applications with [miniquad](https://github.com/not-fl3/miniquad).

[A live WASM demo!](https://ponaskovas.github.io/miniquad-mandelbrot-wasm-demo/)

## Compiling
### Desktop
Just build for the target you need
```sh
cargo build --release --target x86_64-unknown-linux-gnu
cargo build --release --target x86_64-apple-darwin
cargo build --release --target x86_64-pc-windows-gnu
```
### WASM
```sh
cargo build --release --target wasm32-unknown-unknown
```
then copy the resulting `wasm` file into the same directory as [`index.html`](https://github.com/PonasKovas/miniquad-mandelbrot/blob/master/index.html) and serve a static http server.

Using python 3:
```sh
python -m http.server
```
### Android
```sh
docker run --rm -v $(pwd)":/root/src" -w /root/src notfl3/cargo-apk cargo apk build --release
```
The APK file will be in `target/android-artifacts/release/apk/`
