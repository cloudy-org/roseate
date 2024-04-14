<div align="center">

  # 🌹 roseate
  <sub>A small and simple but fancy image viewer built with Rust & Typescript that's cross-platform.</sub>

</div>

> [!CAUTION]
> This project is HEAVEALY a work in progress, hence it may crash or harm your system. Github issues are welcome. 🤝

# 🛠️ Installation
As Roseate is in heavy development I won't offer packages and binaries yet so you'll need to compile the application from source.

## 🏗 Build from source
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
At this stage, for development, you would just run ``make run``. If you would like to install it to your system continue ahead to the [next section](#-install-to-your-system).

4. Run ``make run``.
```sh
make run
```
To run Roseate in development with an image use this make command:
```sh
make run ARGS="./anime_girl.png"
```

#### 🎀 Install into your system.

4. Install to your system.
```sh
make install
```
5. Then the `roseate` binary will be available in your terminal.
```sh
roseate
```
> **Make sure to add ``~/.cargo/bin`` to path.** You can do so by adding ``export PATH=$PATH:~/.cargo/bin`` to your ``.bashrc`` or an equivalent. On Windows, it should [already be in path](https://users.rust-lang.org/t/cargo-bin-path-issue-on-windows-absolute-beginner/38932/2).

Open an image by passing its path.
```sh
roseate ./anime_girls.png
```
You might want to also set the binary at ``~/.cargo/bin/roseate`` as your default image viewer so double clicking on images calls it. Look up how to perform that for your desktop environment or OS.
