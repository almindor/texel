use crate::resources::ColorPalette;
use big_enum_set::BigEnumSet;

pub use texel_types::{SymbolStyle, SymbolStyles, Texel, Texels};

// extra stuff useful only in texel itself
pub trait TexelExt {
    fn to_string(&self) -> String;
}

impl TexelExt for Texel {
    fn to_string(&self) -> String {
        format!(
            "{}{}{}{}{}{}",
            crate::common::goto(self.pos.x, self.pos.y),
            ColorPalette::u8_to_bg(self.bg),
            ColorPalette::u8_to_fg(self.fg),
            styles_to_str(self.styles),
            self.symbol,
            termion::style::Reset,
        )
    }
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
