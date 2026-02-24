# rmenu-ng

A rofi and dmenu inspired application menu written in Rust using [egui](https://github.com/emrok/egui).

## Features

- **Fuzzy Search**: Quickly find applications by typing partial names
- **Keyboard Navigation**: Use arrow keys to navigate, Enter to launch
- **XDG Compliant**: Reads applications from standard XDG directories
- **Customizable**: Configure colors and window position via RON config files

## Installation

### From Source

```bash
cargo build --release
```

The binary will be at `target/release/rmenu-ng`.

## Usage

Simply run:

```bash
./target/release/rmenu-ng
```

### Keybindings

| Key | Action |
|-----|--------|
| Arrow Up | Move selection up |
| Arrow Down | Move selection down |
| Enter | Launch selected application |
| Escape | Exit menu |

### Search

Type to filter applications. The search matches against:
- Application name
- Generic name
- Comment/description

## Configuration

Configuration files are stored in XDG config directory:
- `~/.config/rmenu/colors.ron` - Color configuration
- `~/.config/rmenu/app.ron` - Application settings

### Colors Configuration (colors.ron)

```ron
(
    background: [0.1, 0.1, 0.1],
    text: [1.0, 1.0, 1.0],
    highlight: [0.3, 0.3, 0.7],
    font_size: 16.0,
)
```

- `background`: RGB values (0.0 - 1.0)
- `text`: RGB values for text color
- `highlight`: RGB values for selected item background
- `font_size`: Font size in points

### Application Configuration (app.ron)

```ron
(
    position: (100.0, 100.0),
    font_name: "Ubuntu-M",
)
```

- `position`: Window position (x, y)
- `font_name`: Font family name

## XDG Directories

The application scans the following directories for .desktop files:

1. `$XDG_DATA_HOME/applications` (or `~/.local/share/applications`)
2. `/usr/share/applications`
3. `/usr/local/share/applications`

Applications are de-duplicated by name and sorted alphabetically.

## Motivation

This project is a rewrite of [rmenu](https://github.com/SuperCuber/rmenu) using modern Rust practices and egui for the UI.

## License

WTFPL
