use crate::common::Texel;
use specs::{Component, VecStorage};
use std::env::current_dir;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

/// 256 * 256 ascii chars maximum
pub const SPRITE_MAX_BYTES: usize = u16::max_value() as usize;

#[derive(Default, Debug, PartialEq, Eq)]
pub struct Sprite {
    pub texels: Vec<Texel>,
}

impl Sprite {
    pub fn from_file(path: &Path) -> Result<Self, std::io::Error> {
        let abs_path: PathBuf = if path.is_absolute() {
            path.to_path_buf()
        } else {
            let cwd = current_dir()?;
            cwd.join(path)
        };

        let mut f = File::open(abs_path)?;
        let mut buf: String = String::with_capacity(SPRITE_MAX_BYTES);
        let byte_size = f.read_to_string(&mut buf)?;

        if byte_size > SPRITE_MAX_BYTES {
            return Err(std::io::Error::from(std::io::ErrorKind::InvalidInput));
        }

        let mut texels = Vec::new();

        let mut x = 0;
        let mut y = 0;
        for c in buf.chars() {
            match c {
                ' ' => x += 1,
                '\n' => {
                    x = 0;
                    y += 1;
                }
                _ => {
                    texels.push(Texel {
                        x,
                        y,
                        symbol: c,
                        color: 0, // TODO
                    });
                    x += 1;
                }
            }
        }

        Ok(Sprite::from_texels(texels))
    }

    pub fn from_texels(texels: Vec<Texel>) -> Sprite {
        Sprite { texels }
    }
}

impl Component for Sprite {
    type Storage = VecStorage<Self>;
}
