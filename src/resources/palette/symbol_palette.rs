use crate::common::Error;
use serde::{Deserialize, Serialize};
use texel_types::{Position2D, SymbolStyles, Texel, Texels};

const SYMBOLS_IN_PALETTE: usize = 16;
const DEFAULT_SYMBOLS: [char; SYMBOLS_IN_PALETTE] = [
    '|', '-', '_', '=', '\\', '/', '[', ']', '~', 'O', '*', '^', '#', '@', '!', '?',
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
        self.symbols[index]
    }

    pub fn set_symbol(&mut self, index: usize, symbol: char) -> Result<(), Error> {
        if index >= self.symbols.len() {
            return Err(Error::execution("Symbol index out of bounds"));
        }

        self.symbols[index] = symbol;

        Ok(())
    }

    pub fn line_texels(&self, start_x: i32, y: i32) -> Texels {
        let mut result = Vec::with_capacity(SYMBOLS_IN_PALETTE * 4);

        let mut x = start_x;
        for symbol in self.symbols.iter() {
            result.push(Texel {
                pos: Position2D { x, y },
                symbol: *symbol,
                bg: texel_types::DEFAULT_BG_U8,
                fg: texel_types::DEFAULT_FG_U8,
                styles: SymbolStyles::new(),
            });
            x += 1;
            result.push(Texel {
                pos: Position2D { x, y },
                symbol: ' ',
                bg: texel_types::DEFAULT_BG_U8,
                fg: texel_types::DEFAULT_FG_U8,
                styles: SymbolStyles::new(),
            });
            x += 1;
        }

        result
    }
}
