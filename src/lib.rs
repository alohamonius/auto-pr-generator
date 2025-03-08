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
    github_token: String,
}

impl PrGenerator {
    pub fn new(template_path: String, exclude_patterns: Vec<String>, claude_api_key: String, github_token: String) -> Self {
        Self {
            template_path,
            exclude_patterns,
            claude_api_key,
            github_token,
        }
    }

    fn get_current_branch(&self) -> Result<String, Box<dyn Error>> {
        let output = Command::new("git")
            .args(&["rev-parse", "--abbrev-ref", "HEAD"])
            .output()?;
        
        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    }

    fn get_git_diff(&self, base_branch: &str, current_branch: &str) -> Result<String, Box<dyn Error>> {
        let diff_arg = format!("{}...{}", base_branch, current_branch);
        let mut args = vec!["diff", &diff_arg, "--name-only"];
        
        let output = Command::new("git")
            .args(&args)
            .output()?;
            
        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    }

    fn get_git_log(&self, base_branch: &str, current_branch: &str) -> Result<String, Box<dyn Error>> {
        let log_arg = format!("{}...{}", base_branch, current_branch);
        let output = Command::new("git")
            .args(&["log", &log_arg])
            .output()?;
            
        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    }

    async fn generate_code2prompt_output(&self, changed_files: &str) -> Result<String, Box<dyn Error>> {
        // Convert changed files to include patterns
        let include_patterns: Vec<String> = changed_files
            .split('\n')
            .filter(|f| !f.is_empty())
            .map(|f| format!("**/{}", f))
            .collect();

        // Run code2prompt command
        let include_arg = include_patterns.join(",");
        let output = Command::new("code2prompt")
            .args(&[".", "--include", &include_arg, "-t", &self.template_path])
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

    pub async fn generate_pr_description(&self, base_branch: &str) -> Result<String, Box<dyn Error>> {
        let current_branch = self.get_current_branch()?;
        
        // Get changed files
        let changed_files = self.get_git_diff(base_branch, &current_branch)?;
        
        // Generate code2prompt output
        let code_analysis = self.generate_code2prompt_output(&changed_files).await?;
        
        // Get git information
        let git_log = self.get_git_log(base_branch, &current_branch)?;

        // Prepare prompt for Claude
        let prompt = format!(
            "Please analyze this pull request and generate a comprehensive description.\n\n\
            Code Analysis:\n{}\n\n\
            Git Log:\n{}\n\n\
            Please format the response as a well-structured markdown document with:\n\
            1. A clear title\n\
            2. Summary of changes\n\
            3. Technical details\n\
            4. Impact and considerations",
            code_analysis, git_log
        );

        // Get response from Claude
        let pr_description = self.get_claude_response(&prompt).await?;
        
        // Save to file
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
    let github_token = std::env::var("GITHUB_TOKEN")?;

    let generator = PrGenerator::new(
        template_path.to_string(),
        exclude_patterns,
        claude_api_key,
        github_token,
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