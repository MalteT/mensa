
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Highlight {
    #[serde(default)]
    pub tag: TagHighlight,
    #[serde(default)]
    pub category: CategoryHighlight,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct TagFilter {
    #[serde(default)]
    pub allow: Vec<Tag>,
    #[serde(default)]
    pub deny: Vec<Tag>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(try_from = "CategoryFilterRaw")]
pub struct CategoryFilter {
    pub allow: Option<RegexSet>,
    pub deny: Option<RegexSet>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct CategoryFilterRaw {
    #[serde(default)]
    pub allow: Vec<String>,
    #[serde(default)]
    pub deny: Vec<String>,
}
