use toml_edit::{value, DocumentMut, Item, Table};

use crate::app::{
    EditableConfigUpdate, JournalConfigEdit, LogLevelConfigEdit, MatrixConfigEdit,
    MonitorConfigEdit, TunnelConfigEdit, WebConfigEdit,
};

pub(super) fn validate_update(
    update: &EditableConfigUpdate,
) -> Result<(), super::ConfigWriteError> {
    if let Some(journal) = &update.journal {
        validate_opt_range(
            journal.recent_files,
            1..=100,
            "journal.recent_files must be between 1 and 100",
        )?;
    }
    if let Some(monitor) = &update.monitor {
        validate_opt_range(
            monitor.warn_kill_rate,
            0..=1_000,
            "monitor.warn_kill_rate must be between 0 and 1000",
        )?;
        validate_opt_range(
            monitor.duplicate_max,
            0..=1_000,
            "monitor.duplicate_max must be between 0 and 1000",
        )?;
        validate_opt_range(
            monitor.min_scan_level,
            0..=3,
            "monitor.min_scan_level must be between 0 and 3",
        )?;
        if let Some(poll_interval_ms) = monitor.poll_interval_ms {
            if i64::try_from(poll_interval_ms).is_err()
                || !(100..=60_000).contains(&poll_interval_ms)
            {
                return Err(super::ConfigWriteError::InvalidUpdate {
                    reason: "monitor.poll_interval_ms must be between 100 and 60000",
                });
            }
        }
    }
    if let Some(log_levels) = &update.log_levels {
        validate_log_levels(log_levels)?;
    }
    if let Some(matrix) = &update.matrix {
        if matrix.clear_access_token && matrix.access_token_replacement.is_some() {
            return Err(super::ConfigWriteError::InvalidUpdate {
                reason: "matrix access token cannot be replaced and cleared in one update",
            });
        }
        if i64::try_from(matrix.status_update_interval_seconds).is_err()
            || !(10..=86_400).contains(&matrix.status_update_interval_seconds)
        {
            return Err(super::ConfigWriteError::InvalidUpdate {
                reason: "matrix.status_update_interval_seconds must be between 10 and 86400",
            });
        }
    }
    if update.tunnel.as_ref().is_some_and(|tunnel| {
        tunnel.clear_config_password && tunnel.config_password_replacement.is_some()
    }) {
        return Err(super::ConfigWriteError::InvalidUpdate {
            reason: "tunnel config password cannot be replaced and cleared in one update",
        });
    }
    if let Some(web) = &update.web {
        validate_opt_range(web.port, 1..=65_535, "web.port must be between 1 and 65535")?;
        if web
            .host
            .as_deref()
            .is_some_and(|host| host.trim().is_empty())
        {
            return Err(super::ConfigWriteError::InvalidUpdate {
                reason: "web.host must not be empty",
            });
        }
    }
    Ok(())
}

fn validate_opt_range<T: PartialOrd + Copy>(
    value: Option<T>,
    range: std::ops::RangeInclusive<T>,
    reason: &'static str,
) -> Result<(), super::ConfigWriteError> {
    if value.is_some_and(|candidate| !range.contains(&candidate)) {
        return Err(super::ConfigWriteError::InvalidUpdate { reason });
    }
    Ok(())
}

fn validate_log_levels(edit: &LogLevelConfigEdit) -> Result<(), super::ConfigWriteError> {
    let levels = [
        edit.scan_incoming,
        edit.scan_easy,
        edit.scan_hard,
        edit.kill_easy,
        edit.kill_hard,
        edit.fighter_hull,
        edit.fighter_down,
        edit.ship_shields,
        edit.ship_hull,
        edit.died,
        edit.cargo_lost,
        edit.bait_value_low,
        edit.security_scan,
        edit.security_attack,
        edit.fuel_report,
        edit.fuel_low,
        edit.fuel_critical,
        edit.missions,
        edit.missions_all,
        edit.merits,
        edit.rank_promotion,
        edit.no_kills,
        edit.kill_rate,
        edit.summary_kills,
        edit.summary_faction,
        edit.summary_scans,
        edit.summary_bounties,
        edit.summary_merits,
        edit.duplicate_suppression,
    ];
    for level in levels {
        validate_opt_range(level, 0..=5, "log level must be between 0 and 5")?;
    }
    Ok(())
}

pub(super) fn apply_update(
    document: &mut DocumentMut,
    update: &EditableConfigUpdate,
) -> Result<(), super::ConfigWriteError> {
    if let Some(journal) = &update.journal {
        apply_journal(document, journal);
    }
    if let Some(monitor) = &update.monitor {
        apply_monitor(document, monitor)?;
    }
    if let Some(log_levels) = &update.log_levels {
        apply_log_levels(document, log_levels);
    }
    if let Some(matrix) = &update.matrix {
        apply_matrix(document, matrix)?;
    }
    if let Some(tunnel) = &update.tunnel {
        apply_tunnel(document, tunnel);
    }
    if let Some(web) = &update.web {
        apply_web(document, web);
    }
    Ok(())
}

fn apply_journal(document: &mut DocumentMut, edit: &JournalConfigEdit) {
    let table = section(document, "journal");
    set_string(table, "folder", edit.folder.as_deref());
    set_u16(table, "recent_files", edit.recent_files);
}

fn apply_monitor(
    document: &mut DocumentMut,
    edit: &MonitorConfigEdit,
) -> Result<(), super::ConfigWriteError> {
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
    set_u64(table, "poll_interval_ms", edit.poll_interval_ms)?;
    Ok(())
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

fn apply_matrix(
    document: &mut DocumentMut,
    edit: &MatrixConfigEdit,
) -> Result<(), super::ConfigWriteError> {
    let table = section(document, "matrix");
    table["enabled"] = value(edit.enabled);
    set_optional_string(table, "homeserver", edit.homeserver.as_deref());
    set_optional_string(table, "room_id", edit.room_id.as_deref());
    set_optional_string(table, "mention_user_id", edit.mention_user_id.as_deref());
    let interval = i64::try_from(edit.status_update_interval_seconds).map_err(|_| {
        super::ConfigWriteError::InvalidUpdate {
            reason: "matrix.status_update_interval_seconds overflows i64",
        }
    })?;
    table["status_update_interval_seconds"] = value(interval);
    if edit.clear_access_token {
        table.remove("access_token");
    } else if let Some(token) = &edit.access_token_replacement {
        table["access_token"] = value(token.as_str());
    }
    Ok(())
}

fn apply_tunnel(document: &mut DocumentMut, edit: &TunnelConfigEdit) {
    let table = section(document, "tunnel");
    set_string(table, "provider", edit.provider.as_deref());
    set_bool(table, "auto_start", edit.auto_start);
    if edit.clear_config_password {
        table.remove("config_password");
    } else if let Some(password) = &edit.config_password_replacement {
        table["config_password"] = value(password.as_str());
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

fn set_u64(
    table: &mut Table,
    key: &str,
    value_in: Option<u64>,
) -> Result<(), super::ConfigWriteError> {
    if let Some(value_in) = value_in {
        let converted =
            i64::try_from(value_in).map_err(|_| super::ConfigWriteError::InvalidUpdate {
                reason: "u64 config value overflows i64",
            })?;
        table[key] = value(converted);
    }
    Ok(())
}
