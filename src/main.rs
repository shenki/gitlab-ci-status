/*
 * gitlab-ci-status - GitLab CI Status Tool
 *
 * Copyright (C) 2024 Tenstorrent AI LLC
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use anyhow::{Context, Result};
use colored::*;
use git2::Repository;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct GitLabConfig {
    server: String,
    access_token: String,
    project_name: String,
}

#[derive(Debug, Deserialize)]
struct Pipeline {
    id: u64,
    status: String,
    ref_field: String,
    #[serde(rename = "ref")]
    ref_name: String,
}

#[derive(Debug, Deserialize)]
struct Job {
    id: u64,
    status: String,
    name: String,
    stage: String,
}

async fn get_git_config() -> Result<GitLabConfig> {
    let repo = Repository::open(".").context("Failed to open git repository")?;
    let config = repo.config().context("Failed to get git config")?;
    
    let server = config
        .get_string("gitlab.server")
        .context("gitlab.server not found in .git/config")?;
    
    let access_token = config
        .get_string("gitlab.access-token")
        .context("gitlab.access-token not found in .git/config")?;
    
    let project_name = config
        .get_string("gitlab.project-name")
        .context("gitlab.project-name not found in .git/config")?;
    
    Ok(GitLabConfig {
        server,
        access_token,
        project_name,
    })
}

async fn get_current_branch() -> Result<String> {
    let repo = Repository::open(".").context("Failed to open git repository")?;
    let head = repo.head().context("Failed to get HEAD")?;
    let branch_name = head
        .shorthand()
        .context("Failed to get branch name")?
        .to_string();
    Ok(branch_name)
}

async fn get_pipeline_status(config: &GitLabConfig, branch: &str) -> Result<Vec<Pipeline>> {
    let client = Client::new();
    let url = format!(
        "{}/api/v4/projects/{}/pipelines?ref={}",
        config.server.trim_end_matches('/'),
        urlencoding::encode(&config.project_name),
        branch
    );
    
    let response = client
        .get(&url)
        .header("PRIVATE-TOKEN", &config.access_token)
        .send()
        .await
        .context("Failed to send request to GitLab API")?;
    
    if !response.status().is_success() {
        anyhow::bail!("GitLab API request failed: {}", response.status());
    }
    
    let pipelines: Vec<Pipeline> = response
        .json()
        .await
        .context("Failed to parse pipeline response")?;
    
    Ok(pipelines)
}

async fn get_jobs(config: &GitLabConfig, pipeline_id: u64) -> Result<Vec<Job>> {
    let client = Client::new();
    let url = format!(
        "{}/api/v4/projects/{}/pipelines/{}/jobs",
        config.server.trim_end_matches('/'),
        urlencoding::encode(&config.project_name),
        pipeline_id
    );
    
    let response = client
        .get(&url)
        .header("PRIVATE-TOKEN", &config.access_token)
        .send()
        .await
        .context("Failed to send request to GitLab API")?;
    
    if !response.status().is_success() {
        anyhow::bail!("GitLab API request failed: {}", response.status());
    }
    
    let jobs: Vec<Job> = response
        .json()
        .await
        .context("Failed to parse jobs response")?;
    
    Ok(jobs)
}

fn display_status(status: &str) {
    match status.to_lowercase().as_str() {
        "success" => println!("{}", "● SUCCESS".green().bold()),
        "failed" => println!("{}", "● FAILED".red().bold()),
        "running" | "pending" => println!("{}", "● BUILDING".yellow().bold()),
        "canceled" => println!("{}", "● CANCELED".white().bold()),
        "skipped" => println!("{}", "● SKIPPED".blue().bold()),
        _ => println!("{}", format!("● {}", status.to_uppercase()).white().bold()),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Check if we're in a git repository
    if !Path::new(".git").exists() {
        eprintln!("{}", "Error: Not in a git repository".red());
        std::process::exit(1);
    }
    
    // Get GitLab configuration from .git/config
    let config = get_git_config().await?;
    
    // Get current branch
    let branch = get_current_branch().await?;
    println!("Branch: {}", branch.cyan());
    
    // Get pipeline status
    let pipelines = get_pipeline_status(&config, &branch).await?;
    
    if pipelines.is_empty() {
        println!("{}", "No pipelines found for this branch".yellow());
        return Ok(());
    }
    
    // Get the latest pipeline
    let latest_pipeline = &pipelines[0];
    println!("Pipeline ID: {}", latest_pipeline.id);
    println!("Status: ");
    display_status(&latest_pipeline.status);
    
    // Get jobs for the latest pipeline
    let jobs = get_jobs(&config, latest_pipeline.id).await?;
    
    if !jobs.is_empty() {
        println!("\nJobs:");
        for job in jobs {
            print!("  {} ({}) - ", job.name, job.stage);
            display_status(&job.status);
        }
    }
    
    Ok(())
}
