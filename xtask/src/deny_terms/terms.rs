//! Private term list and case-insensitive fixed-string matching (like `grep -i -F`).

pub struct Terms {
    /// Lowercased terms with their label: the 1-based line number in the source list,
    /// prefixed with the list's tag (empty for the main list, `x` for the extra list).
    items: Vec<(String, String)>,
}

impl Terms {
    /// One term per line; blank lines and lines starting with `#` are skipped.
    pub fn parse(raw: &str) -> Self {
        Self::parse_tagged(raw, "")
    }

    /// Like [`Terms::parse`], with `tag` in front of every list number so a report
    /// names which list a hit came from without naming the term.
    pub fn parse_tagged(raw: &str, tag: &str) -> Self {
        let items = raw
            .lines()
            .map(str::trim)
            .enumerate()
            .filter(|(_, t)| !t.is_empty() && !t.starts_with('#'))
            .map(|(i, t)| (format!("{tag}{}", i + 1), t.to_lowercase()))
            .collect();
        Self { items }
    }

    /// Adds the terms of `other`; the union of both lists.
    pub fn extend(&mut self, other: Terms) {
        self.items.extend(other.items);
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// List labels of the terms found in `text`.
    pub fn hits(&self, text: &str) -> Vec<String> {
        let hay = text.to_lowercase();
        self.items
            .iter()
            .filter(|(_, t)| hay.contains(t.as_str()))
            .map(|(n, _)| n.clone())
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
        assert_eq!(t.hits("ALPHA and beta"), ["3", "4"]);
    }

    #[test]
    fn matches_case_insensitively_including_non_ascii() {
        let t = Terms::parse("Ärger\n한글");
        assert_eq!(t.hits("kein ärger"), ["1"]);
        assert_eq!(t.hits("한글 문서"), ["2"]);
        assert!(t.hits("nothing").is_empty());
    }

    #[test]
    fn empty_or_comment_only_list_is_empty() {
        assert!(Terms::parse("").is_empty());
        assert!(Terms::parse("# only\n  \n").is_empty());
    }

    #[test]
    fn extended_list_matches_both_and_tags_the_extra_numbers() {
        let mut t = Terms::parse("alpha");
        t.extend(Terms::parse_tagged("# extra\nGamma", "x"));
        assert_eq!(t.len(), 2);
        assert_eq!(t.hits("alpha gamma"), ["1", "x2"]);
    }
}
