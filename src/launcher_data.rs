use std::path::PathBuf;
use std::error::Error;
use chrono::{DateTime, Duration, Utc};
use fuzzy_matcher::skim::SkimMatcherV2;
use regex::Regex;
use serde::{Deserialize, Serialize};
use strsim::damerau_levenshtein;
use std::fs;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::process::Stdio;
use std::process::Command;
use std::os::unix::process::CommandExt;

use crate::log_to_file;
use fuzzy_matcher::FuzzyMatcher;

#[derive(Debug, Clone)]
pub struct AppEntry {
    pub name: String,
    pub exec: String,
    pub icon_path: Option<PathBuf>,
    pub comment: String,
    pub terminal: bool,
    pub appid: String
}

/* pub fn get_sample () -> Vec<AppEntry> {
    Vec::from(vec![
        AppEntry {
            name: "Sample App".into(),
            exec: "sample-app".into(),
            icon_path: None,
            comment: "This is a sample app entry.".into(),
            terminal: false
        },
        AppEntry {
            name: "Sample App 2".into(),
            exec: "sample-app-2".into(),
            icon_path: None,
            comment: "This is a sample app entry 2.".into(),
            terminal: false
        },
        AppEntry {
            name: "Sample App 3".into(),
            exec: "sample-app-3".into(),
            icon_path: None,
            comment: "This is a sample app entry 3.".into(),
            terminal: false
        }
    ])
} */


use freedesktop_desktop_entry::{default_paths, get_languages_from_env, Iter};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LaunchStats {
    pub total_launches: u64,
    pub launches: Vec<String>,
}

pub type LaunchHistory = HashMap<String, LaunchStats>;

fn app_key(app: &AppEntry) -> String {
    format!("{}::{}", app.exec, app.name)
}

pub fn record_app_launch(app: &AppEntry, history: &mut LaunchHistory, at: DateTime<Utc>) {
    let key = app_key(app);
    let stats = history.entry(key).or_default();
    stats.total_launches += 1;
    stats.launches.push(at.to_rfc3339());
}

pub fn prune_launch_history(history: &mut LaunchHistory, max_age: Duration) {
    let cutoff = Utc::now() - max_age;
    for stats in history.values_mut() {
        stats.launches.retain(|ts| {
            DateTime::parse_from_rfc3339(ts).ok().map(|parsed| parsed.with_timezone(&Utc) >= cutoff).unwrap_or(false)
        });
        stats.total_launches = stats.launches.len() as u64;
    }
    history.retain(|_, stats| !stats.launches.is_empty());
}

pub fn sort_app_entries_by_launch_history(
    entries: &[AppEntry],
    history: &LaunchHistory,
    recent_window_days: i64,
) -> Vec<AppEntry> {
    let cutoff = Utc::now() - Duration::days(recent_window_days);
    let mut ranked: Vec<(f64, AppEntry)> = entries
        .iter()
        .cloned()
        .map(|entry| {
            let key = app_key(&entry);
            let stats = history.get(&key).cloned().unwrap_or_default();
            let recent_hits = stats
                .launches
                .iter()
                .filter(|ts| {
                    DateTime::parse_from_rfc3339(ts).ok().map(|parsed| parsed.with_timezone(&Utc) >= cutoff).unwrap_or(false)
                })
                .count() as f64;
            let score = recent_hits * 10.0 + stats.total_launches as f64;
            (score, entry)
        })
        .collect();

    ranked.sort_by(|left, right| {
        right.0.partial_cmp(&left.0).unwrap_or(std::cmp::Ordering::Equal)
    });

    ranked.into_iter().map(|(_, entry)| entry).collect()
}

pub fn save_launch_history(history: &LaunchHistory, path: &PathBuf) -> Result<(), Box<dyn Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temp = path.with_extension("json.tmp");
    let data = serde_json::to_string_pretty(history)?;
    fs::write(&temp, data)?;
    fs::rename(temp, path)?;
    Ok(())
}

pub fn load_launch_history(path: &PathBuf) -> Result<LaunchHistory, Box<dyn Error>> {
    if !path.exists() {
        return Ok(LaunchHistory::new());
    }
    let data = fs::read_to_string(path)?;
    if data.trim().is_empty() {
        return Ok(LaunchHistory::new());
    }
    Ok(serde_json::from_str(&data)?)
}

