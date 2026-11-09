//! Keeps a snapshot of the structure of the Picqer API documentation.
//!
//! `cargo xtask docs` writes the snapshots to `tests/docs`. `cargo xtask docs --check` fetches the
//! documentation again and fails when it no longer matches the snapshots.

mod extract;

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const BASE: &str = "https://picqer.com";
const START: &str = "/en/api";
const USER_AGENT: &str = "picqer-rs-xtask (github.com/robmeijerink/picqer-rs)";
const CHANGELOG: &str = "changelog";
const NOT_AN_ENDPOINT_PAGE: &[&str] = &["php-client", "use-cases"];

type Snapshots = BTreeMap<String, String>;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let check = match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["docs"] => false,
        ["docs", "--check"] => true,
        _ => {
            eprintln!("usage: cargo xtask docs [--check]");
            return ExitCode::from(2);
        }
    };
    match run(check) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(check: bool) -> Result<bool, Box<dyn Error>> {
    let snapshots = fetch_snapshots()?;
    let directory = snapshot_directory();
    if check {
        Ok(compare(&directory, &snapshots)?)
    } else {
        write(&directory, &snapshots)?;
        println!(
            "wrote {} snapshots to {}",
            snapshots.len(),
            directory.display()
        );
        Ok(true)
    }
}

fn snapshot_directory() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/docs")
}

fn fetch(path: &str) -> Result<String, Box<dyn Error>> {
    let url = format!("{BASE}{path}");
    let mut response = ureq::get(&url).header("User-Agent", USER_AGENT).call()?;
    Ok(response.body_mut().read_to_string()?)
}

fn fetch_snapshots() -> Result<Snapshots, Box<dyn Error>> {
    let mut snapshots = Snapshots::new();
    let mut seen = BTreeSet::new();
    let mut queue = VecDeque::from([START.to_owned()]);
    while let Some(path) = queue.pop_front() {
        if !seen.insert(path.clone()) {
            continue;
        }
        let html = fetch(&path)?;
        for link in extract::documentation_links(&html) {
            queue.push_back(link);
        }
        let name = path.rsplit('/').next().unwrap_or("introduction");
        let name = if path == START { "introduction" } else { name };
        if NOT_AN_ENDPOINT_PAGE.contains(&name) {
            continue;
        }
        let text = if name == CHANGELOG {
            render(&extract::changelog(&html))?
        } else {
            render(&extract::page(&html))?
        };
        snapshots.insert(format!("{name}.json"), text);
    }
    Ok(snapshots)
}

fn render(value: &impl serde::Serialize) -> Result<String, Box<dyn Error>> {
    let mut text = serde_json::to_string_pretty(value)?;
    text.push('\n');
    Ok(text)
}

fn write(directory: &Path, snapshots: &Snapshots) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(directory)?;
    for (name, text) in snapshots {
        fs::write(directory.join(name), text)?;
    }
    Ok(())
}

fn compare(directory: &Path, snapshots: &Snapshots) -> Result<bool, Box<dyn Error>> {
    let mut clean = true;
    let mut on_disk = BTreeSet::new();
    for entry in fs::read_dir(directory)? {
        let name = entry?.file_name().to_string_lossy().into_owned();
        if name.ends_with(".json") && name != "coverage.json" {
            on_disk.insert(name);
        }
    }
    for (name, text) in snapshots {
        match fs::read_to_string(directory.join(name)) {
            Ok(old) if &old == text => {}
            Ok(old) => {
                clean = false;
                println!("changed: {name}");
                print_difference(&old, text);
            }
            Err(_) => {
                clean = false;
                println!("new page: {name}");
            }
        }
        on_disk.remove(name);
    }
    for name in on_disk {
        clean = false;
        println!("page gone: {name}");
    }
    Ok(clean)
}

fn print_difference(old: &str, new: &str) {
    let old_lines: BTreeSet<&str> = old.lines().collect();
    let new_lines: BTreeSet<&str> = new.lines().collect();
    for line in old_lines.difference(&new_lines) {
        println!("  - {}", line.trim());
    }
    for line in new_lines.difference(&old_lines) {
        println!("  + {}", line.trim());
    }
}
