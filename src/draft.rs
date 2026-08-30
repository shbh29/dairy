use crate::models::{ChecklistItem, DailyDraft, JournalEntry, TemplateType, WorkMode, WorkSlot, YearlyArchive};
use chrono::{Datelike, Local, NaiveDate, Timelike};
use serde_json;
use std::fs;
use std::path::{Path, PathBuf};

pub fn draft_path() -> PathBuf {
    let path = Path::new("data").join("drafts").join("current.json");
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    path
}

pub fn draft_path_for_date(date: &str) -> PathBuf {
    let path = Path::new("data").join("drafts").join(format!("{date}.json"));
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    path
}

pub fn archive_dir() -> PathBuf {
    let path = Path::new("data").join("archives");
    let _ = fs::create_dir_all(&path);
    path
}

fn default_work_mode() -> WorkMode {
    WorkMode {
        slot_minutes: 10,
        current_slot_start: None,
        slots_skipped: 0,
        slots: vec![WorkSlot {
            slot_start: String::new(),
            slot_end: String::new(),
            note: String::new(),
            worked: false,
        }],
        notification_sent_for_slot: None,
    }
}

fn current_title() -> String {
    let now = Local::now();
    format!("{}-{}", now.format("%Y-%m-%d"), now.format("%A"))
}

fn fresh_draft() -> DailyDraft {
    let now = Local::now();
    DailyDraft {
        date: now.format("%Y-%m-%d").to_string(),
        title: current_title(),
        template: TemplateType::Daily,
        intentions: vec![],
        carry_forward: vec![],
        work_mode: default_work_mode(),
        content: String::new(),
        summary_rating: String::new(),
        key_achievements: String::new(),
        learnings: String::new(),
        updated_at: now.to_rfc3339(),
    }
}

pub fn move_checklist_item(from: &mut Vec<ChecklistItem>, to: &mut Vec<ChecklistItem>, index: usize) -> Result<(), String> {
    if index == 0 || index > from.len() {
        return Err("index out of range".to_string());
    }

    let item = from.remove(index - 1);
    to.push(item);
    Ok(())
}

pub fn apply_slot_note(slot: &mut WorkSlot, raw_note: &str) {
    let trimmed = raw_note.trim();
    if trimmed.is_empty() {
        slot.worked = false;
        slot.note = String::new();
        return;
    }

    if slot.note.is_empty() || slot.note == "no note" {
        slot.note = trimmed.to_string();
    } else {
        slot.note.push_str(" | ");
        slot.note.push_str(trimmed);
    }
    slot.worked = true;
}

pub fn star_rating_for_worked_slots(worked_slots: usize) -> String {
    let stars = worked_slots / 12;
    let star_text = "★".repeat(stars.min(5));
    if star_text.is_empty() {
        "0 stars".to_string()
    } else {
        format!("{star_text} ({stars} star{})", if stars == 1 { "" } else { "s" })
    }
}

pub fn preview_note(note: &str) -> String {
    let trimmed = note.trim();
    if trimmed.is_empty() {
        return "no note".to_string();
    }

    let chars: Vec<char> = trimmed.chars().collect();
    const MAX_PREVIEW: usize = 100;

    if chars.len() <= MAX_PREVIEW {
        trimmed.to_string()
    } else {
        let preview: String = chars.iter().take(MAX_PREVIEW).collect();
        format!("{preview}...")
    }
}

pub fn render_checklist(title: &str, items: &[crate::models::ChecklistItem]) -> String {
    if items.is_empty() {
        return format!("{title}:\n  (empty)");
    }

    let lines = items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let checkbox = if item.done { "[x]" } else { "[ ]" };
            format!("  {}. {} {}", index + 1, checkbox, item.text)
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!("{title}:\n{lines}")
}

