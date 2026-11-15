# Pixeltown

Pixeltown is a retro-looking pixel-art city-builder video game inspired
in the original SimCity.

## Requirements

- [Rust](https://rustup.rs) (stable, 2021 edition)
- On Linux, macroquad needs the system graphics and audio libraries, e.g. on Debian/Ubuntu:
  `sudo apt install libx11-dev libxi-dev libgl1-mesa-dev libasound2-dev`

## Install

```sh
git clone git@github.com:tmaiadev/pixeltown.git
cd pixeltown
cargo fetch
```

## Build

```sh
cargo build            # debug build: target/debug/pixeltown
cargo build --release  # optimized build: target/release/pixeltown
```

## Run

```sh
cargo run
cargo run --release
```

Flags go after `--` when using cargo:

```sh
cargo run -- --windowed --show-fps
cargo run --release -- --scene gameplay
```

Or run the built binary directly:

```sh
./target/release/pixeltown --windowed
```

| Flag | Description |
| --- | --- |
| `--windowed` | Run in a window instead of fullscreen |
| `--show-fps` | Show the FPS counter |
| `--scene <name>` | Skip straight to a scene: `intro`, `menu`, `load-game`, `terrain`, `gameplay` |

`--scene gameplay` resumes the latest save, or starts a fresh map if there is none.

## Test

```sh
cargo test
```

## Save files

Saves are written to a `userdata/` folder next to the executable
(e.g. `target/debug/userdata/` when running through cargo).

---

Copyright (C) 2026  Thalles Maia

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
