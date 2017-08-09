extern crate serde;
extern crate serde_json;
extern crate curl;
extern crate clap;

use curl::easy::{Easy, List};
use serde_json::Value;
use clap::App;

use std::fmt;
use std::process::{self, ExitStatus, Command, Stdio};
use std::os::unix::process::ExitStatusExt;

const GITHUB_API_TOKEN: &'static str = env!("GITHUB_API_TOKEN");

enum DeploymentStatus {
    Pending,
    Success,
    Error,
    Failure
}

impl fmt::Display for DeploymentStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &DeploymentStatus::Pending => write!(f, "pending"),
            &DeploymentStatus::Success => write!(f, "success"),
            &DeploymentStatus::Error => write!(f, "error"),
            &DeploymentStatus::Failure => write!(f, "failure")
        }
    }
}

struct Arguments<'a> {
    repo: &'a str,
    sha: &'a str,
    command: &'a str
}

struct Deployment<'a> {
    id: Option<u64>,
    repo: &'a str,
    sha: &'a str
}

impl<'a> Deployment<'a> {
    pub fn create(repo: &'a str, sha: &'a str) -> Self {
        let url = &format!("https://api.github.com/repos/{}/deployments", repo);
        let payload = &format!(r#"{{
"ref": "{}",
"auto_merge": false,
"environment": "production"
}}"#, sha);

        let response = post(url, payload).or_else(|e| {
            eprintln!("Failed to create new deployment: {:?}", e);
            Err(e)
        }).unwrap_or(String::from("{}"));

        let json = serde_json::from_str(&response).unwrap_or(Value::Null);
        let id = json.as_object().and_then(|o| { o["id"].as_u64() });

        Deployment { id: id, repo: repo, sha: sha }
    }

    pub fn update_status(&self, status: DeploymentStatus) -> Result<bool, &'static str> {
        if self.id.is_none() { return Err("Unknown deployment ID") }

        let url = &format!(
            "https://api.github.com/repos/{}/deployments/{}/statuses",
            self.repo, self.id.unwrap()
        );
        let payload = &format!(r#"{{"state": "{}"}}"#, status);

        let response = post(url, payload).or_else(|e| {
            eprintln!("Failed to create new deployment: {:?}", e);
            Err(e)
        }).unwrap_or(String::from("{}"));

        let json = serde_json::from_str(&response).unwrap_or(Value::Null);
        json.as_object().and_then(|o| { o["state"].as_str() })
            .ok_or("Failed find deployment status in response")
            .map(|s| { s == status.to_string().as_str() })
    }
}


pub fn post(url: &str, payload: &str) -> Result<String, &'static str> {
    let mut buffer = Vec::new();
    let mut easy = Easy::new();
    let mut headers = List::new();

    headers.append("User-Agent: github-deployment").unwrap();
    headers.append("Accept: application/vnd.github.v3+json").unwrap();
    headers.append("Content-Type: application/json").unwrap();
    headers.append(&format!("Authorization: token {}", GITHUB_API_TOKEN)).unwrap();

    easy.url(url).unwrap();
    easy.post(true).unwrap();
    easy.http_headers(headers).unwrap();
    easy.post_fields_copy(payload.as_bytes()).unwrap();

    {
        let mut transfer = easy.transfer();
        transfer.write_function(|data| {
            buffer.extend_from_slice(data);
            Ok(data.len())
        }).unwrap();
        transfer.perform().unwrap();
    }

    let empty_json_result: Result<String, &'static str> = Ok(String::from("{}"));
    String::from_utf8(buffer).or(empty_json_result)
}

fn run_command_and_exit(arguments: Arguments) {
    let deployment = Deployment::create(arguments.repo, arguments.sha);

    deployment.update_status(DeploymentStatus::Pending).unwrap_or_else(|e| {
        eprintln!("Failed to update status [{}]: {}", DeploymentStatus::Pending, e);
        false
    });

    let mut process = Command::new("bash")
        .arg("-l")
        .arg("-c")
        .arg(arguments.command)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("Spawning command is failed to start");

    let exit_status = process.wait().unwrap_or_else(|_| {
        deployment.update_status(DeploymentStatus::Error).unwrap_or_else(|e| {
            eprintln!("Failed to update status [{}]: {}", DeploymentStatus::Error, e);
            false
        });
        ExitStatus::from_raw(1)
    });

    if exit_status.success() {
        deployment.update_status(DeploymentStatus::Success).unwrap_or_else(|e| {
            eprintln!("Failed to update status [{}]: {}", DeploymentStatus::Success, e);
            false
        });
    } else {
        deployment.update_status(DeploymentStatus::Failure).unwrap_or_else(|e| {
            eprintln!("Failed to update status [{}]: {}", DeploymentStatus::Failure, e);
            false
        });
    }

    process::exit(match exit_status.code() {
        Some(x) => x,
        None => { eprintln!("Failed to get exit status of subprocess"); 1 }
    })
}

fn main () {
    let matches = App::new("Github Deployment")
        .version("1.0")
        .author("Fedorov Sergey <sergey.fedorov@distribusion.com>")
        .about("Wrap any command with github deployment updates")
        .args_from_usage(
            "-p --repo=[STRING] 'Github repository path as <owner>/<repo>'
             -r --ref=[STRING] 'SHA hash of the deployed code'
             -c --command=[STRING] 'Command to execute in /bin/bash -l'").get_matches();

    let repo = matches.value_of("repo").expect("Please provide a github repo like <owner>/<repo>");
    let sha = matches.value_of("ref").expect("Please provide a sha hash of the code");
    let command = matches.value_of("command").expect("Please provide a command to execute");

    run_command_and_exit(Arguments { command: command, sha: sha, repo: repo })
}
