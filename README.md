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

## Modes

Texel uses modes similar to `vim` but more complex.

* `Object` - used to compose the scene moving, adding, deleting objects/sprites
* `Command` - command mode, entered via `:` similar to vim
* `Edit` - single object/sprite edit mode to change texels (text pixels) via a palette of symbols
* `Write` - immediate write mode, similar to edit more for single object/sprite, all input is direct
* `ColorPalette` - mode in which the color palette is displayed for changing the foreground/background color
* `SymbolPalette` - mode in which the symbols for edit more palette are selected

The default mode is `Object`. Each mode can be reversed to previous one via `ESC`.
The `Command` mode can be entered from any mode except `Write` (so you can type `:` directly).

### Commands in Command mode
* `q, quit`               - quit
* `q!, quit!`             - force quit (don't save)
* `r, read [filename]`    - read a file
* `w, write [filename]`   - save a file
* `import <filename>`     - import object/sprite from text file
* `export <filename>`     - export selected object/sprite to text file
* `delete`                - delete selected
* `deselect`              - deselect all

### [Keymap](docs/keymap.md)
