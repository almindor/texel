# Texel

ASCII Art and landscape editor. Texel aims to make editing ASCII art easy especially for use in games.

### Example Scene

![Example Scene PNG](https://raw.githubusercontent.com/almindor/texel/master/docs/texel_scene.png)


## Building

### Dependencies

Rust v1.38+ is required.

The default terminal backend used is [Crossterm](https://crates.io/crates/crossterm) which supports Linux, Mac OS X and Windows.


[Termion](https://docs.rs/termion/1.5.4/termion/) can be used as well by switching to the `ion` feature.

### Compiling

#### Linux, Windows*, Mac OS X

`cargo build --release`

- NOTE: Windows is currently broken due to crossterm binding issues

#### Redox/Termion

`cargo build --release --no-default-features --features ion`

### [Changelog](CHANGELOG.md)

## Usage

`texel [files]`

### [Documentation](docs/overview.md)

### Configuration

Configuration files are saved in user config directory undex `texel/config.ron` using the [RON](https://github.com/ron-rs/ron) format.

On Linux for example the location would be `$HOME/.config/texel/config.ron`

### [File Format](https://github.com/almindor/texel_types)

Types are defined in the [texel_types](https://github.com/almindor/texel_types) crate. When saving a scene a gzipped RON file is produced. The contents are the Scene object with all the Sprites.