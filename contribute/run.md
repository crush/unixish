# run

## dev

```bash
cargo check
cargo test
cargo run
```

## local testing

```bash
# build once
cargo build --release

# run app directly (no cargo wrapper)
.\target\release\unixish.exe

# optional startup for daily testing
.\target\release\unixish.exe startup on
```

## cli

```bash
unixish check
unixish path
unixish startup status
unixish startup on
unixish startup off
unixish reset
unixish update
```

## tray

- Pause
- Config
- Reload
- Startup On or Off
- Reset
- Update
- Quit
