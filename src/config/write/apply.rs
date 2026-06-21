use toml_edit::{value, DocumentMut, Item, Table};

use crate::app::{
    EditableConfigUpdate, JournalConfigEdit, LogLevelConfigEdit, MatrixConfigEdit,
    MonitorConfigEdit, WebConfigEdit,
};

pub(super) fn validate_update(
    update: &EditableConfigUpdate,
) -> Result<(), super::ConfigWriteError> {
    if let Some(web) = &update.web {
        if let Some(host) = &web.host {
            if !is_loopback_host(host) {
                return Err(super::ConfigWriteError::UnsafeRemoteBind { host: host.clone() });
            }
        }
    }
    Ok(())
}

pub(super) fn apply_update(document: &mut DocumentMut, update: &EditableConfigUpdate) {
    if let Some(journal) = &update.journal {
        apply_journal(document, journal);
    }
    if let Some(monitor) = &update.monitor {
        apply_monitor(document, monitor);
    }
    if let Some(log_levels) = &update.log_levels {
        apply_log_levels(document, log_levels);
    }
    if let Some(matrix) = &update.matrix {
        apply_matrix(document, matrix);
    }
    if let Some(web) = &update.web {
        apply_web(document, web);
    }
}

fn apply_journal(document: &mut DocumentMut, edit: &JournalConfigEdit) {
    let table = section(document, "journal");
    set_string(table, "folder", edit.folder.as_deref());
    set_u16(table, "recent_files", edit.recent_files);
}

fn apply_monitor(document: &mut DocumentMut, edit: &MonitorConfigEdit) {
    let table = section(document, "monitor");
    set_bool(table, "use_utc", edit.use_utc);
    set_bool(table, "live_status", edit.live_status);
    set_bool(table, "dynamic_title", edit.dynamic_title);
    set_u16(table, "warn_kill_rate", edit.warn_kill_rate);
    set_u16(
        table,
        "warn_kill_rate_delay_minutes",
        edit.warn_kill_rate_delay_minutes,
    );
    set_u16(table, "warn_no_kills_minutes", edit.warn_no_kills_minutes);
    set_u16(
        table,
        "warn_no_kills_initial_minutes",
        edit.warn_no_kills_initial_minutes,
    );
    set_u16(table, "warn_cooldown_minutes", edit.warn_cooldown_minutes);
    set_u16(table, "duplicate_max", edit.duplicate_max);
    set_bool(table, "pirate_names", edit.pirate_names);
    set_bool(table, "bounty_faction", edit.bounty_faction);
    set_bool(table, "bounty_value", edit.bounty_value);
    set_bool(table, "extended_stats", edit.extended_stats);
    set_u8(table, "min_scan_level", edit.min_scan_level);
    set_u64(table, "poll_interval_ms", edit.poll_interval_ms);
}

fn apply_log_levels(document: &mut DocumentMut, edit: &LogLevelConfigEdit) {
    let table = section(document, "log_levels");
    set_u8(table, "scan_incoming", edit.scan_incoming);
    set_u8(table, "scan_easy", edit.scan_easy);
    set_u8(table, "scan_hard", edit.scan_hard);
    set_u8(table, "kill_easy", edit.kill_easy);
    set_u8(table, "kill_hard", edit.kill_hard);
    set_u8(table, "fighter_hull", edit.fighter_hull);
    set_u8(table, "fighter_down", edit.fighter_down);
    set_u8(table, "ship_shields", edit.ship_shields);
    set_u8(table, "ship_hull", edit.ship_hull);
    set_u8(table, "died", edit.died);
    set_u8(table, "cargo_lost", edit.cargo_lost);
    set_u8(table, "bait_value_low", edit.bait_value_low);
    set_u8(table, "security_scan", edit.security_scan);
    set_u8(table, "security_attack", edit.security_attack);
    set_u8(table, "fuel_report", edit.fuel_report);
    set_u8(table, "fuel_low", edit.fuel_low);
    set_u8(table, "fuel_critical", edit.fuel_critical);
    set_u8(table, "missions", edit.missions);
    set_u8(table, "missions_all", edit.missions_all);
    set_u8(table, "merits", edit.merits);
    set_u8(table, "rank_promotion", edit.rank_promotion);
    set_u8(table, "no_kills", edit.no_kills);
    set_u8(table, "kill_rate", edit.kill_rate);
    set_u8(table, "summary_kills", edit.summary_kills);
    set_u8(table, "summary_faction", edit.summary_faction);
    set_u8(table, "summary_scans", edit.summary_scans);
    set_u8(table, "summary_bounties", edit.summary_bounties);
    set_u8(table, "summary_merits", edit.summary_merits);
    set_u8(table, "duplicate_suppression", edit.duplicate_suppression);
}

fn apply_matrix(document: &mut DocumentMut, edit: &MatrixConfigEdit) {
    let table = section(document, "matrix");
    table["enabled"] = value(edit.enabled);
    set_optional_string(table, "homeserver", edit.homeserver.as_deref());
    set_optional_string(table, "user_id", edit.user_id.as_deref());
    set_optional_string(table, "room_id", edit.room_id.as_deref());
    set_optional_string(table, "mention_user_id", edit.mention_user_id.as_deref());
    table["status_update_interval_seconds"] = value(edit.status_update_interval_seconds as i64);
    if edit.clear_access_token {
        table.remove("access_token");
    } else if let Some(token) = &edit.access_token_replacement {
        table["access_token"] = value(token.as_str());
    }
}

fn apply_web(document: &mut DocumentMut, edit: &WebConfigEdit) {
    let table = section(document, "web");
    set_bool(table, "enabled", edit.enabled);
    set_string(table, "host", edit.host.as_deref());
    set_u16(table, "port", edit.port);
    set_bool(table, "open_browser", edit.open_browser);
}

fn section<'a>(document: &'a mut DocumentMut, name: &str) -> &'a mut Table {
    let item = document
        .as_table_mut()
        .entry(name)
        .or_insert_with(|| Item::Table(Table::new()));
    loop {
        match item {
            Item::Table(table) => return table,
            _ => *item = Item::Table(Table::new()),
        }
    }
}

fn set_string(table: &mut Table, key: &str, value_in: Option<&str>) {
    if let Some(value_in) = value_in {
        table[key] = value(value_in);
    }
}

fn set_optional_string(table: &mut Table, key: &str, value_in: Option<&str>) {
    match value_in {
        Some(value_in) => table[key] = value(value_in),
        None => {
            table.remove(key);
        }
    }
}

fn set_bool(table: &mut Table, key: &str, value_in: Option<bool>) {
    if let Some(value_in) = value_in {
        table[key] = value(value_in);
    }
}

fn set_u8(table: &mut Table, key: &str, value_in: Option<u8>) {
    if let Some(value_in) = value_in {
        table[key] = value(i64::from(value_in));
    }
}

fn set_u16(table: &mut Table, key: &str, value_in: Option<u16>) {
    if let Some(value_in) = value_in {
        table[key] = value(i64::from(value_in));
    }
}

fn set_u64(table: &mut Table, key: &str, value_in: Option<u64>) {
    if let Some(value_in) = value_in {
        table[key] = value(value_in as i64);
    }
}

fn is_loopback_host(host: &str) -> bool {
    matches!(host.trim(), "127.0.0.1" | "localhost" | "::1" | "[::1]")
}
