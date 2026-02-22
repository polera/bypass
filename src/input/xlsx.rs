use anyhow::{Result, anyhow, bail};
use calamine::{DataType, Range, Reader, Xlsx, open_workbook};
use std::collections::HashMap;
use std::path::Path;

use super::models::{InputEpic, InputFile, InputObjective, InputStory};
use crate::cli::ResourceType;

/// Parse an Excel (.xlsx) file.
///
/// If `--type` is provided, the **first** sheet is used.
/// Otherwise, sheets whose names contain "objective", "epic", or "stor"
/// (case-insensitive) are parsed automatically.
pub fn parse(path: &Path, resource_type: Option<&ResourceType>) -> Result<InputFile> {
    let mut workbook: Xlsx<_> = open_workbook(path)
        .map_err(|e| anyhow!("Cannot open Excel file '{}': {}", path.display(), e))?;

    let sheet_names: Vec<String> = workbook.sheet_names().to_vec();

    if let Some(rt) = resource_type {
        let sheet = sheet_names
            .first()
            .ok_or_else(|| anyhow!("Excel file has no sheets"))?
            .clone();
        let range = get_range(&mut workbook, &sheet)?;
        match rt {
            ResourceType::Objective => Ok(InputFile {
                objectives: objectives_from_range(&range)?,
                ..Default::default()
            }),
            ResourceType::Epic => Ok(InputFile {
                epics: epics_from_range(&range)?,
                ..Default::default()
            }),
            ResourceType::Story => Ok(InputFile {
                stories: stories_from_range(&range)?,
                ..Default::default()
            }),
        }
    } else {
        let mut result = InputFile::default();
        let mut matched = false;

        for sheet in &sheet_names {
            let lower = sheet.to_lowercase();
            let range = get_range(&mut workbook, sheet)?;

            if lower.contains("objective") {
                result.objectives = objectives_from_range(&range)?;
                matched = true;
            } else if lower.contains("epic") {
                result.epics = epics_from_range(&range)?;
                matched = true;
            } else if lower.contains("stor") {
                result.stories = stories_from_range(&range)?;
                matched = true;
            }
        }

        if !matched {
            bail!(
                "No recognized sheet names in '{}'. \
                 Name sheets 'Objectives', 'Epics', or 'Stories', \
                 or supply --type to use the first sheet.",
                path.display()
            );
        }
        Ok(result)
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn get_range(
    workbook: &mut Xlsx<std::io::BufReader<std::fs::File>>,
    name: &str,
) -> Result<Range<DataType>> {
    workbook
        .worksheet_range(name)
        .map_err(|e| anyhow!("Error reading sheet '{}': {}", name, e))
}

/// Build a header-name → column-index map from the first row of a range.
fn headers(range: &Range<DataType>) -> HashMap<String, usize> {
    let mut map = HashMap::new();
    if let Some(row) = range.rows().next() {
        for (i, cell) in row.iter().enumerate() {
            if let DataType::String(s) = cell {
                map.insert(s.trim().to_lowercase(), i);
            }
        }
    }
    map
}

fn cell_str(row: &[DataType], idx: usize) -> String {
    match row.get(idx) {
        Some(DataType::String(s)) => s.trim().to_string(),
        Some(DataType::Float(f)) => format!("{}", f),
        Some(DataType::Int(i)) => i.to_string(),
        Some(DataType::Bool(b)) => b.to_string(),
        _ => String::new(),
    }
}

fn opt_cell(row: &[DataType], idx: usize) -> Option<String> {
    let s = cell_str(row, idx);
    if s.is_empty() { None } else { Some(s) }
}

fn opt_cell_i64(row: &[DataType], idx: usize) -> Option<i64> {
    match row.get(idx) {
        Some(DataType::Float(f)) => Some(*f as i64),
        Some(DataType::Int(i)) => Some(*i),
        Some(DataType::String(s)) => s.trim().parse().ok(),
        _ => None,
    }
}

fn split_semi(s: &str) -> Vec<String> {
    s.split(';')
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
        .collect()
}

// ---------------------------------------------------------------------------
// Sheet → model converters
// ---------------------------------------------------------------------------

fn objectives_from_range(range: &Range<DataType>) -> Result<Vec<InputObjective>> {
    let hdr = headers(range);
    let name_col = hdr
        .get("name")
        .copied()
        .ok_or_else(|| anyhow!("Missing 'name' column"))?;
    let mut out = Vec::new();

    for (i, row) in range.rows().enumerate() {
        if i == 0 {
            continue; // skip header
        }
        let name = cell_str(row, name_col);
        if name.is_empty() {
            continue;
        }
        out.push(InputObjective {
            name,
            description: hdr.get("description").and_then(|&c| opt_cell(row, c)),
            state: hdr.get("state").and_then(|&c| opt_cell(row, c)),
        });
    }
    Ok(out)
}

fn epics_from_range(range: &Range<DataType>) -> Result<Vec<InputEpic>> {
    let hdr = headers(range);
    let name_col = hdr
        .get("name")
        .copied()
        .ok_or_else(|| anyhow!("Missing 'name' column"))?;
    let mut out = Vec::new();

    for (i, row) in range.rows().enumerate() {
        if i == 0 {
            continue;
        }
        let name = cell_str(row, name_col);
        if name.is_empty() {
            continue;
        }
        out.push(InputEpic {
            name,
            description: hdr.get("description").and_then(|&c| opt_cell(row, c)),
            objective: hdr.get("objective").and_then(|&c| opt_cell(row, c)),
            owners: hdr
                .get("owners")
                .map(|&c| split_semi(&cell_str(row, c)))
                .unwrap_or_default(),
            teams: hdr
                .get("teams")
                .map(|&c| split_semi(&cell_str(row, c)))
                .unwrap_or_default(),
            labels: hdr
                .get("labels")
                .map(|&c| split_semi(&cell_str(row, c)))
                .unwrap_or_default(),
            state: hdr.get("state").and_then(|&c| opt_cell(row, c)),
            start_date: hdr.get("start_date").and_then(|&c| opt_cell(row, c)),
            deadline: hdr.get("deadline").and_then(|&c| opt_cell(row, c)),
            template: hdr.get("template").and_then(|&c| opt_cell(row, c)),
        });
    }
    Ok(out)
}

fn stories_from_range(range: &Range<DataType>) -> Result<Vec<InputStory>> {
    let hdr = headers(range);
    let name_col = hdr
        .get("name")
        .copied()
        .ok_or_else(|| anyhow!("Missing 'name' column"))?;
    let mut out = Vec::new();

    for (i, row) in range.rows().enumerate() {
        if i == 0 {
            continue;
        }
        let name = cell_str(row, name_col);
        if name.is_empty() {
            continue;
        }
        out.push(InputStory {
            name,
            story_type: hdr.get("type").and_then(|&c| opt_cell(row, c)),
            description: hdr.get("description").and_then(|&c| opt_cell(row, c)),
            epic: hdr.get("epic").and_then(|&c| opt_cell(row, c)),
            owners: hdr
                .get("owners")
                .map(|&c| split_semi(&cell_str(row, c)))
                .unwrap_or_default(),
            team: hdr.get("team").and_then(|&c| opt_cell(row, c)),
            labels: hdr
                .get("labels")
                .map(|&c| split_semi(&cell_str(row, c)))
                .unwrap_or_default(),
            estimate: hdr.get("estimate").and_then(|&c| opt_cell_i64(row, c)),
            due_date: hdr.get("due_date").and_then(|&c| opt_cell(row, c)),
            workflow_state: hdr.get("workflow_state").and_then(|&c| opt_cell(row, c)),
        });
    }
    Ok(out)
}
