# contribute

this project is open to contributions.

## start

1. clone the repo
2. run `cargo check`
3. run `cargo run`
4. test hotkeys and tray actions on windows

## workflow

1. open an issue first for behavior changes
2. keep changes focused and small
3. run `cargo check` and `cargo test`
4. update docs when behavior changes

## structure

- runtime code: `src/`
- scripts: `scripts/`
- workflows: `.github/workflows/`
- contributor docs: `contribute/`

## release

1. update version in `Cargo.toml`
2. tag `vX.Y.Z`
3. push tag
4. github actions builds and publishes release artifacts
