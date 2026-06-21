use toml::Value;

use super::value_read::read_u8;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogLevelConfig {
    pub scan_incoming: u8,
    pub scan_easy: u8,
    pub scan_hard: u8,
    pub kill_easy: u8,
    pub kill_hard: u8,
    pub fighter_hull: u8,
    pub fighter_down: u8,
    pub ship_shields: u8,
    pub ship_hull: u8,
    pub died: u8,
    pub cargo_lost: u8,
    pub bait_value_low: u8,
    pub security_scan: u8,
    pub security_attack: u8,
    pub fuel_report: u8,
    pub fuel_low: u8,
    pub fuel_critical: u8,
    pub missions: u8,
    pub missions_all: u8,
    pub merits: u8,
    pub rank_promotion: u8,
    pub no_kills: u8,
    pub kill_rate: u8,
    pub summary_kills: u8,
    pub summary_faction: u8,
    pub summary_scans: u8,
    pub summary_bounties: u8,
    pub summary_merits: u8,
    pub duplicate_suppression: u8,
}

impl Default for LogLevelConfig {
    fn default() -> Self {
        Self {
            scan_incoming: 1,
            scan_easy: 1,
            scan_hard: 1,
            kill_easy: 1,
            kill_hard: 1,
            fighter_hull: 1,
            fighter_down: 2,
            ship_shields: 1,
            ship_hull: 1,
            died: 2,
            cargo_lost: 2,
            bait_value_low: 1,
            security_scan: 1,
            security_attack: 1,
            fuel_report: 1,
            fuel_low: 2,
            fuel_critical: 2,
            missions: 1,
            missions_all: 2,
            merits: 0,
            rank_promotion: 2,
            no_kills: 2,
            kill_rate: 1,
            summary_kills: 1,
            summary_faction: 0,
            summary_scans: 0,
            summary_bounties: 1,
            summary_merits: 1,
            duplicate_suppression: 1,
        }
    }
}

pub(super) fn read_log_levels(
    table: &toml::map::Map<String, Value>,
    log_levels: &mut LogLevelConfig,
    warnings: &mut Vec<String>,
) {
    read_u8(
        table.get("scan_incoming"),
        "log_levels.scan_incoming",
        &mut log_levels.scan_incoming,
        warnings,
    );
    read_u8(
        table.get("scan_easy"),
        "log_levels.scan_easy",
        &mut log_levels.scan_easy,
        warnings,
    );
    read_u8(
        table.get("scan_hard"),
        "log_levels.scan_hard",
        &mut log_levels.scan_hard,
        warnings,
    );
    read_u8(
        table.get("kill_easy"),
        "log_levels.kill_easy",
        &mut log_levels.kill_easy,
        warnings,
    );
    read_u8(
        table.get("kill_hard"),
        "log_levels.kill_hard",
        &mut log_levels.kill_hard,
        warnings,
    );
    read_u8(
        table.get("fighter_hull"),
        "log_levels.fighter_hull",
        &mut log_levels.fighter_hull,
        warnings,
    );
    read_u8(
        table.get("fighter_down"),
        "log_levels.fighter_down",
        &mut log_levels.fighter_down,
        warnings,
    );
    read_u8(
        table.get("ship_shields"),
        "log_levels.ship_shields",
        &mut log_levels.ship_shields,
        warnings,
    );
    read_u8(
        table.get("ship_hull"),
        "log_levels.ship_hull",
        &mut log_levels.ship_hull,
        warnings,
    );
    read_u8(
        table.get("died"),
        "log_levels.died",
        &mut log_levels.died,
        warnings,
    );
    read_u8(
        table.get("cargo_lost"),
        "log_levels.cargo_lost",
        &mut log_levels.cargo_lost,
        warnings,
    );
    read_u8(
        table.get("bait_value_low"),
        "log_levels.bait_value_low",
        &mut log_levels.bait_value_low,
        warnings,
    );
    read_u8(
        table.get("security_scan"),
        "log_levels.security_scan",
        &mut log_levels.security_scan,
        warnings,
    );
    read_u8(
        table.get("security_attack"),
        "log_levels.security_attack",
        &mut log_levels.security_attack,
        warnings,
    );
    read_u8(
        table.get("fuel_report"),
        "log_levels.fuel_report",
        &mut log_levels.fuel_report,
        warnings,
    );
    read_u8(
        table.get("fuel_low"),
        "log_levels.fuel_low",
        &mut log_levels.fuel_low,
        warnings,
    );
    read_u8(
        table.get("fuel_critical"),
        "log_levels.fuel_critical",
        &mut log_levels.fuel_critical,
        warnings,
    );
    read_u8(
        table.get("missions"),
        "log_levels.missions",
        &mut log_levels.missions,
        warnings,
    );
    read_u8(
        table.get("missions_all"),
        "log_levels.missions_all",
        &mut log_levels.missions_all,
        warnings,
    );
    read_u8(
        table.get("merits"),
        "log_levels.merits",
        &mut log_levels.merits,
        warnings,
    );
    read_u8(
        table.get("rank_promotion"),
        "log_levels.rank_promotion",
        &mut log_levels.rank_promotion,
        warnings,
    );
    read_u8(
        table.get("no_kills"),
        "log_levels.no_kills",
        &mut log_levels.no_kills,
        warnings,
    );
    read_u8(
        table.get("kill_rate"),
        "log_levels.kill_rate",
        &mut log_levels.kill_rate,
        warnings,
    );
    read_u8(
        table.get("summary_kills"),
        "log_levels.summary_kills",
        &mut log_levels.summary_kills,
        warnings,
    );
    read_u8(
        table.get("summary_faction"),
        "log_levels.summary_faction",
        &mut log_levels.summary_faction,
        warnings,
    );
    read_u8(
        table.get("summary_scans"),
        "log_levels.summary_scans",
        &mut log_levels.summary_scans,
        warnings,
    );
    read_u8(
        table.get("summary_bounties"),
        "log_levels.summary_bounties",
        &mut log_levels.summary_bounties,
        warnings,
    );
    read_u8(
        table.get("summary_merits"),
        "log_levels.summary_merits",
        &mut log_levels.summary_merits,
        warnings,
    );
    read_u8(
        table.get("duplicate_suppression"),
        "log_levels.duplicate_suppression",
        &mut log_levels.duplicate_suppression,
        warnings,
    );
}
