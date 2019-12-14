use crate::exporters::Exporter;
use texel_types::Scene;
use std::io::{Write, Error};

pub struct Plaintext;

type Line = Vec<char>; // we need to override at position and this is way more efficient than string
type Lines = Vec<Line>;

impl Exporter for Plaintext {
    fn export(scene: Scene, output: &mut impl Write) -> Result<(), Error> {
        let mut lines: Lines = Vec::with_capacity(256);
        let mut str_line = String::with_capacity(1024); // we should be able to fit this
        let mut new_line = [0u8; 4];
        let new_line_len = '\n'.encode_utf8(&mut new_line).bytes().len();

        let mut sorted = scene.current().objects.clone();
        sorted.sort_by(|o1, o2| o1.1.z.cmp(&o2.1.z));

        for obj in sorted {
            let pos = obj.1;
            let sprite = obj.0;

            for texel in sprite.frame_iter() {
                let abs_pos = pos + texel.pos;
                if abs_pos.x < 0 || abs_pos.y < 0 {
                    continue
                }
                
                let col = abs_pos.x as usize;
                let row = abs_pos.y as usize;

                fill_to_pos(&mut lines, row, col);

                lines[row][col] = texel.symbol;
            }
        }

        for line in lines {
            str_line.clear();
            for c in line {
                str_line.push(c); // re-utf8
            }

            let bytes = str_line.as_bytes();
            output.write_all(bytes)?;
            output.write_all(&new_line[..new_line_len])?;
        }

        Ok(())
    }
}

fn fill_to_pos(lines: &mut Lines, row: usize, col: usize) {
    // fill in empty "rows"
    while lines.len() <= row {
        lines.push(Line::with_capacity(1024));
    }

    // fill in left-side spaces as needed
    while lines[row].len() <= col {
        lines[row].push(' ');
    }
}
