
### All Modes
* `ESC`          - cancel/return
* `ENTER/RETURN` - confirm/select
* `ARROW KEYS`   - movement
* `h, j, k, l`   - movement
* `:`            - command mode
* `u, U`         - undo/redo

### Object Mode
* `DELETE`       - delete selected
* `TAB`          - select next
* `SHIT + TAB`   - add next to selection
* `H, J, K, L`   - movement to edge
* `n`            - add empty object
* `e`            - edit selected (new if none, switch to `Edit` mode)
* `i`            - write to selected directly (new if none, switch to `Write` mode)
* `z, x`         - apply fg/bg color to all texels in selected
* `Z, X`         - set fg/bg color from palette (switch to `ColorPalette` mode)

### Edit Mode

* `i`            - write to edited directly (switch to `Write` mode)
* `0..9, a..f`   - apply symbol from palette (think hex index)
* `!..), A..F`   - set symbol on palette (think hex index)
* `z, x`         - apply fg/bg color to selected texel
* `Z, X`         - set fg/bg color from palette (switch to `ColorPalette` mode)
* `q, Q, w`      - apply/clear symbol style (bold, italic, underline)

### ColorPalette | SymbolPalette Mode

* `0..9, a..f`   - apply color/symbol from palette (think hex index)
* `!..), A..F`   - set color/symbol on palette (think hex index)
