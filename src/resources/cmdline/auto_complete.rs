use crate::common::{cwd_path, Error, HELP_TOPICS};
use std::fs::read_dir;
use std::path::Path;

#[derive(Debug)]
pub enum Completion {
    Filename(String),
    Directory(String),
    Parameter(String),
}

#[derive(Debug)]
pub struct AutoComplete {
    completions: Vec<Completion>, // bool for is_dir
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

    pub fn complete_help_topic(&mut self, word: &str) -> Option<&Completion> {
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
            let as_string = String::from(*found);
            self.completions.push(Completion::Parameter(as_string));
        }

        if !self.completions.is_empty() {
            self.index = Some(0usize);
            return self.completions.first();
        }

        None
    }

    // TODO: refactor this convoluted mess
    pub fn complete_filename(&mut self, raw_path: &str) -> Result<Option<&Completion>, Error> {
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
        let abs_parent: &Path;
        let str_name;

        if abs_path.is_dir() {
            loc_parent = loc_path;
            abs_parent = &abs_path;
            str_name = "";
        } else {
            abs_parent = abs_path.parent().unwrap_or_else(|| Path::new("/"));
            str_name = match loc_path.file_name() {
                Some(name) => name.to_str().unwrap_or_else(|| ""),
                None => "",
            };
        };

        let mut fs_error: Option<std::io::Error> = None;

        self.completions = read_dir(abs_parent)?
            .filter_map(|e| {
                if let Ok(entry) = e {
                    if fs_error.is_some() {
                        return None; // exit on 1st error
                    }

                    return match entry.file_name().to_str() {
                        None => None,
                        Some(s) => if s.starts_with(str_name) {
                            eprintln!("S: {:?}", s);
                            let file_type = match entry.file_type() {
                                Ok(ft) => ft,
                                Err(err) => {
                                    fs_error = Some(err);
                                    return None;
                                },
                            };

                            let found = String::from(loc_parent.join(s).to_str().unwrap_or_else(|| "???"));

                            match file_type.is_dir() {
                                true => Some(Completion::Directory(found)),
                                false => Some(Completion::Filename(found)),
                            }
                        } else {
                            None
                        }
                    };
                }

                None
            })
            .collect();
        
        if let Some(err) = fs_error {
            return Err(Error::from(err));
        }

        if !self.completions.is_empty() {
            self.index = Some(0usize);
            return Ok(self.completions.first());
        }

        Ok(None)
    }
}
