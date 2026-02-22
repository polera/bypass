use anyhow::Result;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use serde_json::json;

use crate::api::ShortcutClient;
use crate::api::models::{
    CreateEpicRequest, CreateLabelParams, CreateObjectiveRequest, CreateStoryRequest, Epic,
    Objective, Story,
};
use crate::cli::{CreateArgs, OutputFormat};
use crate::config::Config;
use crate::input;
use crate::input::models::{InputEpic, InputObjective, InputStory};
use crate::resolver::Resolver;
use crate::template::Template;

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

pub async fn run(args: CreateArgs, token: Option<String>) -> Result<()> {
    let config = Config::load(token)?;
    let client = ShortcutClient::new(config.api_token)?;

    // Parse the input file.
    let input = input::parse_file(&args.file, args.r#type.as_ref())?;

    let total = input.objectives.len() + input.epics.len() + input.stories.len();
    if total == 0 {
        eprintln!("{}", "No items found in the input file.".yellow());
        return Ok(());
    }

    // Load global epic template if provided.
    let global_template = args
        .template
        .as_ref()
        .map(|p| Template::load(p))
        .transpose()?;

    if matches!(args.output, OutputFormat::Text) {
        println!(
            "Parsed  {} objective(s)  {} epic(s)  {} story/stories",
            input.objectives.len().to_string().cyan(),
            input.epics.len().to_string().cyan(),
            input.stories.len().to_string().cyan(),
        );
    }

    // Fetch workspace data for name resolution.
    let status_msg = "Fetching workspace data (members, groups, workflows)…";
    if matches!(args.output, OutputFormat::Text) {
        eprint!("{status_msg}");
    }

    let mut resolver = Resolver::new(&client).await.inspect_err(|_| {
        if matches!(args.output, OutputFormat::Text) {
            eprintln!();
        }
    })?;

    if matches!(args.output, OutputFormat::Text) {
        eprintln!("  {}", "done".green());
    }

    if args.dry_run {
        return dry_run(&input, &resolver, global_template.as_ref(), &args.output);
    }

    let mut results = RunResults::default();

    // Create order: objectives → epics → stories so that name references
    // within the same file resolve correctly.

    // ---- Objectives ----
    if !input.objectives.is_empty() {
        let pb = make_pb(input.objectives.len() as u64, "objectives");
        for obj in &input.objectives {
            pb.set_message(obj.name.clone());
            match build_and_create_objective(&client, obj).await {
                Ok(created) => {
                    resolver.register_objective(obj.name.clone(), created.id);
                    results.objectives_ok += 1;
                    emit_ok(
                        &args.output,
                        "objective",
                        &created.name,
                        created.id,
                        created.app_url.as_deref(),
                        &pb,
                    );
                }
                Err(e) => {
                    results
                        .errors
                        .push(format!("Objective '{}': {e}", obj.name));
                    emit_err(&args.output, "objective", &obj.name, &e.to_string(), &pb);
                }
            }
            pb.inc(1);
        }
        pb.finish_and_clear();
    }

    // ---- Epics ----
    if !input.epics.is_empty() {
        let pb = make_pb(input.epics.len() as u64, "epics");
        for epic in &input.epics {
            pb.set_message(epic.name.clone());

            // Per-epic template overrides global template.
            let template = epic
                .template
                .as_ref()
                .map(|p| Template::load(std::path::Path::new(p)))
                .transpose()?
                .or_else(|| global_template.clone());

            match build_and_create_epic(&client, epic, &resolver, template.as_ref()).await {
                Ok(created) => {
                    resolver.register_epic(epic.name.clone(), created.id);
                    results.epics_ok += 1;
                    emit_ok(
                        &args.output,
                        "epic",
                        &created.name,
                        created.id,
                        created.app_url.as_deref(),
                        &pb,
                    );
                }
                Err(e) => {
                    results.errors.push(format!("Epic '{}': {e}", epic.name));
                    emit_err(&args.output, "epic", &epic.name, &e.to_string(), &pb);
                }
            }
            pb.inc(1);
        }
        pb.finish_and_clear();
    }

    // ---- Stories ----
    if !input.stories.is_empty() {
        let pb = make_pb(input.stories.len() as u64, "stories");
        for story in &input.stories {
            pb.set_message(story.name.clone());
            match build_and_create_story(&client, story, &resolver).await {
                Ok(created) => {
                    results.stories_ok += 1;
                    emit_ok(
                        &args.output,
                        "story",
                        &created.name,
                        created.id,
                        created.app_url.as_deref(),
                        &pb,
                    );
                }
                Err(e) => {
                    results.errors.push(format!("Story '{}': {e}", story.name));
                    emit_err(&args.output, "story", &story.name, &e.to_string(), &pb);
                }
            }
            pb.inc(1);
        }
        pb.finish_and_clear();
    }

    // ---- Summary ----
    if matches!(args.output, OutputFormat::Text) {
        println!(
            "\n{}",
            "─── Summary ───────────────────────────────────".dimmed()
        );
        println!(
            "  Objectives created : {}",
            results.objectives_ok.to_string().green()
        );
        println!(
            "  Epics created      : {}",
            results.epics_ok.to_string().green()
        );
        println!(
            "  Stories created    : {}",
            results.stories_ok.to_string().green()
        );
        if !results.errors.is_empty() {
            println!(
                "  Errors             : {}",
                results.errors.len().to_string().red()
            );
            for err in &results.errors {
                println!("    {} {err}", "✗".red());
            }
        }
    } else {
        // JSON summary line.
        println!(
            "{}",
            serde_json::to_string(&json!({
                "event": "summary",
                "objectives_created": results.objectives_ok,
                "epics_created": results.epics_ok,
                "stories_created": results.stories_ok,
                "error_count": results.errors.len(),
                "errors": results.errors,
            }))?
        );
    }

    if !results.errors.is_empty() {
        std::process::exit(1);
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Dry-run validation
// ---------------------------------------------------------------------------

fn dry_run(
    input: &crate::input::models::InputFile,
    resolver: &Resolver,
    global_template: Option<&Template>,
    output: &OutputFormat,
) -> Result<()> {
    let mut errors: Vec<String> = Vec::new();

    // Validate objectives.
    for obj in &input.objectives {
        if obj.name.is_empty() {
            errors.push("Objective: 'name' is required".into());
        }
        if let Some(state) = &obj.state {
            validate_objective_state(state, &obj.name, &mut errors);
        }
    }

    // Build a synthetic objective name set for cross-reference validation.
    let batch_objectives: std::collections::HashSet<&str> =
        input.objectives.iter().map(|o| o.name.as_str()).collect();

    // Validate epics.
    for epic in &input.epics {
        if epic.name.is_empty() {
            errors.push("Epic: 'name' is required".into());
            continue;
        }
        if let Some(state) = &epic.state {
            validate_epic_state(state, &epic.name, &mut errors);
        }
        for owner in &epic.owners {
            if !resolver.member_map.contains_key(owner.trim()) {
                errors.push(format!(
                    "Epic '{}': unknown user '{}'. Available: {}",
                    epic.name,
                    owner,
                    list_sample(resolver.available_members())
                ));
            }
        }
        for team in &epic.teams {
            if !resolver.group_map.contains_key(team.trim()) {
                errors.push(format!(
                    "Epic '{}': unknown team '{}'. Available: {}",
                    epic.name,
                    team,
                    list_sample(resolver.available_groups())
                ));
            }
        }
        if let Some(obj) = &epic.objective
            && obj.parse::<i64>().is_err()
            && !batch_objectives.contains(obj.as_str())
            && !resolver.objective_map.contains_key(obj.trim())
        {
            errors.push(format!(
                "Epic '{}': objective '{obj}' not found in current batch \
                     (use a numeric ID to reference a pre-existing objective)",
                epic.name
            ));
        }
        // Check per-epic template file exists.
        if let Some(tmpl_path) = &epic.template
            && !std::path::Path::new(tmpl_path).exists()
        {
            errors.push(format!(
                "Epic '{}': template file '{tmpl_path}' not found",
                epic.name
            ));
        }
    }

    let batch_epics: std::collections::HashSet<&str> =
        input.epics.iter().map(|e| e.name.as_str()).collect();

    // Validate stories.
    for story in &input.stories {
        if story.name.is_empty() {
            errors.push("Story: 'name' is required".into());
            continue;
        }
        if let Some(t) = &story.story_type
            && !["bug", "chore", "feature"].contains(&t.as_str())
        {
            errors.push(format!(
                "Story '{}': invalid type '{t}'. Must be 'bug', 'chore', or 'feature'",
                story.name
            ));
        }
        for owner in &story.owners {
            if !resolver.member_map.contains_key(owner.trim()) {
                errors.push(format!(
                    "Story '{}': unknown user '{}'. Available: {}",
                    story.name,
                    owner,
                    list_sample(resolver.available_members())
                ));
            }
        }
        if let Some(team) = &story.team
            && !resolver.group_map.contains_key(team.trim())
        {
            errors.push(format!(
                "Story '{}': unknown team '{}'. Available: {}",
                story.name,
                team,
                list_sample(resolver.available_groups())
            ));
        }
        if let Some(epic) = &story.epic
            && epic.parse::<i64>().is_err()
            && !batch_epics.contains(epic.as_str())
            && !resolver.epic_map.contains_key(epic.trim())
        {
            errors.push(format!(
                "Story '{}': epic '{epic}' not found in current batch \
                     (use a numeric ID to reference a pre-existing epic)",
                story.name
            ));
        }
        if let Some(ws) = &story.workflow_state
            && !resolver.workflow_state_map.contains_key(ws.trim())
        {
            errors.push(format!(
                "Story '{}': unknown workflow state '{}'. Available: {}",
                story.name,
                ws,
                list_sample(resolver.available_workflow_states())
            ));
        }
    }

    // Check global template.
    if let Some(_t) = global_template {
        // Template was already loaded successfully, nothing to do.
    }

    match output {
        OutputFormat::Text => {
            if errors.is_empty() {
                println!(
                    "{} All validations passed – no resources created (dry run).",
                    "✓".green()
                );
            } else {
                println!("{} {} validation error(s):", "✗".red(), errors.len());
                for e in &errors {
                    println!("  {} {e}", "•".red());
                }
                std::process::exit(1);
            }
        }
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string(&json!({
                    "event": "dry_run",
                    "valid": errors.is_empty(),
                    "errors": errors,
                }))?
            );
            if !errors.is_empty() {
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Resource builders
// ---------------------------------------------------------------------------

async fn build_and_create_objective(
    client: &ShortcutClient,
    input: &InputObjective,
) -> Result<Objective> {
    let req = CreateObjectiveRequest {
        name: input.name.clone(),
        description: input.description.clone(),
        state: input.state.clone(),
    };
    client.create_objective(&req).await
}

async fn build_and_create_epic(
    client: &ShortcutClient,
    input: &InputEpic,
    resolver: &Resolver,
    template: Option<&Template>,
) -> Result<Epic> {
    let owner_ids = if input.owners.is_empty() {
        None
    } else {
        Some(resolver.resolve_members(&input.owners)?)
    };

    let group_ids = if input.teams.is_empty() {
        None
    } else {
        Some(resolver.resolve_groups(&input.teams)?)
    };

    let objective_ids = input
        .objective
        .as_ref()
        .map(|name| Ok::<_, anyhow::Error>(vec![resolver.resolve_objective(name)?]))
        .transpose()?;

    let labels = labels_param(&input.labels);

    let description = match template {
        Some(t) => Some(t.render(input)),
        None => input.description.clone(),
    };

    let req = CreateEpicRequest {
        name: input.name.clone(),
        description,
        state: input.state.clone(),
        objective_ids,
        owner_ids,
        group_ids,
        labels,
        planned_start_date: input.start_date.clone(),
        deadline: input.deadline.clone(),
    };
    client.create_epic(&req).await
}

async fn build_and_create_story(
    client: &ShortcutClient,
    input: &InputStory,
    resolver: &Resolver,
) -> Result<Story> {
    let owner_ids = if input.owners.is_empty() {
        None
    } else {
        Some(resolver.resolve_members(&input.owners)?)
    };

    let group_id = input
        .team
        .as_ref()
        .map(|t| resolver.resolve_group(t))
        .transpose()?;

    let epic_id = input
        .epic
        .as_ref()
        .map(|e| resolver.resolve_epic(e))
        .transpose()?;

    let workflow_state_id = match &input.workflow_state {
        Some(name) => Some(resolver.resolve_workflow_state(name)?),
        None => resolver.default_workflow_state_id,
    };

    let labels = labels_param(&input.labels);

    let req = CreateStoryRequest {
        name: input.name.clone(),
        story_type: input.story_type.clone(),
        description: input.description.clone(),
        owner_ids,
        group_id,
        epic_id,
        workflow_state_id,
        labels,
        estimate: input.estimate,
        deadline: input.due_date.clone(),
    };
    client.create_story(&req).await
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn labels_param(names: &[String]) -> Option<Vec<CreateLabelParams>> {
    if names.is_empty() {
        None
    } else {
        Some(
            names
                .iter()
                .map(|n| CreateLabelParams { name: n.clone() })
                .collect(),
        )
    }
}

fn validate_objective_state(state: &str, name: &str, errors: &mut Vec<String>) {
    if !["in progress", "to do", "done"].contains(&state) {
        errors.push(format!(
            "Objective '{name}': invalid state '{state}'. \
             Must be 'in progress', 'to do', or 'done'"
        ));
    }
}

fn validate_epic_state(state: &str, name: &str, errors: &mut Vec<String>) {
    if !["in progress", "to do", "done"].contains(&state) {
        errors.push(format!(
            "Epic '{name}': invalid state '{state}'. \
             Must be 'in progress', 'to do', or 'done'"
        ));
    }
}

fn list_sample(mut items: Vec<&str>) -> String {
    items.sort_unstable();
    items.dedup();
    let preview: Vec<&str> = items.iter().copied().take(5).collect();
    if items.len() > 5 {
        format!("{} … (+{})", preview.join(", "), items.len() - 5)
    } else {
        preview.join(", ")
    }
}

fn make_pb(len: u64, label: &str) -> ProgressBar {
    let pb = ProgressBar::new(len);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(&format!(
                "{{spinner:.green}} Creating {label} [{{bar:40.cyan/blue}}] \
                 {{pos}}/{{len}} {{msg:.dim}}"
            ))
            .unwrap()
            .progress_chars("█▉▊▋▌▍▎▏  "),
    );
    pb
}

fn emit_ok(
    output: &OutputFormat,
    kind: &str,
    name: &str,
    id: i64,
    url: Option<&str>,
    pb: &ProgressBar,
) {
    match output {
        OutputFormat::Text => {
            pb.println(format!(
                "  {} {kind}: {name}  (#{id}){}",
                "✓".green(),
                url.map(|u| format!("  {}", u.dimmed())).unwrap_or_default()
            ));
        }
        OutputFormat::Json => {
            let line = serde_json::to_string(&json!({
                "event": "created",
                "kind": kind,
                "id": id,
                "name": name,
                "url": url,
            }))
            .unwrap_or_default();
            pb.println(line);
        }
    }
}

fn emit_err(output: &OutputFormat, kind: &str, name: &str, error: &str, pb: &ProgressBar) {
    match output {
        OutputFormat::Text => {
            pb.println(format!("  {} {kind}: {name}\n    {error}", "✗".red()));
        }
        OutputFormat::Json => {
            let line = serde_json::to_string(&json!({
                "event": "error",
                "kind": kind,
                "name": name,
                "error": error,
            }))
            .unwrap_or_default();
            pb.println(line);
        }
    }
}

// ---------------------------------------------------------------------------
// State tracking
// ---------------------------------------------------------------------------

#[derive(Default)]
struct RunResults {
    objectives_ok: usize,
    epics_ok: usize,
    stories_ok: usize,
    errors: Vec<String>,
}
