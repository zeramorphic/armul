# ARMUL: ARM7 emulator

ARMUL is an ARM7TDMI emulator and graphical debugger.
The program was inspired by the [Komodo](https://studentnet.cs.manchester.ac.uk/resources/software/komodo/) software developed by Charlie Brej and Jim Garside.
Please send any issues or bug reports to the [issue tracker](https://github.com/zeramorphic/armul/issues).

## Setup

- Install [`pnpm`](https://pnpm.io/installation).
- [Use pnpm to install node](https://pnpm.io/cli/env) if not already installed.
- Install [Rust](https://rust-lang.org/tools/install/).
- Run `pnpm install` to get JavaScript dependencies.
- Install the Tauri prerequisites [here](https://tauri.app/start/prerequisites/).

Now:
- To run the project in debug mode, run `pnpm tauri dev`.
- To compile a release build, run `pnpm tauri build`.
- To run all processor tests, run `cargo test -p armul`.

## Development information

- The UI is created using [React](https://react.dev/). The main UI library is [shadcn](https://ui.shadcn.com/).
