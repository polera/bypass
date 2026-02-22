use anyhow::Result;
use std::path::Path;

use crate::input::models::InputEpic;

/// A markdown template for epic descriptions.
///
/// The template file is read once and rendered per-epic by replacing
/// `{{variable}}` placeholders with the epic's field values.
///
/// Available variables:
/// - `{{name}}`        – epic name
/// - `{{description}}` – raw description from the input (may be empty)
/// - `{{objective}}`   – linked objective name (may be empty)
/// - `{{owners}}`      – comma-separated owner names
/// - `{{teams}}`       – comma-separated team names
/// - `{{labels}}`      – comma-separated label names
/// - `{{start_date}}`  – planned start date
/// - `{{deadline}}`    – deadline
#[derive(Clone)]
pub struct Template {
    content: String,
}

impl Template {
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Cannot read template '{}': {}", path.display(), e))?;
        Ok(Self { content })
    }

    /// Render the template with values from `epic`.
    /// Unrecognised placeholders are left as-is.
    pub fn render(&self, epic: &InputEpic) -> String {
        let vars: &[(&str, &str)] = &[
            ("name", &epic.name),
            (
                "description",
                epic.description.as_deref().unwrap_or_default(),
            ),
            ("objective", epic.objective.as_deref().unwrap_or_default()),
            ("start_date", epic.start_date.as_deref().unwrap_or_default()),
            ("deadline", epic.deadline.as_deref().unwrap_or_default()),
        ];

        // Build strings for multi-value fields so we can borrow them.
        let owners = epic.owners.join(", ");
        let teams = epic.teams.join(", ");
        let labels = epic.labels.join(", ");

        let mut result = self.content.clone();
        for (key, value) in vars {
            result = result.replace(&format!("{{{{{key}}}}}"), value);
        }
        result = result.replace("{{owners}}", &owners);
        result = result.replace("{{teams}}", &teams);
        result = result.replace("{{labels}}", &labels);
        result
    }
}
