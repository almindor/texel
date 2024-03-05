use crate::os::Terminal;
use big_enum_set::BigEnumSet;

pub use texel_types::{SymbolStyle, Texel};

// extra stuff useful only in texel itself
pub trait TexelExt {
    fn to_string(&self) -> String;
}

impl TexelExt for Texel {
    fn to_string(&self) -> String {
        format!(
            "{}{}{}{}{}{}",
            Terminal::goto(self.pos.x, self.pos.y),
            Terminal::bg_color_sequence(self.bg),
            Terminal::fg_color_sequence(self.fg),
            styles_to_str(self.styles),
            self.symbol,
            Terminal::reset_sequence(),
        )
    }
}

fn styles_to_str(styles: BigEnumSet<SymbolStyle>) -> String {
    let mut result = String::with_capacity(64);

    for style in styles.iter() {
        let sequence = Terminal::style_sequence(style);

        result += &format!("{}{}", result, sequence);
    }

    result
}
