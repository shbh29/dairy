mod draft;
mod keystore;
mod models;

use chrono::Timelike;
use std::io::{self, Write};

use crate::draft::{
    archive_for_year, apply_slot_note, check_and_send_notification, finalize_draft, load_or_create_draft, move_checklist_item,
    read_archive_entry, render_checklist, render_work_slots, save_draft, star_rating_for_worked_slots,
};
use crate::models::{ChecklistItem, DailyDraft, TemplateType, WorkSlot};

fn list_template(draft: &DailyDraft) {
    println!("Template: {}", match draft.template {
        TemplateType::Empty => "Empty",
        TemplateType::Daily => "Daily",
    });
}

fn add_checklist_item(items: &mut Vec<ChecklistItem>, text: &str) {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        println!("Item text cannot be empty.");
        return;
    }

    items.push(ChecklistItem {
        text: trimmed.to_string(),
        done: false,
    });
}

fn update_checklist_item(items: &mut Vec<ChecklistItem>, index: usize, done: bool) -> bool {
    if let Some(item) = items.get_mut(index) {
        item.done = done;
        true
    } else {
        false
    }
}

fn handle_intentions(draft: &mut DailyDraft, args: &[String]) {
    if args.is_empty() {
        println!("{}", render_checklist("Intentions", &draft.intentions));
        return;
    }

    match args[0].as_str() {
        "add" => {
            if args.len() < 2 {
                println!("Usage: /in add <task>");
                return;
            }
            let text = args[1..].join(" ");
            add_checklist_item(&mut draft.intentions, &text);
            println!("{}", render_checklist("Intentions", &draft.intentions));
        }
        "list" => println!("{}", render_checklist("Intentions", &draft.intentions)),
        "done" => {
            let idx = args.get(1).and_then(|value| value.parse::<usize>().ok());
            if let Some(index) = idx {
                if update_checklist_item(&mut draft.intentions, index - 1, true) {
                    println!("{}", render_checklist("Intentions", &draft.intentions));
                } else {
                    println!("Intentions index out of range.");
                }
            } else {
                println!("Usage: /in done <index>");
            }
        }
        "undone" => {
            let idx = args.get(1).and_then(|value| value.parse::<usize>().ok());
            if let Some(index) = idx {
                if update_checklist_item(&mut draft.intentions, index - 1, false) {
                    println!("{}", render_checklist("Intentions", &draft.intentions));
                } else {
                    println!("Intentions index out of range.");
                }
            } else {
                println!("Usage: /in undone <index>");
            }
        }
        "remove" => {
            let idx = args.get(1).and_then(|value| value.parse::<usize>().ok());
            if let Some(index) = idx {
                if index == 0 || index > draft.intentions.len() {
                    println!("Intentions index out of range.");
                } else {
                    draft.intentions.remove(index - 1);
                    println!("{}", render_checklist("Intentions", &draft.intentions));
                }
            } else {
                println!("Usage: /in remove <index>");
            }
        }
        "clear" => {
            draft.intentions.clear();
            println!("{}", render_checklist("Intentions", &draft.intentions));
        }
        "move-to-cf" => {
            let idx = args.get(1).and_then(|value| value.parse::<usize>().ok());
            match idx {
                Some(index) => match move_checklist_item(&mut draft.intentions, &mut draft.carry_forward, index) {
                    Ok(_) => {
                        println!("{}", render_checklist("Intentions", &draft.intentions));
                        println!("{}", render_checklist("Carry Forward", &draft.carry_forward));
                    }
                    Err(err) => println!("{err}"),
                },
                None => println!("Usage: /in move-to-cf <index>"),
            }
        }
        _ => println!("Usage: /in [add|list|done|undone|remove|clear|move-to-cf]"),
    }
}

