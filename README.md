# Texel

ASCII Art and landscape editor. Texel aims to make editing ASCII art easy especially for use in games.

## Building

### Dependencies

Rust v1.38+ is required.

### Compiling

`cargo build --release`

## Usage

`texel [files]`

### [Documentation](docs/overview.md)

### Configuration

Configuration files are saved in user config directory undex `texel/config.ron` using the [RON](https://github.com/ron-rs/ron) format.

On Linux for example the location would be `$HOME/.config/texel/config.ron`

### [File Format](https://github.com/almindor/texel_types)

Types are defined in the [texel_types](https://github.com/almindor/texel_types) crate. When saving a scene a gzipped RON file is produced. The contents are the Scene object with all the Sprites.