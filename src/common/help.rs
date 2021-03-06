use crate::common::fio::{scene_from_rgz_stream, Loaded};
use texel_types::Scene;

pub const HELP_TOPICS: [&str; 5] = ["overview", "commands", "modes", "keys", "config"];

pub const HELP_CONTENTS: [&[u8]; 5] = [
    include_bytes!("../../help/overview.rgz"),
    include_bytes!("../../help/commands.rgz"),
    include_bytes!("../../help/modes.rgz"),
    include_bytes!("../../help/keys.rgz"),
    include_bytes!("../../help/config.rgz"),
];

pub fn topic_index(word: &str) -> Option<usize> {
    match word {
        "overview" => Some(0),
        "commands" => Some(1),
        "modes" => Some(2),
        "keys" => Some(3),
        "config" => Some(4),
        _ => None,
    }
}

pub fn scene_for_help_index(index: usize) -> Scene {
    let bytes = HELP_CONTENTS[index];
    let loaded = scene_from_rgz_stream(bytes).unwrap();

    match loaded {
        Loaded::Scene(scene) => scene,
        Loaded::Sprite(_) => panic!("Invalid const situation"),
    }
}