pub fn load_app_entries() -> Result<Vec<AppEntry>, Box<dyn Error>> {

    let mut results = vec![];
    let locales = get_languages_from_env();

    let entries = Iter::new(default_paths())
        .entries(Some(&locales))
        .collect::<Vec<_>>();
    
    for entry in entries {
        // let path_src = PathSource::guess_from(&entry.path);
        // println!("{:?}", entry);
        if (&entry).no_display() == false {
            results.push(AppEntry {
                exec: (&entry).exec().unwrap_or_default().to_string(),
                name: (&entry).name(&vec!["en"]).as_ref().unwrap().to_string(),
                icon_path: resolve_icon_path((&entry).icon().unwrap_or_default().to_string()),
                comment: (&entry).comment(&vec!["en"]).unwrap_or_default().to_string(),
                terminal: (&entry).terminal(),
                appid: (&entry).appid.to_string()
            });
        }
    }

    Ok(results)
}

fn resolve_icon_path(icon_name: String) -> Option<PathBuf> {
    let candidates = vec![
        format!("/usr/share/icons/hicolor/48x48/apps/{}.png", icon_name),
        format!("/usr/share/pixmaps/{}.png", icon_name)
    ];

    candidates.into_iter().map(PathBuf::from).find(|p| p.exists())
}

pub fn get_apps (apps_entries: &Vec<AppEntry>, filter: &String) -> Vec<AppEntry> {
    // TODO: Stuff to be cached...

    /* let mut apps_entries: Vec<AppEntry> = vec![];
    if apps_entries.len() == 0 {
        apps_entries = load_app_entries().unwrap_or_default();
    } */

    let uri = if let Ok(path) = std::env::var("XDG_STATE_HOME") { path } else { shellexpand::tilde("~/.local/state/fenrir/history.json").to_string() };
    log_to_file(format!("History path {uri}"));
    let history_path = PathBuf::from(uri); // "/tmp/fenrir-launch-history.json");
    let launch_history: LaunchHistory = load_launch_history(&history_path).unwrap_or_default();
    let matcher = SkimMatcherV2::default();


    let filtered: Vec<_> = {
        let q = filter.to_lowercase();
        let sorted = sort_app_entries_by_launch_history(&apps_entries, &launch_history, 30);
        if q.is_empty() {
            sorted.into_iter().collect()
        } else {
            let mut exact: Vec<AppEntry> = Vec::new();
            let mut fuzzy: Vec<AppEntry> = Vec::new();
            for app in sorted.into_iter() {
                let name = app.name.to_lowercase();
                if name.starts_with(&q) {
                    exact.push(app);
                    continue;
                }
                if matcher.fuzzy_match(&name, &q).is_some()
                    || name.split_whitespace().any(|tok| damerau_levenshtein(tok, &q) <= 2)
                {
                    fuzzy.push(app);
                }
            }
            exact.extend(fuzzy);
            exact
        }
    };
    // eprintln!("\n\nFiltered {} apps with query '{}'\n\n", filtered.len(), filter);
    filtered
}

pub fn update_launch_history (app: &AppEntry) {

    // TODO: improve history management in RAM
    let uri = if let Ok(path) = std::env::var("XDG_STATE_HOME") { path } else { shellexpand::tilde("~/.local/state/fenrir/history.json").to_string() };
    log_to_file(format!("History path {uri}"));
    let history_path = PathBuf::from(uri); // "/tmp/fenrir-launch-history.json");

    let mut launch_history: LaunchHistory = load_launch_history(&history_path).unwrap_or_default();
    record_app_launch(app, &mut launch_history, chrono::Utc::now());
    prune_launch_history(&mut launch_history, Duration::days(30));
    let _ = save_launch_history(&launch_history, &history_path);
}


// use regex::Regex;
// use std::fs::OpenOptions;
// use std::os::unix::process::CommandExt;
// use std::process::{Command, Stdio};
use std::io;

pub fn launch_detached(exec_str: &str) -> io::Result<()> {
    // 1. Clean FreeDesktop percent codes and escaped %%
    let placeholder = "\u{0000}";
    let sanitized = exec_str.replace("%%", placeholder);
    
    let re_fields = Regex::new(r"%[UufFdDnNickvm]").unwrap();
    let cleaned = re_fields.replace_all(&sanitized, "");
    let cleaned = cleaned.replace(placeholder, "%");

    let args = shell_words::split(&cleaned)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, format!("Parsing error: {}", e)))?;

    if args.is_empty() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Empty Exec command"));
    }

    let program = &args[0];
    let program_args = &args[1..];

    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/fenrir.log")
        .or_else(|_| OpenOptions::new().write(true).open("/dev/null"))?;

    let stderr_file = log_file.try_clone()?;

    let mut cmd = Command::new(program);
    cmd.args(program_args)
       .stdin(Stdio::null())
       .stdout(Stdio::from(log_file))
       .stderr(Stdio::from(stderr_file));

    // 2. Simple single setsid call to create a new session. 
    // Dropping the spawned child handle without waiting leaves it running detached.
    unsafe {
        cmd.pre_exec(|| {
            libc::setsid();
            Ok(())
        });
    }

    cmd.spawn()?;
    Ok(())
}