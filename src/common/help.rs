use crate::common::fio::{Loaded, scene_from_rgz_stream};
use texel_types::Scene;

#[derive(Debug)]
pub enum Help {
    Overview(Scene),
    Commands(Scene),
    Modes(Scene),
    Keys(Scene),
}

pub const HELP_TOPICS: [&'static str; 4] = [
    "overview",
    "commands",
    "modes",
    "keys",
];

macro_rules! gen_help {
    ($name:ident, $help_enum:ident) => {
        pub fn $name() -> Self {
            let bytes = include_bytes!(concat!("../../help/", stringify!($name), ".rgz"));
            let loaded = scene_from_rgz_stream(&bytes[..]).unwrap();

            match loaded {
                Loaded::Scene(scene) => Help::$help_enum(scene),
                Loaded::Sprite(_) => panic!("Invalid const situation"),
            }
        }
    }
}

impl Help {
    pub fn from_word(word: &str) -> Option<Self> {
        match word {
            "overview" => Some(Help::overview()),
            "commands" => Some(Help::commands()),
            "modes" => Some(Help::modes()),
            "keys" => Some(Help::keys()),
            _ => None,
        }
    }

    gen_help!(overview, Overview);
    gen_help!(commands, Commands);
    gen_help!(modes, Modes);
    gen_help!(keys, Keys);
}