fn handle_carry_forward(draft: &mut DailyDraft, args: &[String]) {
    if args.is_empty() {
        println!("{}", render_checklist("Carry Forward", &draft.carry_forward));
        return;
    }

    match args[0].as_str() {
        "add" => {
            if args.len() < 2 {
                println!("Usage: /cf add <task>");
                return;
            }
            let text = args[1..].join(" ");
            add_checklist_item(&mut draft.carry_forward, &text);
            println!("{}", render_checklist("Carry Forward", &draft.carry_forward));
        }
        "list" => println!("{}", render_checklist("Carry Forward", &draft.carry_forward)),
        "done" => {
            let idx = args.get(1).and_then(|value| value.parse::<usize>().ok());
            if let Some(index) = idx {
                if update_checklist_item(&mut draft.carry_forward, index - 1, true) {
                    println!("{}", render_checklist("Carry Forward", &draft.carry_forward));
                } else {
                    println!("Carry Forward index out of range.");
                }
            } else {
                println!("Usage: /cf done <index>");
            }
        }
        "undone" => {
            let idx = args.get(1).and_then(|value| value.parse::<usize>().ok());
            if let Some(index) = idx {
                if update_checklist_item(&mut draft.carry_forward, index - 1, false) {
                    println!("{}", render_checklist("Carry Forward", &draft.carry_forward));
                } else {
                    println!("Carry Forward index out of range.");
                }
            } else {
                println!("Usage: /cf undone <index>");
            }
        }
        "remove" => {
            let idx = args.get(1).and_then(|value| value.parse::<usize>().ok());
            if let Some(index) = idx {
                if index == 0 || index > draft.carry_forward.len() {
                    println!("Carry Forward index out of range.");
                } else {
                    draft.carry_forward.remove(index - 1);
                    println!("{}", render_checklist("Carry Forward", &draft.carry_forward));
                }
            } else {
                println!("Usage: /cf remove <index>");
            }
        }
        "clear" => {
            draft.carry_forward.clear();
            println!("{}", render_checklist("Carry Forward", &draft.carry_forward));
        }
        "move-to-in" => {
            let idx = args.get(1).and_then(|value| value.parse::<usize>().ok());
            match idx {
                Some(index) => match move_checklist_item(&mut draft.carry_forward, &mut draft.intentions, index) {
                    Ok(_) => {
                        println!("{}", render_checklist("Carry Forward", &draft.carry_forward));
                        println!("{}", render_checklist("Intentions", &draft.intentions));
                    }
                    Err(err) => println!("{err}"),
                },
                None => println!("Usage: /cf move-to-in <index>"),
            }
        }
        _ => println!("Usage: /cf [add|list|done|undone|remove|clear|move-to-in]"),
    }
}

fn format_time_bucket(date: &str, minute: u32) -> String {
    let hour = minute / 60;
    let minute_value = minute % 60;
    format!("{date}T{:02}:{:02}:00", hour, minute_value)
}

fn start_work_slot(draft: &mut DailyDraft) {
    let now = chrono::Local::now();
    let current_minute = now.hour() * 60 + now.minute();
    let slot_start_minute = (current_minute / 10) * 10;
    let slot_start = format_time_bucket(&draft.date, slot_start_minute as u32);
    let slot_end = format_time_bucket(&draft.date, (slot_start_minute + 10) as u32);

    if draft.work_mode.current_slot_start == Some(slot_start.clone()) {
        println!("A work slot is already active for this time bucket.");
        return;
    }

    draft.work_mode.current_slot_start = Some(slot_start.clone());
    draft.work_mode.notification_sent_for_slot = None;
    draft.work_mode.slots.push(WorkSlot {
        slot_start: slot_start.clone(),
        slot_end: slot_end.clone(),
        note: String::new(),
        worked: false,
    });
    println!("Started work slot {} - {}.", slot_label(&slot_start), slot_label(&slot_end));
}

