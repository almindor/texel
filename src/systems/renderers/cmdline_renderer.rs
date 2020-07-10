use crate::common::{Error, Mode, SelectMode, SELECTED_INFO_TEMPLATE};
use crate::os::Terminal;
use crate::resources::{CmdLine, ColorPalette, FrameBuffer, State, SymbolPalette, PALETTE_OFFSET};
use legion::prelude::*;
use texel_types::{ColorMode, Position2D, SymbolStyle, SymbolStyles};

// type SystemData = (
//     specs::Write<'a, FrameBuffer>,
//     specs::Read<'a, State>,
//     specs::Read<'a, CmdLine>,
//     specs::Read<'a, ColorPalette>,
//     specs::Read<'a, SymbolPalette>,
// );

pub fn render_cmdline(world: &mut World, state: &State, out: &mut FrameBuffer) {
    let ts = Terminal::terminal_size();
    let w = i32::from(ts.0);
    let h = i32::from(ts.1);

    print_selected_colors(out, state, w, h);

    if let Some(error) = state.error() {
        print_error(out, error, h);
        return;
    }

    let mode = state.mode();
    let cmdline = world.resources.get::<CmdLine>().unwrap();
    let symbol_palette = world.resources.get::<SymbolPalette>().unwrap();
    let color_palette = world.resources.get::<ColorPalette>().unwrap();

    match mode {
        Mode::Quitting(_) => {}
        Mode::Command => print_cmdline(out, &cmdline, h),
        Mode::Object(_) | Mode::Help(_) => print_mode(out, state, mode, w, h),
        Mode::Write => print_write(out, state, h),
        Mode::Edit => print_edit(out, state, &symbol_palette, h),
        Mode::Color(cm) => print_color_select(out, state, &color_palette, cm, w, h),
        Mode::SelectSymbol(i) => print_symbol_palette(out, state, &symbol_palette, i, w, h),
        Mode::SelectColor(i, cm) => print_color_palette(out, state, &color_palette, i, cm, h),
    }
}

fn print_error(out: &mut FrameBuffer, error: &Error, h: i32) {
    let red = Terminal::rgb_u8(5, 0, 0);
    let white = Terminal::rgb_u8(5, 5, 5);
    let bold = SymbolStyles::only(SymbolStyle::Bold);

    out.write_line(1, h - 1, error, red, white, bold);
}

fn print_cmdline(out: &mut FrameBuffer, cmdline: &CmdLine, h: i32) {
    let cmd_text = format!(":{}", cmdline.cmd());

    out.write_line_default(0, h - 1, cmd_text);
    out.set_cursor_pos(1 + cmdline.cursor_pos() as i32, h); // account for :
}

fn print_selected_colors(out: &mut FrameBuffer, state: &State, w: i32, h: i32) {
    // color selection
    let sc = (state.color(ColorMode::Bg), state.color(ColorMode::Fg));
    let saved_symbol = if state.unsaved_changes() { "*" } else { " " };
    let len = SELECTED_INFO_TEMPLATE.len() as i32;

    out.write_line_default(w - len - 1, h - 1, saved_symbol);
    out.write_line(w - len, h - 1, "▞", sc.0, sc.1, SymbolStyles::new());
    out.set_cursor_pos(w - 1, h - 1);
}

fn print_write(out: &mut FrameBuffer, state: &State, h: i32) {
    let white = Terminal::grayscale_u8(23);
    let bold = SymbolStyles::only(SymbolStyle::Bold);
    let text = format!("--{}--", state.mode().to_str());

    out.write_line(0, h - 1, text, texel_types::DEFAULT_BG_U8, white, bold);
    out.set_cursor_pos(state.cursor.x, state.cursor.y);
}

fn print_mode(out: &mut FrameBuffer, state: &State, mode: Mode, w: i32, h: i32) {
    let white = Terminal::grayscale_u8(23);
    let bold = SymbolStyles::only(SymbolStyle::Bold);
    let text = format!("--{}--", mode.to_str());

    out.write_line(0, h - 1, text, texel_types::DEFAULT_BG_U8, white, bold);
    if mode == Mode::Object(SelectMode::Region) {
        out.set_cursor_pos(state.cursor.x, state.cursor.y);
    } else {
        out.set_cursor_pos(w - 1, h - 1);
    }
}

fn print_edit(out: &mut FrameBuffer, state: &State, palette: &SymbolPalette, h: i32) {
    let white = Terminal::grayscale_u8(23);
    let bold = SymbolStyles::only(SymbolStyle::Bold);

    out.write_line(0, h - 1, "--EDIT--", texel_types::DEFAULT_BG_U8, white, bold);
    out.write_texels(palette.line_texels(PALETTE_OFFSET, h - 1));
    out.set_cursor_pos(state.cursor.x, state.cursor.y);
}

fn print_color_select(out: &mut FrameBuffer, state: &State, palette: &ColorPalette, cm: ColorMode, w: i32, h: i32) {
    let white = Terminal::grayscale_u8(23);
    let bold = SymbolStyles::only(SymbolStyle::Bold);
    let text = format!("--{}--", state.mode().to_str());

    out.write_line(0, h - 1, text, texel_types::DEFAULT_BG_U8, white, bold);
    out.write_texels(palette.line_texels(PALETTE_OFFSET, h - 1, cm));
    out.set_cursor_pos(w - 1, h - 1);
}

fn print_symbol_palette(out: &mut FrameBuffer, state: &State, palette: &SymbolPalette, index: usize, w: i32, h: i32) {
    let white = Terminal::grayscale_u8(23);
    let bold = SymbolStyles::only(SymbolStyle::Bold);
    let text = format!("--{}--", state.mode().to_str());

    out.write_line(1, h - 1, text, texel_types::DEFAULT_BG_U8, white, bold);
    out.write_line(
        PALETTE_OFFSET + (index * 2) as i32,
        h - 1,
        palette.symbol(index),
        texel_types::DEFAULT_BG_U8,
        white,
        bold,
    );
    out.set_cursor_pos(w, h - 1);
}

fn print_color_palette(
    out: &mut FrameBuffer,
    state: &State,
    palette: &ColorPalette,
    index: usize,
    cm: ColorMode,
    h: i32,
) {
    use crate::resources::{MAX_COLOR_INDEX, PALETTE_H, PALETTE_W};

    let mut count = 0;
    let white = Terminal::grayscale_u8(23);
    let bold = SymbolStyles::only(SymbolStyle::Bold);
    let text = format!("--{}--", state.mode().to_str());
    let min = Position2D {
        x: PALETTE_OFFSET,
        y: h - PALETTE_H,
    };

    for y in min.y..min.y + PALETTE_H - 1 {
        for x in min.x..min.x + PALETTE_W {
            if count >= MAX_COLOR_INDEX {
                break;
            }

            let (r, g, b) = ColorPalette::base_to_rgb(count);
            count += 1;

            out.write_line(
                x,
                y,
                " ",
                Terminal::rgb_u8(r, g, b),
                texel_types::DEFAULT_FG_U8,
                SymbolStyles::new(),
            );
        }
    }

    let x = PALETTE_OFFSET + (index as i32);
    let pos = Position2D { x, y: h - 1 };

    out.write_line(1, h - 1, text, texel_types::DEFAULT_BG_U8, white, bold);
    out.write_texel(palette.selector_texel(index, pos, cm));
    out.set_cursor_pos(state.cursor.x, state.cursor.y);
}
