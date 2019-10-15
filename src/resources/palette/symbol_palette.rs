use crate::common::LazyLoaded;
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
    #[serde(skip_serializing)]
    #[serde(default)]
    line_str: String,
}

impl Default for SymbolPalette {
    fn default() -> Self {
        SymbolPalette {
            symbols: DEFAULT_SYMBOLS,
            line_str: Self::to_line_string(&DEFAULT_SYMBOLS),
        }
    }
}

impl From<[char; SYMBOLS_IN_PALETTE]> for SymbolPalette {
    fn from(symbols: [char; SYMBOLS_IN_PALETTE]) -> Self {
        SymbolPalette {
            symbols,
            line_str: Self::to_line_string(&symbols),
        }
    }
}

impl From<&[char]> for SymbolPalette {
    fn from(symbols: &[char]) -> Self {
        let mut symbol_chars: [char; SYMBOLS_IN_PALETTE] = [' '; SYMBOLS_IN_PALETTE];

        for (i, symbol) in symbols.iter().enumerate() {
            symbol_chars[i] = *symbol;
        }

        SymbolPalette {
            symbols: symbol_chars,
            line_str: Self::to_line_string(&symbol_chars),
        }
    }
}

impl LazyLoaded for SymbolPalette {
    fn refresh(&mut self) {
        self.line_str = Self::to_line_string(&self.symbols);
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

    pub fn line_str(&self) -> &str {
        &self.line_str
    }

    fn to_line_string(symbols: &[char]) -> String {
        let mut result = String::with_capacity(SYMBOLS_IN_PALETTE * 4 + 40);
        result += termion::color::Reset.bg_str();
        result += termion::color::Reset.fg_str();
        result += termion::style::Reset.as_ref();

        for (i, c) in symbols.iter().enumerate() {
            result.push(SYMBOL_SELECTOR[i]);
            result.push(':');
            result.push(*c);
            result.push(' ');
        }

        result
    }
}
