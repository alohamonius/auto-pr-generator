use std::error::Error;
use std::process::Command;
use handlebars::Handlebars;
use serde_json::json;
use std::fs;
use reqwest::Client;
use octocrab::Octocrab;
use serde::{Deserialize, Serialize};
use dotenv::dotenv;

const CLAUDE_API_ENDPOINT: &str = "https://api.anthropic.com/v1/messages";

#[derive(Debug, Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<Message>,
}

#[derive(Debug, Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ClaudeResponse {
    content: Vec<Content>,
}

#[derive(Debug, Deserialize)]
struct Content {
    text: String,
}

pub struct PrGenerator {
    template_path: String,
    exclude_patterns: Vec<String>,
    claude_api_key: String,
    TOKEN_GITHUB: String,
}

impl PrGenerator {
    pub fn new(template_path: String, exclude_patterns: Vec<String>, claude_api_key: String, TOKEN_GITHUB: String) -> Self {
        Self {
            template_path,
            exclude_patterns,
            claude_api_key,
            TOKEN_GITHUB,
        }
    }

    fn get_current_branch(&self) -> Result<String, Box<dyn Error>> {
        let output = Command::new("git")
            .args(&["rev-parse", "--abbrev-ref", "HEAD"])
            .output()?;
        
        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    }

    fn get_changed_files(&self, base_branch: &str, current_branch: &str) -> Result<Vec<String>, Box<dyn Error>> {
        let output = Command::new("git")
            .args(&["diff", "--name-only", base_branch, current_branch])
            .output()?;
        
        let files = String::from_utf8(output.stdout)?
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(String::from)
            .collect();
        
        Ok(files)
    }

    fn get_git_diff(&self, base_branch: &str, current_branch: &str) -> Result<String, Box<dyn Error>> {
        // This gets the diff between the base branch and your current branch
        let output = Command::new("git")
            .args(&["diff", base_branch, current_branch])
            .output()?;
            
        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    }
    
    fn get_git_log(&self, base_branch: &str, current_branch: &str) -> Result<String, Box<dyn Error>> {
        // This gets all commits from where your branch diverged
        let output = Command::new("git")
            .args(&["log", &format!("{}..{}", base_branch, current_branch)])
            .output()?;
            
        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    }

    async fn get_claude_response(&self, prompt: &str) -> Result<String, Box<dyn Error>> {
        let client = Client::new();
        
        let request = ClaudeRequest {
            model: "claude-3-opus-20240229".to_string(),
            max_tokens: 4000,
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
        };

        let response = client
            .post(CLAUDE_API_ENDPOINT)
            .header("x-api-key", &self.claude_api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&request)
            .send()
            .await?
            .json::<ClaudeResponse>()
            .await?;

        Ok(response.content[0].text.clone())
    }

    fn get_git_stats(&self, base_branch: &str, current_branch: &str) -> Result<serde_json::Value, Box<dyn Error>> {
        // Get first commit date
        let first_commit = Command::new("git")
            .args(&["log", "--reverse", "--format=%at", &format!("{}..{}", base_branch, current_branch)])
            .output()?;
        let first_commit_output = String::from_utf8(first_commit.stdout)?;
        let first_date = first_commit_output.lines().next().unwrap_or("0");
        
        // Get last commit date
        let last_commit = Command::new("git")
            .args(&["log", "-1", "--format=%at", current_branch])
            .output()?;
        let last_commit_output = String::from_utf8(last_commit.stdout)?;
        let last_date = last_commit_output.lines().next().unwrap_or("0");
        
        // Get commit count
        let commit_count = Command::new("git")
            .args(&["rev-list", "--count", &format!("{}..{}", base_branch, current_branch)])
            .output()?;
        let count_output = String::from_utf8(commit_count.stdout)?;
        let count = count_output.trim().to_string();
        
        // Get contributors with commit counts
        let contributors = Command::new("git")
            .args(&["shortlog", "-sn", "--no-merges", &format!("{}..{}", base_branch, current_branch)])
            .output()?;
        let contribs_output = String::from_utf8(contributors.stdout)?;
        let contribs = contribs_output.trim().to_string();
        
        let hours = (last_date.parse::<i64>()? - first_date.parse::<i64>()?) / 3600;
        let files_changed = self.get_changed_files(base_branch, current_branch)?.len();

        Ok(json!({
            "duration": hours,
            "commit_count": count,
            "contributors": contribs,
            "code_velocity": files_changed as f64 / if hours > 0 { hours as f64 } else { 1.0 }
        }))
    }

    pub async fn generate_pr_description(&self, base_branch: &str) -> Result<String, Box<dyn Error>> {
    let current_branch = self.get_current_branch()?;
    
    // Get source tree
    let output = Command::new("tree")
        .args(&["-L", "2", "--noreport", "--charset", "ascii"])
        .output()?;
    let source_tree = String::from_utf8(output.stdout)?.trim().to_string();
    
    // Get git data
    let git_diff = self.get_git_diff(base_branch, &current_branch)?;
    let git_log = self.get_git_log(base_branch, &current_branch)?;
    let files = self.get_changed_files(base_branch, &current_branch)?;
    let git_stats = self.get_git_stats(base_branch, &current_branch)?;
    
    // Use handlebars to fill template
    let mut handlebars = Handlebars::new();
    handlebars.register_template_file("pr_template", &self.template_path)?;
    
    let data = json!({
        "absolute_code_path": std::env::current_dir()?.display().to_string(),
        "source_tree": source_tree,
        "git_diff_branch": git_diff,
        "git_log_branch": git_log,
        "git_changed_files": files,
        "git_stats": git_stats
    });
    
    let template_output = handlebars.render("pr_template", &data)?;
    
    // Get response from Claude
    let pr_description = self.get_claude_response(&template_output).await?;
    
    fs::write("template_output.md", &template_output)?;
    fs::write("pr_prompt.md", &pr_description)?;
    
    Ok(pr_description)
}
}

pub async fn run() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let template_path = "pr_template.hbs";
    let exclude_patterns = vec![
        String::from("**/dist/**"),
        String::from("**/node_modules/**"),
        String::from("**/scripts/**"),
        String::from("**/package-lock.json"),
        String::from("**/lib/**"),
    ];

    // Get API keys from environment
    let claude_api_key = std::env::var("CLAUDE_API_KEY")?;
    let TOKEN_GITHUB = std::env::var("TOKEN_GITHUB")?;

    let generator = PrGenerator::new(
        template_path.to_string(),
        exclude_patterns,
        claude_api_key,
        TOKEN_GITHUB,
    );

    let pr_description = generator.generate_pr_description("main").await?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_run() {
        assert!(run().await.is_ok());
    }
}