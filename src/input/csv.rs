use anyhow::Result;
use serde::Deserialize;
use std::path::Path;

use super::models::{InputEpic, InputFile, InputObjective, InputStory};
use crate::cli::ResourceType;

/// Parse a CSV file for the given resource type.
/// Multi-value fields (owners, teams, labels) use semicolons (`;`) as the
/// separator within a cell, since commas are the CSV delimiter.
pub fn parse(path: &Path, resource_type: &ResourceType) -> Result<InputFile> {
    match resource_type {
        ResourceType::Objective => Ok(InputFile {
            objectives: parse_typed(path, row_to_objective)?,
            ..Default::default()
        }),
        ResourceType::Epic => Ok(InputFile {
            epics: parse_typed(path, row_to_epic)?,
            ..Default::default()
        }),
        ResourceType::Story => Ok(InputFile {
            stories: parse_typed(path, row_to_story)?,
            ..Default::default()
        }),
    }
}

// ---------------------------------------------------------------------------
// Generic CSV reader
// ---------------------------------------------------------------------------

fn parse_typed<R, T, F>(path: &Path, convert: F) -> Result<Vec<T>>
where
    R: for<'de> Deserialize<'de>,
    F: Fn(R) -> T,
{
    let mut reader = csv::Reader::from_path(path)
        .map_err(|e| anyhow::anyhow!("Failed to open CSV '{}': {}", path.display(), e))?;
    let mut items = Vec::new();
    for (i, result) in reader.deserialize::<R>().enumerate() {
        let row = result.map_err(|e| anyhow::anyhow!("CSV row {} parse error: {}", i + 2, e))?;
        items.push(convert(row));
    }
    Ok(items)
}

/// Split a semicolon-delimited field value into individual trimmed strings.
fn split_semi(value: &str) -> Vec<String> {
    value
        .split(';')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn opt_str(s: String) -> Option<String> {
    let t = s.trim().to_string();
    if t.is_empty() { None } else { Some(t) }
}

// ---------------------------------------------------------------------------
// Objectives
// ---------------------------------------------------------------------------

/// CSV columns: name, description, state
#[derive(Deserialize)]
struct ObjRow {
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    state: String,
}

fn row_to_objective(r: ObjRow) -> InputObjective {
    InputObjective {
        name: r.name.trim().to_string(),
        description: opt_str(r.description),
        state: opt_str(r.state),
    }
}

// ---------------------------------------------------------------------------
// Epics
// ---------------------------------------------------------------------------

/// CSV columns: name, description, objective, owners, teams, labels, state,
///              start_date, deadline, template
/// Multi-value columns (owners, teams, labels) are semicolon-separated.
#[derive(Deserialize)]
struct EpicRow {
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    objective: String,
    #[serde(default)]
    owners: String,
    #[serde(default)]
    teams: String,
    #[serde(default)]
    labels: String,
    #[serde(default)]
    state: String,
    #[serde(default)]
    start_date: String,
    #[serde(default)]
    deadline: String,
    #[serde(default)]
    template: String,
}

fn row_to_epic(r: EpicRow) -> InputEpic {
    InputEpic {
        name: r.name.trim().to_string(),
        description: opt_str(r.description),
        objective: opt_str(r.objective),
        owners: split_semi(&r.owners),
        teams: split_semi(&r.teams),
        labels: split_semi(&r.labels),
        state: opt_str(r.state),
        start_date: opt_str(r.start_date),
        deadline: opt_str(r.deadline),
        template: opt_str(r.template),
    }
}

// ---------------------------------------------------------------------------
// Stories
// ---------------------------------------------------------------------------

/// CSV columns: name, type, description, epic, owners, team, labels,
///              estimate, due_date, workflow_state
/// Multi-value columns (owners, labels) are semicolon-separated.
#[derive(Deserialize)]
struct StoryRow {
    name: String,
    #[serde(rename = "type", default)]
    story_type: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    epic: String,
    #[serde(default)]
    owners: String,
    #[serde(default)]
    team: String,
    #[serde(default)]
    labels: String,
    /// Stored as a string so an empty cell becomes None after parsing.
    #[serde(default)]
    estimate: String,
    #[serde(default)]
    due_date: String,
    #[serde(default)]
    workflow_state: String,
}

fn row_to_story(r: StoryRow) -> InputStory {
    InputStory {
        name: r.name.trim().to_string(),
        story_type: opt_str(r.story_type),
        description: opt_str(r.description),
        epic: opt_str(r.epic),
        owners: split_semi(&r.owners),
        team: opt_str(r.team),
        labels: split_semi(&r.labels),
        estimate: r.estimate.trim().parse::<i64>().ok(),
        due_date: opt_str(r.due_date),
        workflow_state: opt_str(r.workflow_state),
    }
}
