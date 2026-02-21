use serde::Deserialize;

/// Top-level structure for a YAML manifest.  All sections are optional so a
/// file may contain only epics, only stories, etc.
#[derive(Debug, Deserialize, Default)]
pub struct InputFile {
    #[serde(default)]
    pub objectives: Vec<InputObjective>,
    #[serde(default)]
    pub epics: Vec<InputEpic>,
    #[serde(default)]
    pub stories: Vec<InputStory>,
}

// ---------------------------------------------------------------------------
// Objectives
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Clone)]
pub struct InputObjective {
    pub name: String,
    pub description: Option<String>,
    /// "in progress" | "to do" | "done"
    pub state: Option<String>,
}

// ---------------------------------------------------------------------------
// Epics
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Clone)]
pub struct InputEpic {
    pub name: String,
    pub description: Option<String>,
    /// Objective name (resolved to ID) or a numeric ID string.
    pub objective: Option<String>,
    /// Owner names – may be a YAML list or a comma-separated string.
    #[serde(default, deserialize_with = "de_string_or_list")]
    pub owners: Vec<String>,
    /// Team names – may be a YAML list or a comma-separated string.
    #[serde(default, deserialize_with = "de_string_or_list")]
    pub teams: Vec<String>,
    /// Label names – may be a YAML list or a comma-separated string.
    #[serde(default, deserialize_with = "de_string_or_list")]
    pub labels: Vec<String>,
    /// "in progress" | "to do" | "done"
    pub state: Option<String>,
    /// ISO 8601 date, e.g. "2024-01-15".
    pub start_date: Option<String>,
    /// ISO 8601 date.
    pub deadline: Option<String>,
    /// Path to a per-epic markdown template file.
    /// If absent, the global --template flag is used.
    pub template: Option<String>,
}

// ---------------------------------------------------------------------------
// Stories
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Clone)]
pub struct InputStory {
    pub name: String,
    /// "feature" (default) | "bug" | "chore"
    #[serde(rename = "type")]
    pub story_type: Option<String>,
    pub description: Option<String>,
    /// Epic name (resolved to ID) or a numeric ID string.
    pub epic: Option<String>,
    /// Owner names – may be a YAML list or a comma-separated string.
    #[serde(default, deserialize_with = "de_string_or_list")]
    pub owners: Vec<String>,
    /// Single team/group name.
    pub team: Option<String>,
    /// Label names – may be a YAML list or a comma-separated string.
    #[serde(default, deserialize_with = "de_string_or_list")]
    pub labels: Vec<String>,
    /// Story point estimate.
    pub estimate: Option<i64>,
    /// ISO 8601 date.
    pub due_date: Option<String>,
    /// Workflow state name (e.g. "Backlog", "In Progress").
    pub workflow_state: Option<String>,
}

// ---------------------------------------------------------------------------
// Serde helper: accept either a YAML sequence OR a comma-separated string
// ---------------------------------------------------------------------------

fn de_string_or_list<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    use std::fmt;

    struct Vis;

    impl<'de> Visitor<'de> for Vis {
        type Value = Vec<String>;

        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("a string or sequence of strings")
        }

        fn visit_str<E: de::Error>(self, v: &str) -> Result<Vec<String>, E> {
            Ok(split_comma(v))
        }

        fn visit_string<E: de::Error>(self, v: String) -> Result<Vec<String>, E> {
            Ok(split_comma(&v))
        }

        fn visit_seq<A: de::SeqAccess<'de>>(self, mut seq: A) -> Result<Vec<String>, A::Error> {
            let mut out = Vec::new();
            while let Some(s) = seq.next_element::<String>()? {
                out.push(s.trim().to_string());
            }
            Ok(out)
        }

        fn visit_none<E: de::Error>(self) -> Result<Vec<String>, E> {
            Ok(vec![])
        }

        fn visit_unit<E: de::Error>(self) -> Result<Vec<String>, E> {
            Ok(vec![])
        }
    }

    deserializer.deserialize_any(Vis)
}

fn split_comma(s: &str) -> Vec<String> {
    s.split(',')
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
        .collect()
}
