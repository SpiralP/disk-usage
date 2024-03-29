use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum Entry {
    File {
        path: Vec<String>,
        size: u64,
    },
    Directory {
        path: Vec<String>,
        size: u64,
        updating: UpdatingStatus,
    },
}

#[derive(Debug, Serialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum UpdatingStatus {
    Idle,
    Updating,
    Finished,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum EventMessage {
    #[serde(rename_all = "camelCase")]
    DirectoryChange {
        /// always Entry::Directory
        current_directory: Entry,
        entries: Vec<Entry>,
        breadcrumb_entries: Vec<Entry>,
        available_space: u64,
    },

    SizeUpdate {
        /// always Entry::Directory
        entry: Entry,
    },

    Deleting {
        path: Vec<String>,
        status: DeletingStatus,
    },
}

#[derive(Debug, Serialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum DeletingStatus {
    Deleting,
    Finished,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum ControlMessage {
    ChangeDirectory { path: Vec<String> },
    Delete { path: Vec<String> },
    Reveal { path: Vec<String> },
}
