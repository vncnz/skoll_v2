use serde_derive::Deserialize;
use std::process::Command;

use crate::time_of_scope::TimeOfScope;

#[derive(Deserialize,Debug,Clone)]
pub struct NiriWindow {
    pub id: u32,
    pub title: Option<String>,
    pub app_id: String,
    pub workspace_id: u8,
    pub is_focused: bool,
    pub is_urgent: bool
}

/* #[derive(Deserialize,Debug)]
pub struct NiriWorkspace {
    pub id: u8,
    pub idx: u32,
    pub name: Option<String>,
    pub output: String,
    pub is_active: bool,
    pub is_focused: bool,
    pub active_window_id: Option<u32>
} */

#[derive(Deserialize)]
struct FocusedOutput {
    name: String,
}

pub fn get_focused_output_name() -> Option<String> {
    let output = Command::new("niri")
        .args(["msg", "-j", "focused-output"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    serde_json::from_slice::<FocusedOutput>(&output.stdout)
        .ok()
        .map(|output| output.name)
}

pub fn get_niri_windows () -> Vec<NiriWindow> {
    let _time = TimeOfScope::new("get_niri_windows");
    // (Vec::new(), HashMap::new())
    let output = Command::new("niri").arg("msg").arg("-j").arg("windows").output();
    let stdout = String::from_utf8(output.unwrap().stdout).unwrap();
    // println!("\n{:?}", stdout);
    serde_json::from_str(&stdout).expect("Failed to parse niri windows JSON")
}

/* pub fn get_niri_workspaces () -> HashMap<u8, NiriWorkspace> {
    let workspaces: Vec<NiriWorkspace>;
    let workspaces_map: HashMap<u8, NiriWorkspace>;
    {
        let output = Command::new("niri").arg("msg").arg("-j").arg("workspaces").output();
        let stdout = String::from_utf8(output.unwrap().stdout).unwrap();
        // println!("\n{:?}", stdout);
        workspaces = serde_json::from_str(&stdout).unwrap();
        workspaces_map = workspaces.into_iter().map(|ws| (ws.id, ws)).collect();
    }
    workspaces_map
} */

pub fn focus_niri_window (target_id: u32) -> Result<(), String> {
    let status = Command::new("niri")
        .args(["msg", "action", "focus-window", "--id", &target_id.to_string()])
        .status()
        .map_err(|err| format!("Failed to execute niri focus command: {err}"))?;

    if !status.success() {
        return Err(format!("niri focus-window {} exited with status {:?}", target_id, status.code()));
    }

    Ok(())
}