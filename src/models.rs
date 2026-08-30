use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct YearlyArchive {
    pub version: u32,
    pub year: u32,
    pub entries: Vec<JournalEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JournalEntry {
    pub id: String,
    pub date: String,
    pub time: String,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub template: TemplateType,
    pub created_at: String,
    pub updated_at: String,
    pub nonce: String,
    pub ciphertext: String,
    pub tag: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChecklistItem {
    pub text: String,
    pub done: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DailyDraft {
    pub date: String,
    pub title: String,
    pub template: TemplateType,
    pub intentions: Vec<ChecklistItem>,
    pub carry_forward: Vec<ChecklistItem>,
    pub work_mode: WorkMode,
    pub content: String,
    pub summary_rating: String,
    pub key_achievements: String,
    pub learnings: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkMode {
    pub slot_minutes: u32,
    pub current_slot_start: Option<String>,
    pub slots_skipped: u32,
    pub slots: Vec<WorkSlot>,
    pub notification_sent_for_slot: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkSlot {
    pub slot_start: String,
    pub slot_end: String,
    pub note: String,
    pub worked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub enum TemplateType {
    #[default]
    Empty,
    Daily,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeyMaterial {
    pub salt: Vec<u8>,
    pub iterations: u32,
    pub derived_key: Vec<u8>,
    pub keystore_path: String,
}
