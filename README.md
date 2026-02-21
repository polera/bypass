# bypass

Rust CLI for bulk-creating [Shortcut](https://shortcut.com) Objectives, Epics, and Stories via the Shortcut REST API v3.

## Installation

```sh
cargo install --path .
```

## Authentication

Provide your Shortcut API token via one of these methods (highest priority first):

1. `--token <TOKEN>` flag
2. `SHORTCUT_API_TOKEN` environment variable
3. `~/.config/bypass/config.yaml`:
   ```yaml
   api_token: your-token-here
   ```

## Usage

```
bypass create --file <FILE> [OPTIONS]
```

### Options

| Flag | Description |
|------|-------------|
| `--file <FILE>` | Input file (`.yaml`, `.csv`, or `.xlsx`) |
| `--type <TYPE>` | Resource type: `objective`, `epic`, or `story` — required for CSV/XLSX |
| `--template <FILE>` | Markdown template applied to every epic without an inline template |
| `--dry-run` | Validate and resolve names without creating anything |
| `--output <FORMAT>` | `text` (default) or `json` (newline-delimited JSON) |
| `--token <TOKEN>` | Shortcut API token |

## Input Formats

### YAML (recommended)

A single YAML file can contain all three resource types. Resources are created in order: Objectives → Epics → Stories, so in-batch name cross-references resolve correctly.

```sh
bypass create --file examples/manifest.yaml
bypass create --file examples/manifest.yaml --dry-run
bypass create --file examples/manifest.yaml --template examples/epic_template.md
```

See [`examples/manifest.yaml`](examples/manifest.yaml) for a full example.

### CSV

One resource type per file. Use `--type` to specify which kind.

```sh
bypass create --file examples/objectives.csv --type objective
bypass create --file examples/epics.csv     --type epic
bypass create --file examples/stories.csv   --type story
```

Multi-value fields (owners, teams, labels) use `;` as the delimiter within a cell.

### XLSX

Sheet names containing `objective`, `epic`, or `stor` (case-insensitive) are auto-detected. Otherwise `--type` is required.

```sh
bypass create --file data.xlsx
bypass create --file data.xlsx --type epic
```

## Fields

### Objectives

| Field | Required | Description |
|-------|----------|-------------|
| `name` | yes | Objective name |
| `description` | no | Plain-text description |
| `state` | no | `to do`, `in progress`, or `done` |

### Epics

| Field | Required | Description |
|-------|----------|-------------|
| `name` | yes | Epic name |
| `description` | no | Plain-text description |
| `objective` | no | Objective name or numeric Shortcut ID |
| `owners` | no | List or comma-separated member names |
| `teams` | no | List or comma-separated group names |
| `labels` | no | List or comma-separated label names |
| `state` | no | `to do`, `in progress`, or `done` |
| `start_date` | no | ISO 8601 date (e.g. `2024-07-01`) |
| `deadline` | no | ISO 8601 date |
| `template` | no | Path to a Markdown template file |

### Stories

| Field | Required | Description |
|-------|----------|-------------|
| `name` | yes | Story name |
| `type` | no | `feature`, `bug`, or `chore` (default: `feature`) |
| `description` | no | Plain-text description |
| `epic` | no | Epic name or numeric Shortcut ID |
| `owners` | no | List or comma-separated member names |
| `team` | no | Group name |
| `labels` | no | List or comma-separated label names |
| `estimate` | no | Story points (integer) |
| `due_date` | no | ISO 8601 date |
| `workflow_state` | no | Workflow state name (defaults to first unstarted state) |

## Epic Templates

A Markdown template can be applied to every epic to generate a structured description. Specify it globally with `--template` or per-epic with the `template` field. A per-epic `template` overrides the global flag.

Supported variables:

| Variable | Value |
|----------|-------|
| `{{name}}` | Epic name |
| `{{description}}` | Epic description |
| `{{objective}}` | Objective name |
| `{{owners}}` | Comma-separated owner names |
| `{{teams}}` | Comma-separated team names |
| `{{labels}}` | Comma-separated label names |
| `{{start_date}}` | Start date |
| `{{deadline}}` | Deadline |

See [`examples/epic_template.md`](examples/epic_template.md) for a starter template.

## Cross-references

Epics and Stories can reference parent resources by **name** or **numeric Shortcut ID**. Name references resolve against resources created earlier in the same run, so a single YAML manifest with all three sections works end-to-end without pre-existing IDs.

```yaml
# Resolves against an objective created in the same run:
objective: "Q3 2024 – Platform Reliability"

# Or use an existing Shortcut ID directly:
objective: "42"
```

## Output

### Text (default)

Coloured progress output with per-resource status lines.

### JSON

`--output json` emits newline-delimited JSON records, suitable for piping:

```sh
bypass create --file manifest.yaml --output json | jq 'select(.event=="created") | .id'
```

Each line is one of the following event objects:

| `event` | Fields | When |
|---------|--------|------|
| `"created"` | `kind`, `id`, `name`, `url` | Resource created successfully |
| `"error"` | `kind`, `name`, `error` | Resource failed to create |
| `"summary"` | `objectives_created`, `epics_created`, `stories_created`, `error_count`, `errors` | End of run |
| `"dry_run"` | `valid`, `errors` | `--dry-run` result |

## Rate Limits

The Shortcut API allows 200 requests per minute. `bypass` automatically retries on 429 (rate-limited), 500, 503, and 504 responses using exponential backoff (1 s base, 30 s cap, up to 5 retries). On 429 responses the `Retry-After` header is honoured when present.

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | All resources created (or dry-run passed) |
| `1` | One or more errors occurred |
