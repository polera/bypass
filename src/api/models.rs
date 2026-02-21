use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Shared
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Clone)]
pub struct CreateLabelParams {
    pub name: String,
}

// ---------------------------------------------------------------------------
// Objectives
// ---------------------------------------------------------------------------

/// POST /api/v3/objectives
#[derive(Debug, Serialize, Default)]
#[allow(dead_code)]
pub struct CreateObjectiveRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// "in progress" | "to do" | "done"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Objective {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub state: String,
    pub app_url: Option<String>,
}

// ---------------------------------------------------------------------------
// Epics
// ---------------------------------------------------------------------------

/// POST /api/v3/epics
#[derive(Debug, Serialize, Default)]
pub struct CreateEpicRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// "in progress" | "to do" | "done"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    /// Links to Shortcut Objectives (preferred v3 field).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub objective_ids: Option<Vec<i64>>,
    /// Owner member UUIDs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_ids: Option<Vec<String>>,
    /// Team/group UUIDs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<CreateLabelParams>>,
    /// ISO 8601 date, e.g. "2024-01-15".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub planned_start_date: Option<String>,
    /// ISO 8601 date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadline: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Epic {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub state: String,
    pub app_url: Option<String>,
}

// ---------------------------------------------------------------------------
// Stories
// ---------------------------------------------------------------------------

/// POST /api/v3/stories
#[derive(Debug, Serialize, Default)]
pub struct CreateStoryRequest {
    pub name: String,
    /// "feature" (default) | "bug" | "chore"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub story_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Owner member UUIDs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_ids: Option<Vec<String>>,
    /// Single team/group UUID for stories.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id: Option<String>,
    /// Parent epic integer ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub epic_id: Option<i64>,
    /// Workflow state integer ID (required by API; defaults to first unstarted state).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow_state_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<CreateLabelParams>>,
    /// Story point estimate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimate: Option<i64>,
    /// ISO 8601 date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadline: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Story {
    pub id: i64,
    pub name: String,
    pub story_type: String,
    pub app_url: Option<String>,
}

// ---------------------------------------------------------------------------
// Members / Groups / Workflows  (read-only, for name resolution)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct Member {
    pub id: String,
    pub profile: MemberProfile,
    #[serde(default)]
    pub disabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct MemberProfile {
    pub name: String,
    pub mention_name: String,
    pub email_address: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Group {
    pub id: String,
    pub name: String,
    pub mention_name: String,
    #[serde(default)]
    pub archived: bool,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Workflow {
    pub id: i64,
    pub name: String,
    pub default_state_id: i64,
    pub states: Vec<WorkflowState>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WorkflowState {
    pub id: i64,
    pub name: String,
    #[serde(rename = "type")]
    pub state_type: String,
}