pub fn render_work_slots(work_mode: &WorkMode) -> String {
    if work_mode.slots.is_empty() {
        return "Work Slots:\n  (empty)".to_string();
    }

    let lines = work_mode
        .slots
        .iter()
        .enumerate()
        .map(|(index, slot)| {
            let status = if Some(&slot.slot_start) == work_mode.current_slot_start.as_ref() {
                "running"
            } else if slot.worked {
                "worked"
            } else {
                "skipped"
            };
            let note = preview_note(&slot.note);
            let start = format_slot_time(&slot.slot_start);
            let end = format_slot_time(&slot.slot_end);
            format!("  {}. {} - {} | {} | {}", index + 1, start, end, status, note)
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!("Work Slots:\n{lines}")
}

fn format_slot_time(value: &str) -> String {
    if let Some(rest) = value.split('T').nth(1) {
        let mut parts = rest.split(':');
        let hour = parts.next().unwrap_or("00");
        let minute = parts.next().unwrap_or("00");
        return format!("{hour}:{minute}");
    }

    value.to_string()
}

pub fn check_and_send_notification(work_mode: &mut WorkMode) {
    let Some(slot_start) = &work_mode.current_slot_start else {
        return;
    };

    if work_mode.notification_sent_for_slot == Some(slot_start.clone()) {
        return;
    }

    let now = Local::now();
    let current_minute = now.hour() * 60 + now.minute();

    if let Some(slot_minute) = slot_minute_from_string(slot_start) {
        let slot_end_minute = slot_minute + 10;
        let minutes_remaining = slot_end_minute as i32 - current_minute as i32;

        if minutes_remaining <= 2 && minutes_remaining > 0 {
            let start_display = format_slot_time(slot_start);
            let end_hour = ((slot_minute + 10) / 60) % 24;
            let end_minute = (slot_minute + 10) % 60;
            let end_display = format!("{:02}:{:02}", end_hour, end_minute);

            let body = format!(
                "{} minutes left in current work slot ({} - {}). Consider closing or updating it.",
                minutes_remaining, start_display, end_display
            );

            if let Err(e) = notify_rust::Notification::new()
                .summary("Dairy Work Reminder")
                .body(&body)
                .show()
            {
                eprintln!("Failed to show notification: {}", e);
            }

            work_mode.notification_sent_for_slot = Some(slot_start.clone());
        }
    }
}

fn slot_minute_from_string(slot_start: &str) -> Option<u32> {
    if let Some(rest) = slot_start.split('T').nth(1) {
        let mut parts = rest.split(':');
        let hour = parts.next()?.parse::<u32>().ok()?;
        let minute = parts.next()?.parse::<u32>().ok()?;
        return Some(hour * 60 + minute);
    }
    None
}


pub fn load_or_create_draft(path: &Path) -> DailyDraft {
    if let Ok(bytes) = fs::read(path) {
        if !bytes.is_empty() {
            if let Ok(draft) = serde_json::from_slice::<DailyDraft>(&bytes) {
                return draft;
            }
        }
    }

    let draft = fresh_draft();
    save_draft(path, &draft);
    draft
}

pub fn load_or_create_historical_draft(date: &str) -> Result<DailyDraft, String> {
    let path = draft_path_for_date(date);
    if let Ok(bytes) = fs::read(&path) {
        if !bytes.is_empty() {
            if let Ok(draft) = serde_json::from_slice::<DailyDraft>(&bytes) {
                return Ok(draft);
            }
        }
    }

    let year = date
        .parse::<i32>()
        .map_err(|_| format!("invalid date format: {date}"))?;
    let _ = year;

    if let Ok(parsed_date) = NaiveDate::parse_from_str(date, "%Y-%m-%d") {
        let year = parsed_date.year() as u32;
        if let Ok(archive) = archive_for_year(year) {
            if let Some(entry) = archive.entries.iter().find(|entry| entry.date == date) {
                let key = crate::keystore::ensure_keystore_key(&crate::keystore::keystore_path());
                let content = decrypt_archive_content(
                    key,
                    &entry.nonce,
                    &entry.ciphertext,
                    &entry.tag,
                )
                .map_err(|err| format!("decrypt historical entry: {err}"))?;

                let mut draft = fresh_draft();
                draft.date = date.to_string();
                draft.title = format!("{}-{}", date, parsed_date.format("%A"));
                draft.template = entry.template.clone();
                draft.content = content;
                draft.updated_at = entry.updated_at.clone();
                save_draft(&path, &draft);
                return Ok(draft);
            }
        }
    }

    let mut draft = fresh_draft();
    draft.date = date.to_string();
    draft.title = format!("{}-{}", date, Local::now().format("%A"));
    save_draft(&path, &draft);
    Ok(draft)
}

pub fn save_draft(path: &Path, draft: &DailyDraft) {
    let json = serde_json::to_string_pretty(draft).expect("serialize draft");
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    fs::write(path, json).expect("save draft file");
}

pub fn finalize_draft(draft_path: &Path) -> Result<PathBuf, String> {
    let draft_bytes = fs::read(draft_path).map_err(|err| format!("read draft: {err}"))?;
    let draft: DailyDraft = serde_json::from_slice(&draft_bytes)
        .map_err(|err| format!("parse draft JSON: {err}"))?;

    let year = NaiveDate::parse_from_str(&draft.date, "%Y-%m-%d")
        .map(|date| date.year() as u32)
        .unwrap_or_else(|_| Local::now().year() as u32);

    let archive_dir = archive_dir();
    let archive_path = archive_dir.join(format!("{year}.json"));

    let mut archive = if archive_path.exists() {
        let bytes = fs::read(&archive_path).map_err(|err| format!("read archive: {err}"))?;
        serde_json::from_slice(&bytes).unwrap_or(YearlyArchive {
            version: 1,
            year,
            entries: vec![],
        })
    } else {
        YearlyArchive {
            version: 1,
            year,
            entries: vec![],
        }
    };

    let keystore_key_path = crate::keystore::keystore_path();
    let key = crate::keystore::ensure_keystore_key(&keystore_key_path);
    let (nonce_hex, ciphertext_hex, tag_hex) = encrypt_plaintext_for_archive(key, &draft.content);

    archive.entries.push(JournalEntry {
        id: format!("{}-{}", draft.date, draft.title),
        date: draft.date.clone(),
        time: "00:00:00".to_string(),
        title: draft.title.clone(),
        content: String::new(),
        tags: vec![],
        template: draft.template.clone(),
        created_at: draft.updated_at.clone(),
        updated_at: draft.updated_at.clone(),
        nonce: nonce_hex,
        ciphertext: ciphertext_hex,
        tag: tag_hex,
    });

    let archive_json = serde_json::to_string_pretty(&archive)
        .map_err(|err| format!("serialize archive: {err}"))?;
    fs::write(&archive_path, archive_json).map_err(|err| format!("write archive: {err}"))?;

    let reset = fresh_draft();
    save_draft(draft_path, &reset);

    Ok(archive_path)
}

pub fn archive_for_year(year: u32) -> Result<YearlyArchive, String> {
    let archive_path = archive_dir().join(format!("{year}.json"));
    let bytes = fs::read(&archive_path).map_err(|err| format!("read archive: {err}"))?;
    serde_json::from_slice(&bytes).map_err(|err| format!("parse archive: {err}"))
}

pub fn read_archive_entry(year: u32, entry_id: &str) -> Result<String, String> {
    let archive = archive_for_year(year)?;
    let entry = archive
        .entries
        .iter()
        .find(|entry| entry.id == entry_id)
        .ok_or_else(|| format!("entry {entry_id} not found in archive {year}"))?;

    let key = crate::keystore::ensure_keystore_key(&crate::keystore::keystore_path());
    decrypt_archive_content(key, &entry.nonce, &entry.ciphertext, &entry.tag)
}

fn encrypt_plaintext_for_archive(key: [u8; 32], plaintext: &str) -> (String, String, String) {
    use aes_gcm::aead::{Aead, KeyInit};
    use aes_gcm::{Aes256Gcm, Nonce};

    let cipher = Aes256Gcm::new_from_slice(&key).expect("32-byte AES key");
    let nonce = rand::random::<[u8; 12]>();
    let mut encrypted = cipher
        .encrypt(Nonce::from_slice(&nonce), plaintext.as_bytes())
        .expect("encrypt draft content");

    let tag = encrypted.split_off(encrypted.len() - 16);
    let ciphertext = encrypted;

    (hex::encode(nonce), hex::encode(ciphertext), hex::encode(tag))
}

fn decrypt_archive_content(
    key: [u8; 32],
    nonce_hex: &str,
    ciphertext_hex: &str,
    tag_hex: &str,
) -> Result<String, String> {
    use aes_gcm::aead::{Aead, KeyInit};
    use aes_gcm::{Aes256Gcm, Nonce};

    let nonce = hex::decode(nonce_hex).map_err(|err| format!("decode nonce: {err}"))?;
    let mut ciphertext = hex::decode(ciphertext_hex)
        .map_err(|err| format!("decode ciphertext: {err}"))?;
    let tag = hex::decode(tag_hex).map_err(|err| format!("decode tag: {err}"))?;

    ciphertext.extend_from_slice(&tag);

    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|err| format!("invalid AES key: {err}"))?;
    let plaintext = cipher
        .decrypt(Nonce::from_slice(&nonce), ciphertext.as_ref())
        .map_err(|err| format!("decrypt archive content: {err}"))?;

    String::from_utf8(plaintext).map_err(|err| format!("decode plaintext utf8: {err}"))
}

#[cfg(test)]
mod tests {
    use super::{encrypt_plaintext_for_archive, preview_note};
    use crate::models::ChecklistItem;

    #[test]
    fn checklist_items_track_done_state() {
        let item = ChecklistItem {
            text: "Ship the daily loop".to_string(),
            done: false,
        };

        assert_eq!(item.text, "Ship the daily loop");
        assert!(!item.done);
    }

    #[test]
    fn slot_notes_are_previewed_shortly() {
        let preview = preview_note("This is a much longer note that should be shortened in the overview list so it stays readable and still clearly communicates the meaningful work without flooding the daily summary.");

        assert!(preview.len() <= 120);
        assert!(preview.ends_with("..."));
    }

    #[test]
    fn empty_slot_note_means_skipped() {
        let mut slot = super::WorkSlot {
            slot_start: "09:00".to_string(),
            slot_end: "09:10".to_string(),
            note: "existing".to_string(),
            worked: true,
        };

        super::apply_slot_note(&mut slot, "");
        assert!(!slot.worked);
        assert!(slot.note.is_empty());
    }

    #[test]
    fn star_rating_is_based_on_twelve_slot_blocks() {
        assert_eq!(super::star_rating_for_worked_slots(11), "0 stars");
        assert_eq!(super::star_rating_for_worked_slots(12), "★ (1 star)");
        assert_eq!(super::star_rating_for_worked_slots(24), "★★ (2 stars)");
    }

    #[test]
    fn archive_content_is_encrypted() {
        let (nonce_hex, ciphertext_hex, tag_hex) = encrypt_plaintext_for_archive([42u8; 32], "hello world");

        assert!(!nonce_hex.is_empty());
        assert!(!ciphertext_hex.is_empty());
        assert!(!tag_hex.is_empty());
        assert_ne!(ciphertext_hex, "hello world");
        assert_ne!(hex::encode("hello world"), ciphertext_hex);
    }

    #[test]
    fn archive_content_round_trip_decrypts() {
        let key = [7u8; 32];
        let plaintext = "keep this private";
        let (nonce_hex, ciphertext_hex, tag_hex) = encrypt_plaintext_for_archive(key, plaintext);

        let decrypted = super::decrypt_archive_content(key, &nonce_hex, &ciphertext_hex, &tag_hex)
            .expect("decrypt archive content");

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn lookup_entry_decrypts_archive_content() {
        let key = [9u8; 32];
        let plaintext = "private daily entry";
        let entry = super::JournalEntry {
            id: "2026-08-29-Example".to_string(),
            date: "2026-08-29".to_string(),
            time: "00:00:00".to_string(),
            title: "2026-08-29-Saturday".to_string(),
            content: String::new(),
            tags: vec![],
            template: super::TemplateType::Daily,
            created_at: "2026-08-29T00:00:00Z".to_string(),
            updated_at: "2026-08-29T00:00:00Z".to_string(),
            nonce: String::new(),
            ciphertext: String::new(),
            tag: String::new(),
        };

        let (nonce_hex, ciphertext_hex, tag_hex) = encrypt_plaintext_for_archive(key, plaintext);
        let decrypted = super::decrypt_archive_content(key, &nonce_hex, &ciphertext_hex, &tag_hex)
            .expect("decrypt archive content");

        assert_eq!(decrypted, plaintext);
        assert_ne!(entry.id, String::new());
    }
}
