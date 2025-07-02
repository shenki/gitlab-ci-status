# GitLab CI Status Tool

A command-line tool to check the CI status of your current branch on GitLab.

## Features

- Reads GitLab configuration from `.git/config`
- Displays colored status output (green/red/yellow)
- Shows both pipeline and job status
- Supports all GitLab CI statuses (success, failed, running, pending, etc.)

## Setup

1. Add your GitLab configuration to `.git/config`:

```ini
[gitlab]
    server = https://gitlab.com
    access-token = your-gitlab-access-token
    project-name = your-username/your-project
```

2. Build the tool:

```bash
cargo build --release
```

3. Run the tool:

```bash
cargo run
```

## Configuration

The tool expects the following configuration in your `.git/config` file:

- `gitlab.server`: Your GitLab server URL (e.g., `https://gitlab.com`)
- `gitlab.access-token`: Your GitLab personal access token
- `gitlab.project-name`: Your project name in the format `username/project` or `group/project`

## Output

The tool will display:
- Current branch name
- Latest pipeline ID and status
- Individual job statuses with their stages

Status colors:
- 🟢 Green: Success
- 🔴 Red: Failed
- 🟡 Yellow: Building/Running/Pending
- ⚪ White: Canceled
- 🔵 Blue: Skipped

## Requirements

- Rust 1.70+
- Git repository with proper GitLab configuration
- GitLab personal access token with API access 