//! Raw CLI arguments (program name stripped).

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Args(Vec<String>);

impl Args {
    pub fn from_process() -> Self {
        Self(std::env::args().skip(1).collect())
    }

    pub fn new<I: IntoIterator<Item = S>, S: Into<String>>(items: I) -> Self {
        Self(items.into_iter().map(Into::into).collect())
    }

    pub fn first(&self) -> Option<&str> {
        self.get(0)
    }

    pub fn get(&self, i: usize) -> Option<&str> {
        self.0.get(i).map(String::as_str)
    }

    /// Value of `--name value` or `--name=value`.
    pub fn flag(&self, name: &str) -> Option<&str> {
        let eq = format!("{name}=");
        self.0.iter().enumerate().find_map(|(i, a)| {
            if a == name {
                self.0.get(i + 1).map(String::as_str)
            } else {
                a.strip_prefix(&eq)
            }
        })
    }
}
