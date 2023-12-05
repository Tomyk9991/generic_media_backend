use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct List {
    entries: Vec<ListEntry>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListEntry {
    title: String,
    checked: bool,
    sub_entries: Vec<SubEntry>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubEntry {
    title: String,
    checked: bool
}