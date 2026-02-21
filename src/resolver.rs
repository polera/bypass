use std::collections::HashMap;

use anyhow::Result;

use crate::api::ShortcutClient;
use crate::error::BypassError;

/// Holds lookup tables built from workspace data.
/// Also accumulates IDs of resources created in the current run so that
/// later resources in the same file can reference them by name.
pub struct Resolver {
    /// Full name, mention name, and email → member UUID.
    pub member_map: HashMap<String, String>,
    /// Full name and mention name → group UUID.
    pub group_map: HashMap<String, String>,
    /// Workflow state name → state integer ID.
    pub workflow_state_map: HashMap<String, i64>,
    /// The first "unstarted" workflow state found – used as the story default.
    pub default_workflow_state_id: Option<i64>,

    // In-run cross-reference maps (populated as resources are created).
    pub objective_map: HashMap<String, i64>,
    pub epic_map: HashMap<String, i64>,
}

impl Resolver {
    /// Fetch members, groups, and workflows in parallel and build lookup maps.
    pub async fn new(client: &ShortcutClient) -> Result<Self> {
        let (members, groups, workflows) = tokio::try_join!(
            client.list_members(),
            client.list_groups(),
            client.list_workflows(),
        )?;

        // ----- members -----
        let mut member_map: HashMap<String, String> = HashMap::new();
        for m in &members {
            if m.disabled {
                continue;
            }
            member_map.insert(m.profile.name.clone(), m.id.clone());
            member_map.insert(m.profile.mention_name.clone(), m.id.clone());
            if let Some(email) = &m.profile.email_address {
                member_map.insert(email.clone(), m.id.clone());
            }
        }

        // ----- groups / teams -----
        let mut group_map: HashMap<String, String> = HashMap::new();
        for g in &groups {
            if g.archived {
                continue;
            }
            group_map.insert(g.name.clone(), g.id.clone());
            group_map.insert(g.mention_name.clone(), g.id.clone());
        }

        // ----- workflow states -----
        let mut workflow_state_map: HashMap<String, i64> = HashMap::new();
        let mut default_workflow_state_id: Option<i64> = None;

        for wf in &workflows {
            for state in &wf.states {
                // Last-write wins for duplicate names across workflows.
                workflow_state_map.insert(state.name.clone(), state.id);
                if default_workflow_state_id.is_none() && state.state_type == "unstarted" {
                    default_workflow_state_id = Some(state.id);
                }
            }
            // Fall back to the workflow's declared default state.
            if default_workflow_state_id.is_none() {
                default_workflow_state_id = Some(wf.default_state_id);
            }
        }

        Ok(Self {
            member_map,
            group_map,
            workflow_state_map,
            default_workflow_state_id,
            objective_map: HashMap::new(),
            epic_map: HashMap::new(),
        })
    }

    // ------------------------------------------------------------------
    // Lookups
    // ------------------------------------------------------------------

    pub fn resolve_member(&self, name: &str) -> Result<String> {
        self.member_map
            .get(name.trim())
            .cloned()
            .ok_or_else(|| BypassError::NameNotFound { resource_type: "user".into(), name: name.to_string() }.into())
    }

    pub fn resolve_members(&self, names: &[String]) -> Result<Vec<String>> {
        names.iter().map(|n| self.resolve_member(n)).collect()
    }

    pub fn resolve_group(&self, name: &str) -> Result<String> {
        self.group_map
            .get(name.trim())
            .cloned()
            .ok_or_else(|| BypassError::NameNotFound { resource_type: "team".into(), name: name.to_string() }.into())
    }

    pub fn resolve_groups(&self, names: &[String]) -> Result<Vec<String>> {
        names.iter().map(|n| self.resolve_group(n)).collect()
    }

    pub fn resolve_workflow_state(&self, name: &str) -> Result<i64> {
        self.workflow_state_map
            .get(name.trim())
            .copied()
            .ok_or_else(|| BypassError::NameNotFound { resource_type: "workflow state".into(), name: name.to_string() }.into())
    }

    /// Resolve an objective by name.  Accepts a raw integer string as a
    /// pass-through numeric ID (e.g. "12345").
    pub fn resolve_objective(&self, name: &str) -> Result<i64> {
        if let Ok(id) = name.trim().parse::<i64>() {
            return Ok(id);
        }
        self.objective_map
            .get(name.trim())
            .copied()
            .ok_or_else(|| BypassError::NameNotFound { resource_type: "objective".into(), name: name.to_string() }.into())
    }

    /// Resolve an epic by name.  Accepts a raw integer string as a
    /// pass-through numeric ID.
    pub fn resolve_epic(&self, name: &str) -> Result<i64> {
        if let Ok(id) = name.trim().parse::<i64>() {
            return Ok(id);
        }
        self.epic_map
            .get(name.trim())
            .copied()
            .ok_or_else(|| BypassError::NameNotFound { resource_type: "epic".into(), name: name.to_string() }.into())
    }

    // ------------------------------------------------------------------
    // Registration (called after successful creation)
    // ------------------------------------------------------------------

    pub fn register_objective(&mut self, name: String, id: i64) {
        self.objective_map.insert(name, id);
    }

    pub fn register_epic(&mut self, name: String, id: i64) {
        self.epic_map.insert(name, id);
    }

    // ------------------------------------------------------------------
    // Available names (for error hints)
    // ------------------------------------------------------------------

    pub fn available_members(&self) -> Vec<&str> {
        self.member_map.keys().map(|s| s.as_str()).collect()
    }

    pub fn available_groups(&self) -> Vec<&str> {
        self.group_map.keys().map(|s| s.as_str()).collect()
    }

    pub fn available_workflow_states(&self) -> Vec<&str> {
        self.workflow_state_map.keys().map(|s| s.as_str()).collect()
    }
}
