<div align="center">

  # 🌹 roseate
  <sub>A small and simple but fancy image viewer built with Rust & Typescript that's cross platform.</sub>

</div>

> [!CAUTION]
> This project is HEAVEALY a work in progress, if may crash or harm your system. Github issues are welcome. 🤝

# 🛠️ Installation
As roseate is in heavy development I won't offer packages and binaries yet so you'll need to compile the application from source.

## 🪄 Build from source
### Prerequisites:
- [npm](https://docs.npmjs.com/downloading-and-installing-node-js-and-npm)
- [Tauri CLI](https://crates.io/crates/tauri-cli) ( `cargo install tauri-cli` )
- [Rust](https://www.rust-lang.org/tools/install) (must be **1.60+**)
- [Make](https://www.gnu.org/software/make) (recommended, otherwise you'll need to run commands from the [Makefile](./Makefile) manually)

1. Clone the repo.
```sh
git clone https://github.com/cloudy-org/desktop-app-template
cd desktop-app-template
```
2. Pull the submodules (includes [ctk](https://github.com/cloudy-org/cirrus)).
```sh
git submodule update --init --recursive
```
3. Install npm dependencies.
```sh
make install-deps
```

#### ⚙️ For Development
At this stage for development you would just run ``make run``. If you would like to install it to your system countiue ahead to the [next section](#-install-to-your-system).

4. Run ``make run``.
```sh
make run
```
To run Roseate in development with an image use this make command:
```sh
make run ARGS="./anime_girl.png"
```

#### 🎀 Install into your system. (Linux)

4. Install to your system.
```sh
make install
```
5. Then the `roseate` binray will be aviable in your terminal.
```sh
roseate
```
> **Make sure to add ``~/.cargo/bin`` to path.** You can do so by adding ``export PATH=$PATH:~/.cargo/bin`` to your ``.bashrc`` or an equivalent.

Open an image by passing it's path.
```sh
roseate ./anime_girls.png
```
You might want to also set the binrary at ``~/.cargo/bin/roseate`` as your default image viewer so double clicking on images calls it. Look up how to perform that for your desktop enviornment or OS.
