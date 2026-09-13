use std::path::PathBuf;
use std::collections::HashMap;
use std::error::Error;
use std::fs;

pub struct Editor {
    path: PathBuf,

    bools: HashMap<String, bool>,
    numbers: HashMap<String, usize>,
    byte_arrays: HashMap<String, Vec<u8>>,
    str_arrays: HashMap<String, Vec<String>>
}

impl Editor {
    pub fn new(path: &PathBuf) -> Self {
        Self {
            path: path.to_path_buf(),

            bools: HashMap::new(),
            numbers: HashMap::new(),
            byte_arrays: HashMap::new(),
            str_arrays: HashMap::new()
        }
    }

    pub fn bool(mut self, name: &str, value: bool) -> Self {
        self.bools.insert(name.to_string(), value);
        self
    }

    pub fn number(mut self, name: &str, value: usize) -> Self {
        self.numbers.insert(name.to_string(), value);
        self
    }

    pub fn byte_array(mut self, name: &str, value: &[u8]) -> Self {
        self.byte_arrays.insert(name.to_string(), value.to_vec());
        self
    }

    pub fn str_array(mut self, name: &str, value: &[impl AsRef<str>]) -> Self {
        self.str_arrays.insert(
            name.to_string(),
            value.iter().map(|str| str.as_ref().to_string()).collect()
        );

        self
    }

    pub fn finalize(self) -> Result<(), Box<dyn Error>> {
        let content = fs::read_to_string(&self.path)?;
        let mut lines = content
            .lines()
            .map(|line| line.to_string())
            .collect::<Vec<String>>();

        for (name, value) in &self.bools {
            for line in lines.iter_mut() {
                if !line.contains(&format!("pub const {}", name)) {
                    continue;
                }

                if !line.contains(": bool") {
                    continue;
                }

                *line = format!("pub const {}: bool = {};", name, value);
                break;
            }
        }

        for (name, value) in &self.numbers {
            for line in lines.iter_mut() {
                if !line.contains(&format!("pub const {}", name)) {
                    continue;
                }

                if !line.contains(": usize") {
                    continue;
                }

                *line = format!("pub const {}: usize = {};", name, value);
                break;
            }
        }

        for (name, value) in &self.byte_arrays {
            for line in lines.iter_mut() {
                if !line.contains(&format!("pub const {}", name)) {
                    continue;
                }

                if !line.contains(": &[u8]") {
                    continue;
                }

                *line = format!("pub const {}: &[u8] = &{:?};", name, value);
                break;
            }
        }

        for (name, value) in &self.str_arrays {
            for line in lines.iter_mut() {
                if !line.contains(&format!("pub const {}", name)) {
                    continue;
                }

                if !line.contains(": &[&str]") {
                    continue;
                }

                let inner = value
                    .iter()
                    .map(|str| format!("\"{}\"", str))
                    .collect::<Vec<_>>()
                    .join(", ");

                *line = format!("pub const {}: &[&str] = &[{}];", name, inner);
                break;
            }
        }

        fs::write(&self.path, lines.join("\n"))?;
        Ok(())
    }
}
