<div align="center">

  # 🌹 roseate
  <sub>A small and simple but fancy image viewer built with Rust & Typescript that's cross platform.</sub>

</div>

> [!CAUTION]
> This project is HEAVEALY a work in progress, if may crash or harm your system. Github issues are welcome. 🤝

# 🛠️ How to install for development.

### Prerequisites:
- [npm](https://docs.npmjs.com/downloading-and-installing-node-js-and-npm)
- [Tauri CLI](https://crates.io/crates/tauri-cli)
- [Rust](https://www.rust-lang.org/tools/install) (must be **1.60+**)
- [Make](https://www.gnu.org/software/make) (recommended, otherwise you'll need to run commands from the [Makefile](./Makefile) manually)

1. Clone the repo.
```sh
git clone https://github.com/cloudy-org/desktop-app-template
```
2. Pull the submodules (includes [ctk](https://github.com/cloudy-org/cirrus)).
```sh
git submodule update --init --recursive
```
3. Install npm dependencies.
```sh
make install-deps
```
4. Run ``make run``.
```sh
make run
```
To run Roseate in development with an image use this make command:
```sh
make run ARGS="./anime_girl.png"
```

<br>

> TODO: README