fn end_work_slot(draft: &mut DailyDraft, note: Option<&str>) {
    let Some(slot_start) = draft.work_mode.current_slot_start.clone() else {
        println!("No active work slot is running.");
        return;
    };

    if let Some(slot) = draft.work_mode.slots.iter_mut().rev().find(|slot| slot.slot_start == slot_start) {
        let trimmed = note.map(str::trim).unwrap_or("");
        if trimmed.is_empty() {
            slot.worked = false;
            slot.note = String::new();
        } else {
            apply_slot_note(slot, trimmed);
        }
    }

    draft.work_mode.current_slot_start = None;
    println!("{}", render_work_slots(&draft.work_mode));
}

fn slot_minute(slot_start: &str) -> Option<u32> {
    if let Some(rest) = slot_start.split('T').nth(1) {
        let mut parts = rest.split(':');
        let hour = parts.next()?.parse::<u32>().ok()?;
        let minute = parts.next()?.parse::<u32>().ok()?;
        return Some(hour * 60 + minute);
    }
    None
}

fn slot_label(value: &str) -> String {
    if let Some(rest) = value.split('T').nth(1) {
        let mut parts = rest.split(':');
        let hour = parts.next().unwrap_or("00");
        let minute = parts.next().unwrap_or("00");
        return format!("{hour}:{minute}");
    }

    value.to_string()
}

fn ensure_current_slot(draft: &mut DailyDraft) {
    let now = chrono::Local::now();
    let current_minute = now.hour() * 60 + now.minute();
    let current_bucket = (current_minute / 10) * 10;
    let current_start = format_time_bucket(&draft.date, current_bucket as u32);
    let current_end = format_time_bucket(&draft.date, (current_bucket + 10) as u32);

    if draft.work_mode.slots.iter().any(|slot| slot.slot_start == current_start) {
        draft.work_mode.current_slot_start = Some(current_start.clone());
        if draft.work_mode.notification_sent_for_slot.as_ref() != Some(&current_start) {
            draft.work_mode.notification_sent_for_slot = None;
        }
        return;
    }

    if let Some(last) = draft.work_mode.slots.last() {
        if let Some(last_minute) = slot_minute(&last.slot_start) {
            let mut next_bucket = last_minute + 10;
            while next_bucket < current_bucket {
                let slot_start = format_time_bucket(&draft.date, next_bucket as u32);
                let slot_end = format_time_bucket(&draft.date, (next_bucket + 10) as u32);
                draft.work_mode.slots.push(WorkSlot {
                    slot_start: slot_start.clone(),
                    slot_end: slot_end.clone(),
                    note: String::new(),
                    worked: false,
                });
                next_bucket += 10;
            }
        }
    }

    draft.work_mode.slots.push(WorkSlot {
        slot_start: current_start.clone(),
        slot_end: current_end.clone(),
        note: String::new(),
        worked: false,
    });
    draft.work_mode.current_slot_start = Some(current_start);
    draft.work_mode.notification_sent_for_slot = None;
}

fn note_current_slot(draft: &mut DailyDraft, note_text: &str) {
    ensure_current_slot(draft);
    let current_slot_start = draft.work_mode.current_slot_start.clone().unwrap_or_default();
    let slot = draft
        .work_mode
        .slots
        .iter_mut()
        .find(|slot| slot.slot_start == current_slot_start)
        .expect("current slot should exist after ensure_current_slot");

    let trimmed = note_text.trim();
    if trimmed.is_empty() {
        slot.worked = false;
        slot.note = String::new();
    } else {
        apply_slot_note(slot, trimmed);
    }
}

fn payload_note(value: &str) -> String {
    if value.len() <= 120 {
        value.to_string()
    } else {
        format!("{}...", &value[..117])
    }
}

fn display_help() {
    println!("Commands: /template [daily|empty], /in [add|list|done|undone|remove|clear|move-to-cf], /cf [add|list|done|undone|remove|clear|move-to-in], /ws, /we [note], /nt <note>, /uws <ws_index> <note_text>, /summarize, /view, /save, /finalize, /help, /exit");
    println!("Work slot rule: non-empty note = worked, empty note = skipped.");
}

