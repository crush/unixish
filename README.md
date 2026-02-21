```bash
> install?

  cargo install --git https://github.com/crush/unixish unixish
  powershell -ExecutionPolicy Bypass -Command "irm https://raw.githubusercontent.com/crush/unixish/main/scripts/install.ps1 | iex"

> usage?

  unixish                 # run tray + hotkeys
  unixish check           # validate config
  unixish path            # print config path
  unixish startup status  # startup status
  unixish startup on      # enable startup
  unixish startup off     # disable startup
  unixish reset           # reset config
  unixish log             # print log path

> tray?

  Pause
  Config
  Reload
  Startup On or Off
  Reset
  Log
  Quit

> features?

  - global hotkeys
  - 95% x 95% almost maximize
  - half and center actions
  - boundary repeat monitor hop on left and right
  - taskbar-aware monitor area
  - json config in appdata
  - startup and tray controls

> keys?

  ctrl+shift+c     # almost
  ctrl+shift+x     # center
  shift+left       # left
  shift+right      # right
  shift+up         # top
  shift+down       # bottom
  ctrl+shift+right # next monitor
  ctrl+shift+left  # prev monitor

> config?

  %appdata%\\unixish\\config.json

> contribute?

  contribute/readme.md

> stack?

  rust ? windows-rs
```
