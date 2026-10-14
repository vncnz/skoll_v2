use serde_derive::Deserialize;
use std::process::Command;
use std::collections::HashMap;
use std::error::Error;

#[derive(Deserialize,Debug,Clone)]
pub struct NiriWindow {
    pub id: u32,
    pub title: Option<String>,
    pub app_id: String,
    pub workspace_id: u8,
    pub is_focused: bool,
    pub is_urgent: bool
}

#[derive(Deserialize,Debug)]
pub struct NiriWorkspace {
    pub id: u8,
    pub idx: u32,
    pub name: Option<String>,
    pub output: String,
    pub is_active: bool,
    pub is_focused: bool,
    pub active_window_id: Option<u32>
}

pub fn get_niri_windows () -> (Vec<NiriWindow>, HashMap<u8, NiriWorkspace>) {
    // (Vec::new(), HashMap::new())
    let windows: Vec<NiriWindow>;
    {
        let output = Command::new("niri").arg("msg").arg("-j").arg("windows").output();
        let stdout = String::from_utf8(output.unwrap().stdout).unwrap();
        // println!("\n{:?}", stdout);
        windows = serde_json::from_str(&stdout).unwrap();
    }
    let workspaces: Vec<NiriWorkspace>;
    let workspaces_map: HashMap<u8, NiriWorkspace>;
    {
        let output = Command::new("niri").arg("msg").arg("-j").arg("workspaces").output();
        let stdout = String::from_utf8(output.unwrap().stdout).unwrap();
        // println!("\n{:?}", stdout);
        workspaces = serde_json::from_str(&stdout).unwrap();
        workspaces_map = workspaces.into_iter().map(|ws| (ws.id, ws)).collect();
    }
    (windows, workspaces_map)
}

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