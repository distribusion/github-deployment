extern crate serde;
extern crate serde_json;
extern crate curl;
extern crate clap;

use curl::easy::{Easy, List};
use serde_json::Value;
use clap::{App, Arg};

use std::fmt;

const GITHUB_API_TOKEN: &'static str = env!("GITHUB_API_TOKEN");

#[derive(PartialOrd, Ord, PartialEq, Eq, Debug)]
enum DeploymentStatus {
    Unknown,
    Pending,
    Error,
    Success,
    Failure
}

impl<'a> From<&'a str> for DeploymentStatus {
    fn from(origin: &str) -> Self {
        match origin {
            "pending" => DeploymentStatus::Pending,
            "error" => DeploymentStatus::Error,
            "success" => DeploymentStatus::Success,
            "failure" => DeploymentStatus::Failure,
            _ => DeploymentStatus::Unknown
        }
    }
}

impl fmt::Display for DeploymentStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &DeploymentStatus::Unknown => write!(f, "unknown"),
            &DeploymentStatus::Pending => write!(f, "pending"),
            &DeploymentStatus::Error => write!(f, "error"),
            &DeploymentStatus::Success => write!(f, "success"),
            &DeploymentStatus::Failure => write!(f, "failure")
        }
    }
}

struct Deployment<'a> {
    id: Option<u64>,
    repo: &'a str,
    head: &'a str,
    base: Option<&'a str>
}

impl<'a> Deployment<'a> {
    pub fn create(repo: &'a str, head: &'a str, base: Option<&'a str>) -> Self {
        let url = &format!("https://api.github.com/repos/{}/deployments", repo);

        // TODO: Add payload in case of `base` given
        let payload = &format!(r#"{{
"ref": "{}",
"auto_merge": false,
"environment": "production"
}}"#, head);

        let response = post(url, payload).or_else(|e| {
            eprintln!("Failed to create new deployment: {:?}", e);
            Err(e)
        }).unwrap_or(String::from("{}"));

        let json = serde_json::from_str(&response).unwrap_or(Value::Null);
        let id = json.as_object().and_then(|o| { o["id"].as_u64() });

        Deployment { id: id, repo: repo, head: head, base: base }
    }

    pub fn update_status(&self, status: &DeploymentStatus) -> Result<(), &'static str> {
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
            .and_then(|s| {
                if *status == DeploymentStatus::from(s) {
                    Ok(())
                } else {
                    Err("Status didn't change after update")
                }
            })
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

fn main () {
    let matches = App::new("Github Deployment")
        .version("1.0")
        .author("Fedorov Sergey <sergey.fedorov@distribusion.com>")
        .about("Create a new Github deployment and set its status")
        .arg(
            Arg::with_name("head").required(true).takes_value(true).long("head")
                .help("The ref to deploy. This can be a branch, tag, or SHA")
        ).arg(
            Arg::with_name("base").required(false).takes_value(true).long("base").
                help("The ref of previous deployment. This can be a branch, tag, or SHA")

        ).arg(
            Arg::with_name("status").required(true).takes_value(true).long("status")
                .possible_values(&["pending", "error", "success", "failure"]) // FIXME: Take from enum
                .default_value("pending") // FIXME: Take from enum
                .help("A deployment status to be set")
        ).arg(
            Arg::with_name("repo").required(true)
                .help("A Github repository path as <owner>/<repo>")
        ).get_matches();

    let repo = matches.value_of("repo").unwrap();
    let head = matches.value_of("head").unwrap();
    let status = matches.value_of("status").unwrap();
    let base = matches.value_of("base");

    let deployment = Deployment::create(repo, head, base);
    let deployment_status = DeploymentStatus::from(status);

    if let Err(reason) = deployment.update_status(&deployment_status) {
        eprintln!("Status update of deployment#{} to '{}' failed\nReason: {}",
                  deployment.id.unwrap_or(0), deployment_status, reason);
    } else {
        println!("Status update of deployment#{} to '{}' succeed",
                 deployment.id.unwrap_or(0), deployment_status);

    }
}
