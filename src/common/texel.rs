use crate::resources::ColorPalette;
use big_enum_set::BigEnumSet;

pub use texel_types::{SymbolStyle, SymbolStyles, Texel, Texels};

pub fn texel_to_string(texel: &Texel) -> String {
    format!(
        "{}{}{}{}{}{}",
        crate::common::goto(texel.pos.x, texel.pos.y),
        ColorPalette::u8_to_bg(texel.bg),
        ColorPalette::u8_to_fg(texel.fg),
        styles_to_str(texel.styles),
        texel.symbol,
        termion::style::Reset,
    )
}

fn styles_to_str(styles: BigEnumSet<SymbolStyle>) -> String {
    use termion::style::{Bold, Italic, Underline};
    let mut result = String::with_capacity(64);

    for style in styles.iter() {
        result += match style {
            SymbolStyle::Bold => Bold.as_ref(),
            SymbolStyle::Italic => Italic.as_ref(),
            SymbolStyle::Underline => Underline.as_ref(),
        }
    }

    result
}
