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
