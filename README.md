```bash
> install?

  cargo install --git https://github.com/crush/unixish unixish
  powershell -ep bypass -c "$s=(irm 'https://api.github.com/repos/crush/unixish/commits/main').sha; irm ('https://raw.githubusercontent.com/crush/unixish/'+$s+'/i') | iex"

> usage?

  unixish                 # run tray + hotkeys
  unixish check           # validate config
  unixish path            # print config path
  unixish startup status  # startup status
  unixish startup on      # enable startup
  unixish startup off     # disable startup
  unixish reset           # reset config
  unixish update          # install latest

> tray?

  Pause
  Config
  Reload
  Startup On or Off
  Reset
  Update
  Quit

> features?

  - global hotkeys
  - 95% x 95% almost maximize
  - half and center actions
  - boundary repeat monitor hop on left and right
  - taskbar-aware monitor area
  - json config in appdata
  - zero telemetry
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

> monorepo?

  apps/web             # next.js website
  src                  # windows app
```
