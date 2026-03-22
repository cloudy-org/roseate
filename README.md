<div align="center">

  # 🌹 roseate
  <sub>A fancy, fast, GPU-accelerated image viewer built for simplicity and all the control in the world. **VERY WIP!**</sub>

  <img width="700px" src="./assets/alpha_preview_1.png">

  <sub>(image from alpha build, this is **NOT** the final product)</sub>

</div>

> [!CAUTION]
> This project is HEAVILY a **work in progress**, it could crash or freeze up your system. Github issues are very welcome. 🤝
> 
> Additionally, you are playing with alpha versions of Roseate. Many little features I intend to be available on a stable release, either don't exist yet or are unfinished. PRs for unassigned issues or issues labelled with "good first issue" are welcome. 🤝

# 📖 Wiki
The wiki will provide more in-depth info moving forward: https://cloudy-org.github.io/wiki/apps/roseate/

# 🌟 Devlogs
<a href="https://www.youtube.com/watch?v=8bSdw34x98k&list=PLI8mjQYZec82ZajoDnDkAicBarEZ1yPd-&index=2">
  <img width="400px" src="https://img.youtube.com/vi/8bSdw34x98k/maxresdefault.jpg">
</a>

You can find devlogs **in video format** on my youtube channel.

I have a playlist for it here: **https://www.youtube.com/playlist?list=PLI8mjQYZec82ZajoDnDkAicBarEZ1yPd-**

# 🛠️ Installation
Roseate is in heavy development so you won't see many packages and binaries offered, you'll mostly need to compile the application from source.

> [!warning]
> Roseate is in **ALPHA**, expect bugs during installation.

## 🪟 Windows
Head over to [github releases](https://github.com/cloudy-org/roseate/releases) and grab a windows installer from the latest alpha release in assets.

<img width="240px" src="./assets/window_setup_in_assets.png">

## 🐧 Linux
First check your linux distribution for available packages.

If there isn't any, either grab the binary from the releases or compile from source.

[![packaging_status](https://repology.org/badge/vertical-allrepos/roseate.svg)](https://repology.org/project/roseate/versions)

### Arch Linux
I officially maintain both **[`roseate`](https://aur.archlinux.org/packages/roseate)** and **[`roseate-bin`](https://aur.archlinux.org/packages/roseate-bin)** on the **Arch Linux** AUR:

```sh
yay -S roseate-bin
```

## 🏗️ Build from source

### Prerequisites:
- **[Rust](https://www.rust-lang.org/tools/install)** and **Cargo** (Rust **`1.89.0`**+ is required!).
- **Linux** (dependencies required by **[eframe](https://crates.io/crates/eframe)**, you most likely already have all of these installed)
  - **[libxcb](https://archlinux.org/packages/extra/x86_64/libxcb/)**
  - **[openssl](https://archlinux.org/packages/core/x86_64/openssl/)**
  - **[libxkbcommon](https://archlinux.org/packages/extra/x86_64/libxkbcommon/)**
  - **[xdg-desktop-portal](https://github.com/flatpak/xdg-desktop-portal)**

Ignore all deps under "Linux" if you're not on **Linux**.

1. Clone the repository and pull git submodules.
```sh
git clone https://github.com/cloudy-org/roseate
cd roseate

git submodule update --init --recursive
```

2. Build the release binary.
```sh
cargo build --release
```

3. The binary is located at `./target/release`.

If you're on **Linux** there is a handy `make install` command to install the compiled binary, the **`.desktop` file** and **icons** into your system as well as `make uninstall`:

```sh
sudo make install
```

# Development
> [!NOTE]
> Building a development build WILL SIGNIFICANTLY KILL performance!
> Read more [here](https://github.com/cloudy-org/roseate/blob/6e7e638997110af0149f06ceadb87c3ec088cf84/Cargo.toml#L48-L53).

For development, you would just run ``cargo run``.

```sh
cargo run
```

To run Roseate in development with an image, append `--` and pass an image path after like so:

```sh
cargo run -- ./anime_girl.png
```

To run with verbose debugging, call cargo run with the `RUST_LOG=DEBUG` environment variable:

```sh
RUST_LOG=DEBUG cargo run -- ./anime_girl.png
```
```
[2024-10-20T02:20:36Z DEBUG roseate] Image '/home/goldy/Downloads/anime_girl.png' loading from path...
[2024-10-20T02:20:36Z DEBUG eframe] Using the glow renderer
[2024-10-20T02:20:36Z DEBUG sctk] Bound new global [70] wl_output v4
[2024-10-20T02:20:36Z DEBUG sctk] Bound new global [74] wl_output v4
[2024-10-20T02:20:36Z DEBUG sctk] Bound new global [30] zxdg_output_manager_v1 v3
[2024-10-20T02:20:36Z DEBUG sctk] Bound new global [10] wl_seat v7
[2024-10-20T02:20:36Z DEBUG sctk] Bound new global [16] wp_cursor_shape_manager_v1 v1

... (truncated for the sanity of this readme)
```

<br>

<div align="center">

  <img width="650px" src="./assets/gif_showcase_1.gif">

</div>