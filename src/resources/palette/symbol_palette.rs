use crate::common::{Error, Texel};
use crate::resources::ColorPalette;
use big_enum_set::BigEnumSet;
use serde::{Deserialize, Serialize};

const SYMBOLS_IN_PALETTE: usize = 16;
const DEFAULT_SYMBOLS: [char; SYMBOLS_IN_PALETTE] = [
    '|', '-', '_', '=', '\\', '/', '[', ']', '~', 'O', '*', '^', '#', '@', '!', '?',
];
const SYMBOL_SELECTOR: [char; SYMBOLS_IN_PALETTE] = [
    '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', 'a', 'b', 'c', 'd', 'e', 'f',
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolPalette {
    symbols: [char; SYMBOLS_IN_PALETTE],
}

impl Default for SymbolPalette {
    fn default() -> Self {
        SymbolPalette {
            symbols: DEFAULT_SYMBOLS,
        }
    }
}

impl From<[char; SYMBOLS_IN_PALETTE]> for SymbolPalette {
    fn from(symbols: [char; SYMBOLS_IN_PALETTE]) -> Self {
        SymbolPalette { symbols }
    }
}

impl From<&[char]> for SymbolPalette {
    fn from(symbols: &[char]) -> Self {
        let mut symbol_chars: [char; SYMBOLS_IN_PALETTE] = [' '; SYMBOLS_IN_PALETTE];

        for (i, symbol) in symbols.iter().enumerate() {
            symbol_chars[i] = *symbol;
        }

        SymbolPalette { symbols: symbol_chars }
    }
}

impl SymbolPalette {
    pub fn symbol(&self, index: usize) -> char {
        // index here is natural digit conversion, but we go 1,2...9,0,a,b,c...f
        let mut i = index;
        if index == 0 {
            i = 9;
        } else if index < 10 {
            i = index - 1;
        }

        self.symbols[i]
    }

    pub fn set_symbol(&mut self, index: usize, symbol: char) -> Result<(), Error> {
        if index >= self.symbols.len() {
            return Err(Error::execution("Symbol index out of bounds"));
        }

        self.symbols[index] = symbol;

        Ok(())
    }

    pub fn line_texels(&self, start_x: i32, y: i32) -> Vec<Texel> {
        let mut result = Vec::with_capacity(SYMBOLS_IN_PALETTE * 4);

        let mut x = start_x;
        for (i, symbol) in DEFAULT_SYMBOLS.iter().enumerate() {
            let selector = SYMBOL_SELECTOR[i];
            result.push(Texel {
                x,
                y,
                symbol: selector,
                bg: ColorPalette::default_bg_u8(),
                fg: ColorPalette::default_fg_u8(),
                styles: BigEnumSet::new(),
            });
            x += 1;

            result.push(Texel {
                x,
                y,
                symbol: ':',
                bg: ColorPalette::default_bg_u8(),
                fg: ColorPalette::default_fg_u8(),
                styles: BigEnumSet::new(),
            });
            x += 1;

            result.push(Texel {
                x,
                y,
                symbol: *symbol,
                bg: ColorPalette::default_bg_u8(),
                fg: ColorPalette::default_fg_u8(),
                styles: BigEnumSet::new(),
            });
            x += 1;
        }

        result
    }
}
