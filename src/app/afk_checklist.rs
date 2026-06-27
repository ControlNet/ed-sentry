use serde::{Deserialize, Deserializer, Serialize};

const HARDPOINTS_DEPLOYED_FLAG: u64 = 0x40;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AfkChecklistState {
    hardpoints_deployed: ChecklistRow,
    engine_pips_zero: ChecklistRow,
    cargo_loaded: ChecklistRow,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ChecklistRow {
    state: ChecklistRowState,
    source: ChecklistRowSource,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ChecklistRowState {
    Pass,
    Fail,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum ChecklistRowSource {
    #[serde(rename = "Status.json")]
    StatusJson,
    #[serde(rename = "Cargo.json")]
    CargoJson,
    #[serde(rename = "unknown")]
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct AfkChecklistView {
    pub rows: Vec<AfkChecklistRowView>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct AfkChecklistRowView {
    pub id: &'static str,
    pub label: &'static str,
    pub detail: &'static str,
    pub state: ChecklistRowState,
    pub source: ChecklistRowSource,
}

#[derive(Deserialize)]
struct StatusFile {
    #[serde(default, deserialize_with = "optional_u64")]
    #[serde(rename = "Flags")]
    flags: Option<u64>,
    #[serde(default, deserialize_with = "optional_u64_vec")]
    #[serde(rename = "Pips")]
    pips: Option<Vec<u64>>,
}

#[derive(Deserialize)]
struct CargoFile {
    #[serde(default, deserialize_with = "optional_string")]
    #[serde(rename = "Vessel")]
    vessel: Option<String>,
    #[serde(default, deserialize_with = "optional_u64")]
    #[serde(rename = "Count")]
    count: Option<u64>,
    #[serde(default, deserialize_with = "optional_inventory")]
    #[serde(rename = "Inventory")]
    inventory: Option<Vec<CargoInventoryItem>>,
}

#[derive(Deserialize)]
struct CargoInventoryItem {
    #[serde(rename = "Name")]
    _name: Option<String>,
    #[serde(rename = "Name_Localised")]
    _name_localised: Option<String>,
    #[serde(rename = "Count")]
    _count: Option<u64>,
    #[serde(rename = "Stolen")]
    _stolen: Option<u64>,
    #[serde(rename = "MissionID")]
    _mission_id: Option<u64>,
}

fn optional_u64<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Option::<u64>::deserialize(deserializer).ok().flatten())
}

fn optional_u64_vec<'de, D>(deserializer: D) -> Result<Option<Vec<u64>>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Option::<Vec<u64>>::deserialize(deserializer).ok().flatten())
}

fn optional_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Option::<String>::deserialize(deserializer).ok().flatten())
}

fn optional_inventory<'de, D>(deserializer: D) -> Result<Option<Vec<CargoInventoryItem>>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Option::<Vec<CargoInventoryItem>>::deserialize(deserializer)
        .ok()
        .flatten())
}

impl AfkChecklistState {
    pub fn from_companion_json(status_json: &str, cargo_json: &str) -> Self {
        let status = serde_json::from_str::<StatusFile>(status_json).ok();
        let cargo = serde_json::from_str::<CargoFile>(cargo_json).ok();

        Self {
            hardpoints_deployed: hardpoints_deployed(status.as_ref()),
            engine_pips_zero: engine_pips_zero(status.as_ref()),
            cargo_loaded: cargo_loaded(cargo.as_ref()),
        }
    }

    pub fn from_optional_companion_json(
        status_json: Option<&str>,
        cargo_json: Option<&str>,
    ) -> Self {
        match (status_json, cargo_json) {
            (Some(status), Some(cargo)) => Self::from_companion_json(status, cargo),
            (Some(status), None) => Self::from_companion_json(status, ""),
            (None, Some(cargo)) => Self::from_companion_json("", cargo),
            (None, None) => Self::unknown(),
        }
    }

    pub fn unknown() -> Self {
        let unknown = ChecklistRow::unknown();

        Self {
            hardpoints_deployed: unknown.clone(),
            engine_pips_zero: unknown.clone(),
            cargo_loaded: unknown,
        }
    }

