
mod plaintext;

pub use plaintext::Plaintext;

pub trait Exporter {
    fn export(scene: texel_types::Scene, output: &mut impl std::io::Write) -> Result<(), std::io::Error>;
}
