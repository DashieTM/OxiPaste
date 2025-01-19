# OxiPaste
A small clipboard manager written in Iced/Rust(GTK before).
Requires wl-clipboard to be installed in order to run.

## Configuration
The config file is placed in xdg-config-home/oxipaste/config.toml.
The following values are supported:
```toml
keepOpen = false
PlainTextContextActions = [['notify-send']]
AddressContextActions = [['xdg-open'], ['notify-send']]
ImageContextActions = [['sh', '-c', 'wl-paste | satty -f -']]
```

## Screenshot
![Screenshot of Main Application](./screenshots/home.png?raw=true)
![Screenshot of Context](./screenshots/context.png?raw=true)

## License Notices
All icons used are Material icons made publicly available by Google.
