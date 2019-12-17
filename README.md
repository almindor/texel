# Texel

ASCII Art and landscape editor. Texel aims to make editing ASCII art easy especially for use in games.

### Example Scene

![alt text](https://raw.githubusercontent.com/almindor/texel/master/docs/texel_scene.png)


## Building

### Dependencies

Rust v1.38+ is required.

Platforms supported are Redox, Mac OS X, and Linux due to [Termion](https://docs.rs/termion/1.5.4/termion/). Crossterm support is planned to add Windows support later.

### Compiling

`cargo build --release`

### [Changelog](CHANGELOG.md)

## Usage

`texel [files]`

### [Documentation](docs/overview.md)

### Configuration

Configuration files are saved in user config directory undex `texel/config.ron` using the [RON](https://github.com/ron-rs/ron) format.

On Linux for example the location would be `$HOME/.config/texel/config.ron`

### [File Format](https://github.com/almindor/texel_types)

Types are defined in the [texel_types](https://github.com/almindor/texel_types) crate. When saving a scene a gzipped RON file is produced. The contents are the Scene object with all the Sprites.