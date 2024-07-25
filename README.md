# Space Acres

[![Latest Release](https://img.shields.io/github/v/release/autonomys/space-acres?display_name=tag&style=flat-square)](https://github.com/autonomys/space-acres/releases)
[![Downloads Latest](https://img.shields.io/github/downloads/autonomys/space-acres/latest/total?style=flat-square)](https://github.com/autonomys/space-acres/releases/latest)
[![Rust](https://img.shields.io/github/actions/workflow/status/autonomys/space-acres/rust.yml?branch=main)](https://github.com/autonomys/space-acres/actions/workflows/rust.yaml)

Space Acres is an opinionated GUI application for farming on [Autonomys Network](https://www.autonomys.xyz/).

## Current status

Current status of the project is Alpha.

This means that while it should generally work, expect things to not work sometimes, break in unexpected ways and error
handling to be lacking.

Current version supports Gemini 3h chain only and doesn't allow to select anything else. It supports upgrading existing
installations from 3g.

## Features

Current features:

* Configuration (reward address, node location, multiple farms, P2P ports)
* Node sync with displayed progress, speed and ETA
* Farmer plotting/farming piece cache/plotting/replotting progress display and speed calculation
* Farmer auditing/proving performance indicators
* Farmer sector state visualization

Upcoming features/capabilities: see open issues, also consider contributing if something is missing!

## Installation

See [docs/INSTALLATION.md](docs/INSTALLATION.md) for details

## Project structure

The project at high level is structured in a few large modules:

* `backend` handles all the backend functionality
    * `config` contains configuration data structure with ability to read, write and validate it
    * `farmer` contains farmer implementation with a wrapper data structure that abstracts away its internals
    * `networking` contains networking stack that is shared between `farmer` and `node` with a wrapper data structure
      that abstracts away its internals
    * `node` contains consensus node with a wrapper data structure that abstracts away its internals
    * `utils` contains some low-level utilities
* `docs` contains documentation files
* `frontend` handles majority of frontend logic with each module corresponding to a major application screen/view
* `res` contains various non-code resources required for application operation and/or packaging
    * `app.css` contains a few small non-critical tweaks for presentation, it will likely be necessary to ship a GTK4
      theme with the app in the future to ensure consistent look
    * `linux` contains Linux-specific resources
    * `windows` contains Windows-specific resources
* `main.rs` handles high-level UI and communication with backend, wiring everything together

Application supports bare minimum configuration and doesn't support operator functionality (not yet anyway).

## How to build

In order to build this app you'll need to install both dependencies necessary for building
[Subspace](https://github.com/autonomys/subspace) and [GTK4](https://github.com/gtk-rs/gtk4-rs), follow their
documentation for details, otherwise `cargo run` will get you where to want to be.

## Contribution

Contributions of various kinds are welcome and appreciated.

## License

Zero-Clause BSD

https://opensource.org/licenses/0BSD

https://tldrlegal.com/license/bsd-0-clause-license 
