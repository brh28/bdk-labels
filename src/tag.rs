/// A tag is a colon-separated path of segments.
///
/// Single segment:    `["kyc"]`                      → serialized as `kyc:`
/// Key-value:         `["project", "income"]`         → `project:income`
/// Hierarchical:      `["debit", "expenses", "food"]` → `debit:expenses:food`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag(Vec<String>);

impl Tag {
    pub fn new(segments: Vec<String>) -> Option<Self> {
        if segments.is_empty() || segments.iter().any(|s| s.is_empty()) {
            return None;
        }
        Some(Self(segments))
    }

    pub fn segments(&self) -> &[String] {
        &self.0
    }

    /// Parse a single tag token (e.g. `"kyc:"`, `"project:income"`).
    pub fn parse(token: &str) -> Option<Self> {
        let segments: Vec<String> = token
            .split(':')
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .collect();
        Self::new(segments)
    }

    /// Serialize to a label string token.
    pub fn to_token(&self) -> String {
        if self.0.len() == 1 {
            format!("{}:", self.0[0])
        } else {
            self.0.join(":")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_boolean_tag() {
        let tag = Tag::parse("kyc:").unwrap();
        assert_eq!(tag.segments(), &["kyc"]);
        assert_eq!(tag.to_token(), "kyc:");
    }

    #[test]
    fn parse_key_value_tag() {
        let tag = Tag::parse("project:income").unwrap();
        assert_eq!(tag.segments(), &["project", "income"]);
        assert_eq!(tag.to_token(), "project:income");
    }

    #[test]
    fn parse_hierarchical_tag() {
        let tag = Tag::parse("debit:expenses:groceries").unwrap();
        assert_eq!(tag.segments(), &["debit", "expenses", "groceries"]);
        assert_eq!(tag.to_token(), "debit:expenses:groceries");
    }
}
