//! Private term list and case-insensitive fixed-string matching (like `grep -i -F`).

pub struct Terms {
    /// Lowercased terms with their 1-based number in the source list.
    items: Vec<(usize, String)>,
}

impl Terms {
    /// One term per line; blank lines and lines starting with `#` are skipped.
    pub fn parse(raw: &str) -> Self {
        let items = raw
            .lines()
            .map(str::trim)
            .enumerate()
            .filter(|(_, t)| !t.is_empty() && !t.starts_with('#'))
            .map(|(i, t)| (i + 1, t.to_lowercase()))
            .collect();
        Self { items }
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// List numbers of the terms found in `text`.
    pub fn hits(&self, text: &str) -> Vec<usize> {
        let hay = text.to_lowercase();
        self.items
            .iter()
            .filter(|(_, t)| hay.contains(t.as_str()))
            .map(|(n, _)| *n)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::Terms;

    #[test]
    fn skips_blank_and_comment_lines_and_keeps_list_numbers() {
        let t = Terms::parse("# list\n\n alpha \nBeta\r\n");
        assert_eq!(t.len(), 2);
        assert_eq!(t.hits("ALPHA and beta"), vec![3, 4]);
    }

    #[test]
    fn matches_case_insensitively_including_non_ascii() {
        let t = Terms::parse("Ärger\n한글");
        assert_eq!(t.hits("kein ärger"), vec![1]);
        assert_eq!(t.hits("한글 문서"), vec![2]);
        assert!(t.hits("nothing").is_empty());
    }

    #[test]
    fn empty_or_comment_only_list_is_empty() {
        assert!(Terms::parse("").is_empty());
        assert!(Terms::parse("# only\n  \n").is_empty());
    }
}
