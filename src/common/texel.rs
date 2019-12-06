use crate::resources::ColorPalette;
use big_enum_set::BigEnumSet;

pub use crate::texel_types::{TexelV1, SymbolStyle, SymbolStyles};

pub type Texel = TexelV1; // alias so we can switch to another version easily
pub type Texels = Vec<TexelV1>;

pub fn texel_to_string(texel: &Texel) -> String {
    format!(
        "{}{}{}{}{}{}",
        crate::common::goto(texel.x, texel.y),
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
