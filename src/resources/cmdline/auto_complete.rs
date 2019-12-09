use crate::common::{cwd_path, Error, HELP_TOPICS};
use std::fs::read_dir;
use std::path::Path;

#[derive(Debug)]
pub struct AutoComplete {
    completions: Vec<String>,
    index: Option<usize>,
}

impl AutoComplete {
    pub fn new() -> Self {
        Self {
            completions: Vec::with_capacity(256usize),
            index: None,
        }
    }

    pub fn clear(&mut self) {
        self.completions.clear();
        self.index = None;
    }

    pub fn complete_help_topic(&mut self, word: &str) -> Option<&String> {
        if let Some(index) = self.index {
            if index < self.completions.len() - 1 {
                self.index = Some(index + 1);
                return self.completions.get(index + 1);
            } else if !self.completions.is_empty() {
                self.index = Some(0);
                return self.completions.first();
            }

            return None;
        }

        for found in HELP_TOPICS.iter().filter(|t| t.starts_with(word)) {
            self.completions.push(String::from(*found));
        }

        if !self.completions.is_empty() {
            self.index = Some(0usize);
            return self.completions.first();
        }

        None
    }

    // TODO: refactor this convoluted mess
    pub fn complete_filename(&mut self, raw_path: &str) -> Result<Option<&String>, Error> {
        if let Some(index) = self.index {
            if index < self.completions.len() - 1 {
                self.index = Some(index + 1);
                return Ok(self.completions.get(index + 1));
            } else if !self.completions.is_empty() {
                self.index = Some(0);
                return Ok(self.completions.first());
            }

            return Ok(None);
        }

        let loc_path = Path::new(raw_path);
        let abs_path = cwd_path(loc_path)?;
        let mut loc_parent = loc_path.parent().unwrap_or_else(|| Path::new(""));
        let abs_parent = if abs_path.is_dir() {
            loc_parent = loc_path;
            &abs_path
        } else {
            abs_path.parent().unwrap_or_else(|| Path::new("/"))
        };

        if let Some(name) = loc_path.file_name() {
            let str_name = name.to_str().unwrap_or_else(|| "");

            self.completions = read_dir(abs_parent)?
                .filter_map(|e| {
                    if let Ok(entry) = e {
                        return match entry.file_name().to_str() {
                            None => None,
                            Some(s) => {
                                if abs_path.is_dir() || s.starts_with(str_name) {
                                    Some(String::from(loc_parent.join(s).to_str().unwrap_or_else(|| "???")))
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