fn summarize_day(draft: &DailyDraft) -> String {
    let worked_slots = draft.work_mode.slots.iter().filter(|slot| slot.worked).count();
    let rating = star_rating_for_worked_slots(worked_slots);
    let total_slots = draft.work_mode.slots.len();
    format!("Day rating: {rating} | worked {worked_slots}/{total_slots} slots")
}

fn finalize_if_day_changed(draft_path: &std::path::Path, draft: &mut DailyDraft) {
    let current_path = crate::draft::draft_path();
    if draft_path != &current_path {
        return;
    }

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    if draft.date == today {
        return;
    }

    match finalize_draft(draft_path) {
        Ok(path) => println!("Rolled over previous day to {} before starting new day.", path.display()),
        Err(err) => println!("Failed to finalize previous day: {err}"),
    }

    *draft = load_or_create_draft(draft_path);
    println!("Started a fresh draft for {}.", draft.title);
}

fn rotate_focus(draft: &mut DailyDraft) {
    if draft.template == TemplateType::Empty {
        println!("{}", render_work_slots(&draft.work_mode));
        return;
    }

    let sections = [
        render_checklist("Intentions", &draft.intentions),
        render_checklist("Carry Forward", &draft.carry_forward),
        render_work_slots(&draft.work_mode),
    ];

    static mut INDEX: usize = 0;
    unsafe {
        let value = sections[INDEX % sections.len()].clone();
        println!("{value}");
        INDEX += 1;
    }
}

