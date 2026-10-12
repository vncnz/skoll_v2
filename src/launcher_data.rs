use std::path::PathBuf;

pub struct AppEntry {
    pub name: String,
    pub exec: String,
    pub icon_path: Option<PathBuf>,
    pub comment: String,
    pub terminal: bool
}

pub fn get_sample () -> Vec<AppEntry> {
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
}