    pub fn to_view(&self) -> AfkChecklistView {
        AfkChecklistView {
            rows: vec![
                AfkChecklistRowView {
                    id: "hardpoints_deployed",
                    label: "Hardpoints deployed",
                    detail: "Status Flags bit 0x40 is set",
                    state: self.hardpoints_deployed.state,
                    source: self.hardpoints_deployed.source,
                },
                AfkChecklistRowView {
                    id: "engine_pips_zero",
                    label: "Engine pips zero",
                    detail: "Status Pips[1] is 0",
                    state: self.engine_pips_zero.state,
                    source: self.engine_pips_zero.source,
                },
                AfkChecklistRowView {
                    id: "cargo_loaded",
                    label: "Cargo loaded",
                    detail: "Cargo.json Ship cargo is non-empty",
                    state: self.cargo_loaded.state,
                    source: self.cargo_loaded.source,
                },
            ],
        }
    }

    pub fn apply_status_json(&mut self, status_json: Option<&str>) -> bool {
        let status = status_json.and_then(|json| serde_json::from_str::<StatusFile>(json).ok());
        let next_hardpoints = hardpoints_deployed(status.as_ref());
        let next_engine_pips = engine_pips_zero(status.as_ref());
        let changed = self.hardpoints_deployed != next_hardpoints
            || self.engine_pips_zero != next_engine_pips;

        self.hardpoints_deployed = next_hardpoints;
        self.engine_pips_zero = next_engine_pips;
        changed
    }

    pub fn apply_cargo_json(&mut self, cargo_json: Option<&str>) -> bool {
        let cargo = cargo_json.and_then(|json| serde_json::from_str::<CargoFile>(json).ok());
        let next_cargo = cargo_loaded(cargo.as_ref());
        let changed = self.cargo_loaded != next_cargo;

        self.cargo_loaded = next_cargo;
        changed
    }
}

impl ChecklistRow {
    const fn known(state: ChecklistRowState, source: ChecklistRowSource) -> Self {
        Self { state, source }
    }

    const fn unknown() -> Self {
        Self::known(ChecklistRowState::Unknown, ChecklistRowSource::Unknown)
    }
}

fn hardpoints_deployed(status: Option<&StatusFile>) -> ChecklistRow {
    let Some(flags) = status.and_then(|status| status.flags) else {
        return ChecklistRow::unknown();
    };

    if flags & HARDPOINTS_DEPLOYED_FLAG != 0 {
        ChecklistRow::known(ChecklistRowState::Pass, ChecklistRowSource::StatusJson)
    } else {
        ChecklistRow::known(ChecklistRowState::Fail, ChecklistRowSource::StatusJson)
    }
}

fn engine_pips_zero(status: Option<&StatusFile>) -> ChecklistRow {
    let Some(engine_pips) = status
        .and_then(|status| status.pips.as_ref())
        .and_then(|pips| pips.get(1))
    else {
        return ChecklistRow::unknown();
    };

    if *engine_pips == 0 {
        ChecklistRow::known(ChecklistRowState::Pass, ChecklistRowSource::StatusJson)
    } else {
        ChecklistRow::known(ChecklistRowState::Fail, ChecklistRowSource::StatusJson)
    }
}

fn cargo_loaded(cargo: Option<&CargoFile>) -> ChecklistRow {
    let Some(cargo) = cargo else {
        return ChecklistRow::unknown();
    };
    if cargo.vessel.as_deref() != Some("Ship") {
        return ChecklistRow::unknown();
    }
    if cargo.count.is_some_and(|count| count > 0)
        || cargo
            .inventory
            .as_ref()
            .is_some_and(|inventory| !inventory.is_empty())
    {
        return ChecklistRow::known(ChecklistRowState::Pass, ChecklistRowSource::CargoJson);
    }
    match cargo.count {
        Some(0) => ChecklistRow::known(ChecklistRowState::Fail, ChecklistRowSource::CargoJson),
        Some(_) => ChecklistRow::known(ChecklistRowState::Pass, ChecklistRowSource::CargoJson),
        None => ChecklistRow::unknown(),
    }
}

#[cfg(test)]
mod tests;
