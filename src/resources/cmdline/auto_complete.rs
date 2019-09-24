use crate::common::{cwd_path, Error};
use std::fs::read_dir;
use std::path::Path;

#[derive(Debug)]
pub struct FileComplete {
    completions: Vec<String>,
    index: Option<usize>,
}

impl FileComplete {
    pub fn new() -> Self {
        Self {
            completions: Vec::with_capacity(256usize),
            index: None,
        }
    }

    pub fn clear(&mut self) {
        self.index = None
    }

    // TODO: refactor this convoluted mess
    pub fn with_path(&mut self, raw_path: &str) -> Result<Option<&String>, Error> {
        if let Some(index) = self.index {
            if index < self.completions.len() - 1 {
                self.index = Some(index + 1);
                return Ok(self.completions.get(index + 1));
            } else if self.completions.len() > 0 {
                self.index = Some(0);
                return Ok(self.completions.first());
            }

            return Ok(None);
        }

        let loc_path = Path::new(raw_path);
        let abs_path = cwd_path(loc_path)?;
        let mut loc_parent = loc_path.parent().unwrap_or(Path::new(""));
        let abs_parent = if abs_path.is_dir() {
            loc_parent = loc_path;
            &abs_path
        } else {
            abs_path.parent().unwrap_or(Path::new("/"))
        };

        if let Some(name) = loc_path.file_name() {
            let str_name = name.to_str().unwrap_or("");

            self.completions = read_dir(abs_parent)?
                .filter_map(|e| {
                    if let Some(entry) = e.ok() {
                        return match entry.file_name().to_str() {
                            None => None,
                            Some(s) => {
                                if abs_path.is_dir() || s.starts_with(str_name) {
                                    Some(String::from(loc_parent.join(s).to_str().unwrap_or("???")))
                                } else {
                                    None
                                }
                            }
                        };
                    }

                    None
                })
                .collect();

            if !self.completions.is_empty() {
                self.index = Some(0usize);
                return Ok(self.completions.first());
            }
        }

        Ok(None)
    }
}
