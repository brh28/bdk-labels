use crate::tag::Tag;

/// Our extended label: a human-readable description plus structured tags,
/// serialized into the BIP329 `label` string field.
///
/// Format: `"description ; tag1 tag2:val tag3:a:b"`
/// If no ` ; ` is present, the entire string is a description with no tags.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Label {
    pub description: Option<String>,
    pub tags: Vec<Tag>,
}

impl Label {
    pub fn empty() -> Self {
        Self { description: None, tags: vec![] }
    }

    /// Parse from a BIP329 `label` string.
    pub fn parse(s: &str) -> Self {
        match s.split_once(" ; ") {
            None => {
                let desc = s.trim().to_owned();
                Self {
                    description: if desc.is_empty() { None } else { Some(desc) },
                    tags: vec![],
                }
            }
            Some((desc, tags_str)) => {
                let desc = desc.trim().to_owned();
                let tags = tags_str
                    .split_whitespace()
                    .filter_map(Tag::parse)
                    .collect();
                Self {
                    description: if desc.is_empty() { None } else { Some(desc) },
                    tags,
                }
            }
        }
    }

    /// Serialize to a BIP329 `label` string. Returns `None` if empty.
    pub fn to_bip329_string(&self) -> Option<String> {
        let desc = self.description.as_deref().unwrap_or("").trim();
        if self.tags.is_empty() {
            if desc.is_empty() { None } else { Some(desc.to_owned()) }
        } else {
            let tag_str = self.tags.iter().map(Tag::to_token).collect::<Vec<_>>().join(" ");
            if desc.is_empty() {
                Some(format!(" ; {tag_str}"))
            } else {
                Some(format!("{desc} ; {tag_str}"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_description_only() {
        let label = Label::parse("groceries");
        assert_eq!(label.description.as_deref(), Some("groceries"));
        assert!(label.tags.is_empty());
    }

    #[test]
    fn parse_with_tags() {
        let label = Label::parse("groceries ; kyc: project:income debit:expenses:groceries");
        assert_eq!(label.description.as_deref(), Some("groceries"));
        assert_eq!(label.tags.len(), 3);
        assert_eq!(label.tags[0].segments(), &["kyc"]);
        assert_eq!(label.tags[1].segments(), &["project", "income"]);
        assert_eq!(label.tags[2].segments(), &["debit", "expenses", "groceries"]);
    }

    #[test]
    fn parse_tags_only() {
        let label = Label::parse(" ; kyc: project:income");
        assert!(label.description.is_none());
        assert_eq!(label.tags.len(), 2);
    }

    #[test]
    fn empty_when_blank() {
        let label = Label::parse("");
        assert!(label.description.is_none());
        assert!(label.tags.is_empty());
        assert!(label.to_bip329_string().is_none());
    }

    #[test]
    fn roundtrip() {
        let input = "groceries ; kyc: project:income debit:expenses:groceries";
        let label = Label::parse(input);
        assert_eq!(label.to_bip329_string().as_deref(), Some(input));
    }

    #[test]
    fn roundtrip_description_only() {
        let input = "groceries";
        let label = Label::parse(input);
        assert_eq!(label.to_bip329_string().as_deref(), Some(input));
    }
}