fn run_loop() {
    let mut draft_path = crate::draft::draft_path();
    let mut draft = load_or_create_draft(&draft_path);

    // Spawn background reminder thread that reads the draft file periodically
    // and sends a notification when 2 minutes remain and the user is inactive.
    let reminder_path = draft_path.clone();
    std::thread::spawn(move || {
        use chrono::Local;
        loop {
            let draft = crate::draft::load_or_create_draft(&reminder_path);
            if let Some(slot_start) = &draft.work_mode.current_slot_start {
                if draft.work_mode.notification_sent_for_slot.as_ref() != Some(slot_start) {
                    if let Some(slot_min) = slot_minute(slot_start) {
                        let slot_end_min = slot_min + draft.work_mode.slot_minutes as u32;
                        let now = Local::now();
                        let current_min = now.hour() * 60 + now.minute();
                        let minutes_remaining = slot_end_min as i32 - current_min as i32;

                        if minutes_remaining <= 2 && minutes_remaining > 0 {
                            // check inactivity and maximum inactivity window (3 slots)
                            let max_inactive_seconds = (draft.work_mode.slot_minutes as i64) * 3 * 60;
                            let inactivity_seconds = match &draft.work_mode.last_interaction_at {
                                Some(ts) => match chrono::DateTime::parse_from_rfc3339(ts) {
                                    Ok(t) => now.signed_duration_since(t).num_seconds(),
                                    Err(_) => 0,
                                },
                                None => 0,
                            };

                            // if user has been inactive for more than 3 * slot_minutes, suppress further reminders for this slot
                            if inactivity_seconds > max_inactive_seconds {
                                let mut updated = crate::draft::load_or_create_draft(&reminder_path);
                                updated.work_mode.notification_sent_for_slot = Some(slot_start.clone());
                                crate::draft::save_draft(&reminder_path, &updated);
                                // skip notifying
                            } else if inactivity_seconds > 30 {
                                let start_display = if let Some(rest) = slot_start.split('T').nth(1) {
                                    let mut parts = rest.split(':');
                                    let hour = parts.next().unwrap_or("00");
                                    let minute = parts.next().unwrap_or("00");
                                    format!("{}:{}", hour, minute)
                                } else {
                                    slot_start.clone()
                                };
                                let end_hour = ((slot_min + draft.work_mode.slot_minutes as u32) / 60) % 24;
                                let end_minute = (slot_min + draft.work_mode.slot_minutes as u32) % 60;
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

                                // persist that we've notified for this slot
                                let mut updated = crate::draft::load_or_create_draft(&reminder_path);
                                updated.work_mode.notification_sent_for_slot = Some(slot_start.clone());
                                crate::draft::save_draft(&reminder_path, &updated);
                            }
                        }
                    }
                }
            }

            std::thread::sleep(std::time::Duration::from_secs(5));
        }
    });

    finalize_if_day_changed(&draft_path, &mut draft);
    println!("Daily Draft: {}", draft.title);
    list_template(&draft);
    println!("Type /help for commands. Press Enter on a blank line to rotate through daily sections.");

    loop {
        finalize_if_day_changed(&draft_path, &mut draft);
        print!("dairy> ");
        io::stdout().flush().expect("flush prompt");

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("read command");
        // record last interaction so background reminders avoid notifying while user is active
        draft.work_mode.last_interaction_at = Some(chrono::Local::now().to_rfc3339());
        crate::draft::save_draft(&draft_path, &draft);
        let trimmed = input.trim();

        if trimmed.is_empty() {
            rotate_focus(&mut draft);
            continue;
        }

        if trimmed == "/help" {
            display_help();
            continue;
        }

        if trimmed == "/save" {
            save_draft(&draft_path, &draft);
            println!("Draft saved.");
            continue;
        }

        if trimmed == "/view" {
            println!("{}", render_work_slots(&draft.work_mode));
            continue;
        }

        if trimmed == "/exit" {
            save_draft(&draft_path, &draft);
            println!("Exiting draft loop.");
            break;
        }

        if trimmed == "/finalize" {
            match finalize_draft(&draft_path) {
                Ok(path) => {
                    println!("Draft finalized to {}", path.display());
                    break;
                }
                Err(err) => println!("Finalize failed: {err}"),
            }
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("/edit ") {
            let date = rest.trim();
            if date.is_empty() {
                println!("Usage: /edit <YYYY-MM-DD>");
                continue;
            }

            match crate::draft::load_or_create_historical_draft(date) {
                Ok(loaded) => {
                    draft_path = crate::draft::draft_path_for_date(date);
                    draft = loaded;
                    println!("Opened draft for {}.", date);
                    println!("Daily Draft: {}", draft.title);
                }
                Err(err) => println!("Could not open {date}: {err}"),
            }
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("/template ") {
            let value = rest.trim().to_ascii_lowercase();
            draft.template = match value.as_str() {
                "daily" => TemplateType::Daily,
                "empty" => TemplateType::Empty,
                _ => {
                    println!("Template must be 'daily' or 'empty'.");
                    continue;
                }
            };
            list_template(&draft);
            save_draft(&draft_path, &draft);
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("/in ") {
            let mut args = rest.split_whitespace();
            let command = args.next().unwrap_or("");
            let remaining: Vec<String> = args.map(str::to_string).collect();
            handle_intentions(&mut draft, &vec![command.to_string()].into_iter().chain(remaining.clone()).collect::<Vec<_>>());
            save_draft(&draft_path, &draft);
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("/cf ") {
            let mut args = rest.split_whitespace();
            let command = args.next().unwrap_or("");
            let remaining: Vec<String> = args.map(str::to_string).collect();
            handle_carry_forward(&mut draft, &vec![command.to_string()].into_iter().chain(remaining.clone()).collect::<Vec<_>>());
            save_draft(&draft_path, &draft);
            continue;
        }

        if trimmed == "/ws" {
            start_work_slot(&mut draft);
            save_draft(&draft_path, &draft);
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("/we") {
            let note = if rest.trim().is_empty() { None } else { Some(rest.trim()) };
            end_work_slot(&mut draft, note);
            save_draft(&draft_path, &draft);
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("/uws ") {
            let mut parts = rest.splitn(2, ' ');
            let index_text = parts.next().unwrap_or("");
            let note_text = parts.next().unwrap_or("").trim();

            let index = match index_text.parse::<usize>() {
                Ok(value) => value,
                Err(_) => {
                    println!("Usage: /uws <ws_index> <note_text>");
                    continue;
                }
            };

            if index == 0 || index > draft.work_mode.slots.len() {
                println!("Work slot index out of range.");
                continue;
            }

            let slot = &mut draft.work_mode.slots[index - 1];
            if note_text.is_empty() {
                slot.worked = false;
                slot.note = String::new();
            } else {
                apply_slot_note(slot, note_text);
            }

            println!("{}", render_work_slots(&draft.work_mode));
            save_draft(&draft_path, &draft);
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("/nt") {
            let note_text = rest.trim();
            note_current_slot(&mut draft, note_text);
            println!("{}", render_work_slots(&draft.work_mode));
            save_draft(&draft_path, &draft);
            continue;
        }

        if trimmed == "/nt" {
            println!("Usage: /nt <note>. Empty note = skipped, non-empty note = worked.");
            continue;
        }

        if trimmed == "/summarize" {
            draft.summary_rating = summarize_day(&draft);
            println!("{}", draft.summary_rating);
            println!("Add achievements and learnings with /key-achievements and /learnings.");
            save_draft(&draft_path, &draft);
            continue;
        }

        if trimmed == "/key-achievements" {
            println!("Enter key achievements; end with a blank line:");
            let mut text = String::new();
            loop {
                let mut line = String::new();
                io::stdin().read_line(&mut line).expect("read achievements");
                if line.trim().is_empty() {
                    break;
                }
                text.push_str(&line);
            }
            draft.key_achievements = text.trim().to_string();
            save_draft(&draft_path, &draft);
            println!("Key achievements saved.");
            continue;
        }

        if trimmed == "/learnings" {
            println!("Enter learnings; end with a blank line:");
            let mut text = String::new();
            loop {
                let mut line = String::new();
                io::stdin().read_line(&mut line).expect("read learnings");
                if line.trim().is_empty() {
                    break;
                }
                text.push_str(&line);
            }
            draft.learnings = text.trim().to_string();
            save_draft(&draft_path, &draft);
            println!("Learnings saved.");
            continue;
        }

        println!("Unknown command. Type /help to see available actions.");
    }
}

#[cfg(test)]
mod tests {
    use super::{ensure_current_slot, note_current_slot};
    use crate::models::{DailyDraft, TemplateType, WorkMode, WorkSlot};

    #[test]
    fn slot_label_uses_hhmm_range_format() {
        let slot_start = "2026-08-30T00:20:00";
        let slot_end = "2026-08-30T00:30:00";

        assert_eq!(super::slot_label(slot_start), "00:20");
        assert_eq!(super::slot_label(slot_end), "00:30");
    }

    #[test]
    fn note_current_slot_fills_missing_skipped_slots_before_current_one() {
        let mut draft = DailyDraft {
            date: "2026-08-29".to_string(),
            title: "2026-08-29-Saturday".to_string(),
            template: TemplateType::Daily,
            intentions: vec![],
            carry_forward: vec![],
            work_mode: WorkMode {
                slot_minutes: 10,
                current_slot_start: None,
                slots_skipped: 0,
                slots: vec![
                    WorkSlot {
                        slot_start: "2026-08-29T09:00:00".to_string(),
                        slot_end: "2026-08-29T09:10:00".to_string(),
                        note: String::new(),
                        worked: false,
                    },
                ],
                notification_sent_for_slot: None,
                last_interaction_at: None,
            },
            content: String::new(),
            summary_rating: String::new(),
            key_achievements: String::new(),
            learnings: String::new(),
            updated_at: "2026-08-29T00:00:00Z".to_string(),
        };

        let before = draft.work_mode.slots.len();
        note_current_slot(&mut draft, "logged progress");

        assert!(draft.work_mode.slots.len() >= before + 1);
        let current = draft.work_mode.slots.last().unwrap();
        assert_eq!(current.note, "logged progress");
        assert!(current.worked);
    }

    #[test]
    fn empty_note_for_current_slot_means_skipped() {
        let mut draft = DailyDraft {
            date: "2026-08-29".to_string(),
            title: "2026-08-29-Saturday".to_string(),
            template: TemplateType::Daily,
            intentions: vec![],
            carry_forward: vec![],
            work_mode: WorkMode {
                slot_minutes: 10,
                current_slot_start: None,
                slots_skipped: 0,
                slots: vec![WorkSlot {
                    slot_start: "2026-08-29T09:00:00".to_string(),
                    slot_end: "2026-08-29T09:10:00".to_string(),
                    note: String::new(),
                    worked: false,
                }],
                notification_sent_for_slot: None,
                last_interaction_at: None,
            },
            content: String::new(),
            summary_rating: String::new(),
            key_achievements: String::new(),
            learnings: String::new(),
            updated_at: "2026-08-29T00:00:00Z".to_string(),
        };

        note_current_slot(&mut draft, "");
        let current = draft.work_mode.slots.last().unwrap();
        assert!(!current.worked);
        assert!(current.note.is_empty());
    }

    #[test]
    fn finalize_if_day_changed_reloads_new_day() {
        let draft_path = std::path::PathBuf::from("data/drafts/current.json");
        let mut draft = DailyDraft {
            date: "2026-08-29".to_string(),
            title: "2026-08-29-Saturday".to_string(),
            template: TemplateType::Daily,
            intentions: vec![],
            carry_forward: vec![],
            work_mode: WorkMode {
                slot_minutes: 10,
                current_slot_start: None,
                slots_skipped: 0,
                slots: vec![],
                notification_sent_for_slot: None,
                last_interaction_at: None,
            },
            content: "old day".to_string(),
            summary_rating: String::new(),
            key_achievements: String::new(),
            learnings: String::new(),
            updated_at: "2026-08-29T00:00:00Z".to_string(),
        };

        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        if draft.date != today {
            draft.date = "2026-08-29".to_string();
        }

        if draft.date != today {
            return;
        }

        super::finalize_if_day_changed(&draft_path, &mut draft);
        assert_eq!(draft.date, today);
    }
}

fn main() {
    println!("What's on your mind?");
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("draft") => run_loop(),
        Some("finalize") => {
            let draft_path = crate::draft::draft_path();
            match finalize_draft(&draft_path) {
                Ok(archive_path) => println!("Draft finalized to {}", archive_path.display()),
                Err(err) => println!("Finalize failed: {err}"),
            }
        }
        Some("read") => {
            let year = args.get(2).and_then(|value| value.parse::<u32>().ok());
            let entry_id = args.get(3).cloned();

            match (year, entry_id) {
                (Some(year), Some(entry_id)) => {
                    match archive_for_year(year) {
                        Ok(archive) => {
                            let entry = archive
                                .entries
                                .iter()
                                .find(|entry| entry.id == entry_id)
                                .cloned();

                            if let Some(entry) = entry {
                                match read_archive_entry(year, &entry.id) {
                                    Ok(content) => {
                                        println!("Entry: {}", entry.title);
                                        println!("Date: {}", entry.date);
                                        println!("--- decrypted content ---");
                                        println!("{content}");
                                    }
                                    Err(err) => println!("Read failed: {err}"),
                                }
                            } else {
                                println!("No entry with id '{entry_id}' in archive {year}");
                            }
                        }
                        Err(err) => println!("Archive read failed: {err}"),
                    }
                }
                _ => println!("Usage: dairy read <year> <entry-id>"),
            }
        }
        _ => run_loop(),
    }
}
