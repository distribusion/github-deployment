extern crate serde;
extern crate serde_json;
extern crate curl;
extern crate clap;

#[macro_use]
extern crate lazy_static;

use curl::easy::{Easy, List};
use serde_json::Value;
use clap::{App, Arg};

use std::fmt;
use std::env;
use std::process;

const GITHUB_API_TOKEN: &'static str = env!("GITHUB_API_TOKEN");
const DEBUG_FLAG: &'static str = "DEBUG";
const REVISION: &'static str = env!("REVISION");
const VERSION: &'static str = "1.0";

lazy_static! {
    static ref LONG_VERSION: String = format!("{} [{}]", VERSION, REVISION);
}

macro_rules! debug {
    ($($arg:tt)*) => (
        match env::var(DEBUG_FLAG) {
            Ok(flag) => { if flag == "true" { println!($($arg)*) } },
            Err(_) => { panic!("Unable to fetch env var {}", DEBUG_FLAG) }
        }
    )
}

#[derive(PartialEq, Eq, Debug)]
enum Status {
    Unknown,
    Pending,
    Error,
    Success,
    Failure
}

impl<'a> From<&'a str> for Status {
    fn from(origin: &str) -> Self {
        match origin {
            "pending" => Status::Pending,
            "error" => Status::Error,
            "success" => Status::Success,
            "failure" => Status::Failure,
            _ => Status::Unknown
        }
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Status::Unknown => write!(f, "unknown"),
            &Status::Pending => write!(f, "pending"),
            &Status::Error => write!(f, "error"),
            &Status::Success => write!(f, "success"),
            &Status::Failure => write!(f, "failure")
        }
    }
}

#[derive(Debug)]
struct Deployment<'a> {
    id: u64,
    repo: &'a str,
    head: &'a str,
    base: Option<&'a str>
}

impl<'a> Deployment<'a> {
    pub fn payload(head: &'a str, base: Option<&'a str>) -> String {
        let previous_ref = if base.is_some() {
            format!(r#"
              "previous_ref": "{}"
            "#, base.unwrap())
        } else {
            String::new()
        };

        format!(r#"{{
          "ref": "{}",
          "auto_merge": false,
          "environment": "production",
          "payload": {{
            {}
          }}
        }}"#, head, previous_ref)
    }

    pub fn create(repo: &'a str, head: &'a str, base: Option<&'a str>) -> Result<Self, &'static str> {
        let url = &format!("https://api.github.com/repos/{}/deployments", repo);
        let payload = &Deployment::payload(head, base);

        debug!("[DEBUG] Deployment create payload: {}", payload);

        let response = post(url, payload)?;

        debug!("[DEBUG] Deployment create response: {}", response);

        let json: Value = serde_json::from_str(&response)
            .map_err(|_| "payload is not a valid json" )?;
        let id = json.as_object().and_then(|o| { o["id"].as_u64() })
            .ok_or("json attribute 'id' is missing")?;

        Ok(Deployment { id: id, repo: repo, head: head, base: base })
    }

    pub fn update_status(&self, status: &Status) -> Result<(), &'static str> {
        let payload = &format!(r#"{{"state": "{}"}}"#, status);
        let url = &format!("https://api.github.com/repos/{}/deployments/{}/statuses",
                           self.repo, self.id);

        debug!("[DEBUG] Deployment status update payload: {}", payload);

        let response = post(url, payload)?;

        debug!("[DEBUG] Deployment status update response: {}", response);

        let json: Value = serde_json::from_str(&response)
            .map_err(|_| "payload is not a valid json" )?;

        json.as_object()
            .and_then(|o| { o["state"].as_str() })
            .and_then(|s| { if *status == Status::from(s) { Some(()) } else { None } })
            .ok_or("deployment update status do not change")
    }
}

fn post(url: &str, payload: &str) -> Result<String, &'static str> {
    let mut buffer = Vec::new();
    let mut easy = Easy::new();
    let mut headers = List::new();
    let success_response_codes = &[200, 201];

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

    let response_code = &easy.response_code().unwrap_or(500);
    if !success_response_codes.iter().any(|code| { code == response_code }) {
        return Err("http response code not in 200 or 201")
    }

    debug!("[DEBUG] POST {}: ({}) {:?}",
           url, response_code, String::from_utf8(buffer.clone()));

    String::from_utf8(buffer).map_err(|_| { "response can not be transformed to String" })
}

fn cli<'a, 'b>() -> App<'a, 'b> {
    App::new("Github Deployment")
        .version(VERSION)
        .long_version(LONG_VERSION.as_ref())
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
            Arg::with_name("quiet").required(false).short("q").long("quiet")
                .help("Exit without a failure even if shit happened")
        ).arg(
            Arg::with_name("debug").required(false).short("d").long("debug")
                .conflicts_with("quiet")
                .help("Show the debug messages")
        ).arg(
            Arg::with_name("repo").required(true)
                .help("A Github repository path as <owner>/<repo>")
        )
}

fn main () {
    let args = cli().get_matches();
    let repo = args.value_of("repo").unwrap();
    let head = args.value_of("head").unwrap();
    let base = args.value_of("base");

    env::set_var(DEBUG_FLAG, &format!("{}", args.is_present("debug")));

    let deployment = Deployment::create(repo, head, base);

    debug!("[DEBUG] Deployment: {:?}", deployment);

    if deployment.is_err() {
        eprintln!("[ERROR] Failed to create deployment: {}", deployment.unwrap_err());
        if args.is_present("quiet") { process::exit(0) } else { process::exit(1) }
    }

    let status_update = deployment.unwrap()
        .update_status(&Status::from(args.value_of("status").unwrap()));
    if status_update.is_err() {
        eprintln!("[ERROR] Failed to update deployment status: {}", status_update.unwrap_err());
        if args.is_present("quiet") { process::exit(0) } else { process::exit(1) }
    }
}
