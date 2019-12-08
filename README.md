# Texel

ASCII Art and landscape editor. Texel aims to make editing ASCII art easy especially
for use in games.

## Building

### Dependencies

Rust v1.38+ is required.

This editor uses [termion](https://gitlab.redox-os.org/redox-os/termion) for handling the terminal. I have not tested it outside of Linux yet.

### Compiling

`cargo build --release`

## Usage

`texel [files]`

## Configuration

Configuration files are saved in user config directory undex `texel/config.ron` using the [RON](https://github.com/ron-rs/ron) format.

On Linux for example the location would be `$HOME/.config/texel/config.ron`

## [Documentation](docs/overview.md)