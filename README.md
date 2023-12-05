# Space Acres

Space Acres is an opinionated unofficial GUI application for farming on [Subspace Network](https://subspace.network/).

## Current status

Current status of the project is Alpha.

This means that while it should generally work, expect things to not work sometimes, break in unexpected ways and error
handling to be lacking.

Current version supports Gemini 3g chain only and doesn't allow to select anything else.

## Features

Current features:
* Initial configuration
* Node sync with displayed progress
* Farmer plotting/farming with a single farm with displayed plotting/replotting progress

Some of the upcoming features/capabilities (not necessarily in priority order):
* Automatic builds in CI with pre-built executables/installers (Linux and macOS)
* Testing on macOS
* Welcome screen
* Writing logs to a file
* Displaying of earned farming rewards or at least link to block explorers
* Better status reporting of the node and farmer (piece cache sync, etc.)
* Displaying sync/plotting/replotting speed (UI already supports this, but there is no backend code to calculate the speed)
* Support for multiple farms (backend and config file already support this if you edit config manually, but you will not see them in UI)
* Farmer benchmarking support
* Re-configuration screen with old configuration filled instead of starting from scratch

## Project structure

The project at high level is structured in a few large modules:
* `backend` handles all the backend functionality
  * `config` contains configuration data structure with ability to read, write and validate it
  * `farmer` contains farmer implementation with a wrapper data structure that abstracts away its internals
  * `networking` contains networking stack that is shared between `farmer` and `node` with a wrapper data structure that abstracts away its internals
  * `node` contains consensus node with a wrapper data structure that abstracts away its internals
  * `utils` contains some low-level utilities
* `main.rs` contains UI and communication with backend, though UI will move into `frontend` in the future
* `res/app.css` contains a few small non-critical tweaks for presentation, it will likely be necessary to ship a GTK4 theme with the app in the future to ensure consistent look

Application supports bare minimum configuration and doesn't support operator functionality (not yet anyway).

## How to build

In order to build this app you'll need to install both dependencies necessary for building
[Subspace](https://github.com/subspace/subspace) and [GTK4](https://github.com/gtk-rs/gtk4-rs), follow their
documentation for details, otherwise `cargo run` will get you where to want to be.

## Contribution
Contributions of various kinds are welcome and appreciated.

## License
Zero-Clause BSD

https://opensource.org/licenses/0BSD

https://tldrlegal.com/license/bsd-0-clause-license 
