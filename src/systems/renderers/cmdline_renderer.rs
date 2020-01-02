use crate::common::{Error, Mode};
use crate::os::Terminal;
use crate::resources::{CmdLine, ColorPalette, FrameBuffer, State, SymbolPalette, PALETTE_OFFSET};
use specs::System;
use texel_types::{ColorMode, Position2D, SymbolStyle, SymbolStyles};

pub struct CmdLineRenderer;

impl<'a> System<'a> for CmdLineRenderer {
    type SystemData = (
        specs::Write<'a, FrameBuffer>,
        specs::Read<'a, State>,
        specs::Read<'a, CmdLine>,
        specs::Read<'a, ColorPalette>,
        specs::Read<'a, SymbolPalette>,
    );

    fn run(&mut self, (mut out, state, cmdline, color_palette, symbol_palette): Self::SystemData) {
        let ts = Terminal::terminal_size();
        let w = i32::from(ts.0);
        let h = i32::from(ts.1);

        print_selected_colors(&mut out, &state, w, h);

        if let Some(error) = state.error() {
            print_error(&mut out, error, h);
            return;
        }

        let mode = state.mode();

        match mode {
            Mode::Quitting(_) => {}
            Mode::Command => print_cmdline(&mut out, &cmdline, h),
            Mode::Object | Mode::Help(_) => print_mode(&mut out, mode, w, h),
            Mode::Write => print_write(&mut out, &state, h),
            Mode::Edit => print_edit(&mut out, &state, &symbol_palette, h),
            Mode::Color(cm) => print_color_select(&mut out, &state, &color_palette, cm, w, h),
            Mode::SelectSymbol(i) => print_symbol_palette(&mut out, &state, i, w, h),
            Mode::SelectColor(i, cm) => print_color_palette(&mut out, &state, &color_palette, i, cm, h),
        }
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

    out.write_line_default(w - 19, h - 1, saved_symbol);
    out.write_line(w - 18, h - 1, "▞", sc.0, sc.1, SymbolStyles::new());
    out.set_cursor_pos(w - 1, h - 1);
}

fn print_write(out: &mut FrameBuffer, state: &State, h: i32) {
    let white = Terminal::grayscale_u8(23);
    let bold = SymbolStyles::only(SymbolStyle::Bold);
    let text = format!("--{}--", state.mode().to_str());

    out.write_line(0, h - 1, text, texel_types::DEFAULT_BG_U8, white, bold);
    out.set_cursor_pos(state.cursor.x, state.cursor.y);
}

fn print_mode(out: &mut FrameBuffer, mode: Mode, w: i32, h: i32) {
    let white = Terminal::grayscale_u8(23);
    let bold = SymbolStyles::only(SymbolStyle::Bold);
    let text = format!("--{}--", mode.to_str());

    out.write_line(0, h - 1, text, texel_types::DEFAULT_BG_U8, white, bold);
    out.set_cursor_pos(w - 1, h - 1);
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

fn print_symbol_palette(out: &mut FrameBuffer, state: &State, index: usize, w: i32, h: i32) {
    let white = Terminal::grayscale_u8(23);
    let bold = SymbolStyles::only(SymbolStyle::Bold);
    let text = format!("--{}--", state.mode().to_str());
    let i_txt = format!("{}", crate::common::index_from_one(index));

    out.write_line(1, h - 1, text, texel_types::DEFAULT_BG_U8, white, bold);
    out.write_line(
        PALETTE_OFFSET + index as i32,
        h - 1,
        i_txt,
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

    for y in min.y..min.y + PALETTE_H {
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
