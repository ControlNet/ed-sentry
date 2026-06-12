use std::fmt;

use chrono::{DateTime, Utc};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BasicJournalEvent {
    pub timestamp: DateTime<Utc>,
    pub event: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommanderEvent {
    pub timestamp: DateTime<Utc>,
    pub event: String,
    pub name: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoadGameEvent {
    pub timestamp: DateTime<Utc>,
    pub event: String,
    pub commander: Option<String>,
    pub ship: Option<String>,
    pub ship_localised: Option<String>,
    pub game_mode: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LoadoutEvent {
    pub timestamp: DateTime<Utc>,
    pub event: String,
    pub ship: Option<String>,
    pub ship_localised: Option<String>,
    pub fuel_capacity_main: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RankEvent {
    pub timestamp: DateTime<Utc>,
    pub event: String,
    pub combat: Option<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProgressEvent {
    pub timestamp: DateTime<Utc>,
    pub event: String,
    pub combat: Option<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocationEvent {
    pub timestamp: DateTime<Utc>,
    pub event: String,
    pub star_system: Option<String>,
    pub body: Option<String>,
    pub body_type: Option<String>,
    pub docked: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SupercruiseDestinationDropEvent {
    pub timestamp: DateTime<Utc>,
    pub event: String,
    pub destination_type: Option<String>,
    pub destination_type_localised: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TravelEvent {
    pub timestamp: DateTime<Utc>,
    pub event: String,
    pub star_system: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaunchFighterEvent {
    pub timestamp: DateTime<Utc>,
    pub event: String,
    pub player_controlled: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReceiveTextEvent {
    pub timestamp: DateTime<Utc>,
    pub event: String,
    pub from: Option<String>,
    pub from_localised: Option<String>,
    pub message: Option<String>,
    pub channel: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShipTargetedEvent {
    pub timestamp: DateTime<Utc>,
    pub event: String,
    pub target_locked: Option<bool>,
    pub scan_stage: Option<u8>,
    pub ship: Option<String>,
    pub ship_localised: Option<String>,
    pub pilot_name: Option<String>,
    pub pilot_name_localised: Option<String>,
    pub pilot_rank: Option<String>,
    pub legal_status: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct BountyReward {
    #[serde(rename = "Faction")]
    pub faction: Option<String>,
    #[serde(rename = "Reward")]
    pub reward: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BountyEvent {
    pub timestamp: DateTime<Utc>,
    pub event: String,
    pub total_reward: Option<u64>,
    pub rewards: Option<Vec<BountyReward>>,
    pub victim_faction: Option<String>,
    pub victim_faction_localised: Option<String>,
    pub target: Option<String>,
    pub target_localised: Option<String>,
    pub pilot_name_localised: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FactionKillBondEvent {
    pub timestamp: DateTime<Utc>,
    pub event: String,
    pub reward: Option<u64>,
    pub awarding_faction: Option<String>,
    pub victim_faction: Option<String>,
    pub victim_faction_localised: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MissionEvent {
    pub timestamp: DateTime<Utc>,
    pub event: String,
    pub mission_id: Option<u64>,
    pub name: Option<String>,
    pub localised_name: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MissionListItem {
    pub mission_id: Option<u64>,
    pub name: Option<String>,
    pub expires: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MissionsEvent {
    pub timestamp: DateTime<Utc>,
    pub event: String,
    pub active: Vec<MissionListItem>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShieldStateEvent {
    pub timestamp: DateTime<Utc>,
    pub event: String,
    pub shields_up: Option<bool>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HullDamageEvent {
    pub timestamp: DateTime<Utc>,
    pub event: String,
    pub health: Option<f64>,
    pub player_pilot: Option<bool>,
    pub fighter: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EjectCargoEvent {
    pub timestamp: DateTime<Utc>,
    pub event: String,
    pub cargo_type: Option<String>,
    pub cargo_type_localised: Option<String>,
    pub count: Option<u64>,
    pub abandoned: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShipyardSwapEvent {
    pub timestamp: DateTime<Utc>,
    pub event: String,
    pub ship_type: Option<String>,
    pub ship_type_localised: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ReservoirReplenishedEvent {
    pub timestamp: DateTime<Utc>,
    pub event: String,
    pub fuel_main: Option<f64>,
    pub fuel_reservoir: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PowerplayMeritsEvent {
    pub timestamp: DateTime<Utc>,
    pub event: String,
    pub merits_gained: Option<u64>,
    pub power: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MusicEvent {
    pub timestamp: DateTime<Utc>,
    pub event: String,
    pub music_track: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RawJournalEvent {
    pub timestamp: DateTime<Utc>,
    pub event: String,
    pub raw: Value,
}

#[derive(Clone, Debug, PartialEq)]
pub enum JournalEvent {
    Commander(CommanderEvent),
    LoadGame(LoadGameEvent),
    Loadout(LoadoutEvent),
    Rank(RankEvent),
    Progress(ProgressEvent),
    Location(LocationEvent),
    SupercruiseDestinationDrop(SupercruiseDestinationDropEvent),
    SupercruiseEntry(TravelEvent),
    FSDJump(TravelEvent),
    ReceiveText(ReceiveTextEvent),
    ShipTargeted(ShipTargetedEvent),
    Bounty(BountyEvent),
    FactionKillBond(FactionKillBondEvent),
    MissionRedirected(MissionEvent),
    Missions(MissionsEvent),
    MissionAccepted(MissionEvent),
    MissionCompleted(MissionEvent),
    MissionFailed(MissionEvent),
    MissionAbandoned(MissionEvent),
    ShieldState(ShieldStateEvent),
    HullDamage(HullDamageEvent),
    FighterDestroyed(BasicJournalEvent),
    LaunchFighter(LaunchFighterEvent),
    StartJump(BasicJournalEvent),
    EjectCargo(EjectCargoEvent),
    ReservoirReplenished(ReservoirReplenishedEvent),
    PowerplayMerits(PowerplayMeritsEvent),
    Music(MusicEvent),
    ShipyardSwap(ShipyardSwapEvent),
    Shutdown(BasicJournalEvent),
    Died(BasicJournalEvent),
    StartupSnapshot(RawJournalEvent),
    Station(RawJournalEvent),
    Exploration(RawJournalEvent),
    Navigation(RawJournalEvent),
    CargoMaterial(RawJournalEvent),
    ShipModule(RawJournalEvent),
    MissionDetail(RawJournalEvent),
    CombatDetail(RawJournalEvent),
    Odyssey(RawJournalEvent),
    Social(RawJournalEvent),
    Powerplay(RawJournalEvent),
    Squadron(RawJournalEvent),
    Carrier(RawJournalEvent),
    Colonisation(RawJournalEvent),
    Unknown {
        timestamp: DateTime<Utc>,
        event: String,
        raw: Value,
    },
}

#[derive(Debug)]
pub enum JournalParseError {
    MalformedJson {
        source: serde_json::Error,
    },
    MissingTimestamp,
    InvalidTimestamp {
        value: String,
        source: chrono::ParseError,
    },
    MissingEvent,
    InvalidEventFields {
        event: String,
        source: serde_json::Error,
    },
}

impl JournalEvent {
    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            Self::Commander(event) => event.timestamp,
            Self::LoadGame(event) => event.timestamp,
            Self::Loadout(event) => event.timestamp,
            Self::Location(event) => event.timestamp,
            Self::SupercruiseDestinationDrop(event) => event.timestamp,
            Self::Rank(event) => event.timestamp,
            Self::Progress(event) => event.timestamp,
            Self::SupercruiseEntry(event) => event.timestamp,
            Self::FSDJump(event) => event.timestamp,
            Self::FighterDestroyed(event)
            | Self::StartJump(event)
            | Self::Shutdown(event)
            | Self::Died(event) => event.timestamp,
            Self::LaunchFighter(event) => event.timestamp,
            Self::Missions(event) => event.timestamp,
            Self::ReceiveText(event) => event.timestamp,
            Self::ShipTargeted(event) => event.timestamp,
            Self::Bounty(event) => event.timestamp,
            Self::FactionKillBond(event) => event.timestamp,
            Self::MissionRedirected(event)
            | Self::MissionAccepted(event)
            | Self::MissionCompleted(event)
            | Self::MissionFailed(event)
            | Self::MissionAbandoned(event) => event.timestamp,
            Self::ShieldState(event) => event.timestamp,
            Self::HullDamage(event) => event.timestamp,
            Self::EjectCargo(event) => event.timestamp,
            Self::ReservoirReplenished(event) => event.timestamp,
            Self::PowerplayMerits(event) => event.timestamp,
            Self::Music(event) => event.timestamp,
            Self::ShipyardSwap(event) => event.timestamp,
            Self::StartupSnapshot(event)
            | Self::Station(event)
            | Self::Exploration(event)
            | Self::Navigation(event)
            | Self::CargoMaterial(event)
            | Self::ShipModule(event)
            | Self::MissionDetail(event)
            | Self::CombatDetail(event)
            | Self::Odyssey(event)
            | Self::Social(event)
            | Self::Powerplay(event)
            | Self::Squadron(event)
            | Self::Carrier(event)
            | Self::Colonisation(event) => event.timestamp,
            Self::Unknown { timestamp, .. } => *timestamp,
        }
    }

    pub fn event_name(&self) -> &str {
        match self {
            Self::Commander(event) => &event.event,
            Self::LoadGame(event) => &event.event,
            Self::Loadout(event) => &event.event,
            Self::Location(event) => &event.event,
            Self::SupercruiseDestinationDrop(event) => &event.event,
            Self::Rank(event) => &event.event,
            Self::Progress(event) => &event.event,
            Self::SupercruiseEntry(event) => &event.event,
            Self::FSDJump(event) => &event.event,
            Self::FighterDestroyed(event)
            | Self::StartJump(event)
            | Self::Shutdown(event)
            | Self::Died(event) => &event.event,
            Self::LaunchFighter(event) => &event.event,
            Self::Missions(event) => &event.event,
            Self::ReceiveText(event) => &event.event,
            Self::ShipTargeted(event) => &event.event,
            Self::Bounty(event) => &event.event,
            Self::FactionKillBond(event) => &event.event,
            Self::MissionRedirected(event)
            | Self::MissionAccepted(event)
            | Self::MissionCompleted(event)
            | Self::MissionFailed(event)
            | Self::MissionAbandoned(event) => &event.event,
            Self::ShieldState(event) => &event.event,
            Self::HullDamage(event) => &event.event,
            Self::EjectCargo(event) => &event.event,
            Self::ReservoirReplenished(event) => &event.event,
            Self::PowerplayMerits(event) => &event.event,
            Self::Music(event) => &event.event,
            Self::ShipyardSwap(event) => &event.event,
            Self::StartupSnapshot(event)
            | Self::Station(event)
            | Self::Exploration(event)
            | Self::Navigation(event)
            | Self::CargoMaterial(event)
            | Self::ShipModule(event)
            | Self::MissionDetail(event)
            | Self::CombatDetail(event)
            | Self::Odyssey(event)
            | Self::Social(event)
            | Self::Powerplay(event)
            | Self::Squadron(event)
            | Self::Carrier(event)
            | Self::Colonisation(event) => &event.event,
            Self::Unknown { event, .. } => event,
        }
    }

    pub fn raw_payload(&self) -> Option<&Value> {
        match self {
            Self::StartupSnapshot(event)
            | Self::Station(event)
            | Self::Exploration(event)
            | Self::Navigation(event)
            | Self::CargoMaterial(event)
            | Self::ShipModule(event)
            | Self::MissionDetail(event)
            | Self::CombatDetail(event)
            | Self::Odyssey(event)
            | Self::Social(event)
            | Self::Powerplay(event)
            | Self::Squadron(event)
            | Self::Carrier(event)
            | Self::Colonisation(event) => Some(&event.raw),
            Self::Unknown { raw, .. } => Some(raw),
            _ => None,
        }
    }
}

impl fmt::Display for JournalParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MalformedJson { source } => write!(formatter, "malformed journal JSON: {source}"),
            Self::MissingTimestamp => write!(formatter, "journal event is missing timestamp"),
            Self::InvalidTimestamp { value, source } => {
                write!(formatter, "invalid journal timestamp {value:?}: {source}")
            }
            Self::MissingEvent => write!(formatter, "journal event is missing event name"),
            Self::InvalidEventFields { event, source } => {
                write!(
                    formatter,
                    "invalid fields for journal event {event}: {source}"
                )
            }
        }
    }
}

impl std::error::Error for JournalParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::MalformedJson { source } => Some(source),
            Self::InvalidTimestamp { source, .. } => Some(source),
            Self::InvalidEventFields { source, .. } => Some(source),
            Self::MissingTimestamp | Self::MissingEvent => None,
        }
    }
}

const STARTUP_SNAPSHOT_EVENTS: &[&str] = &[
    "Fileheader",
    "Statistics",
    "Materials",
    "Passengers",
    "Cargo",
    "StoredShips",
    "StoredModules",
    "Powerplay",
];

const STATION_EVENTS: &[&str] = &[
    "DockingRequested",
    "DockingGranted",
    "DockingDenied",
    "DockingCancelled",
    "DockingTimeout",
    "Docked",
    "Undocked",
    "Market",
    "MarketBuy",
    "MarketSell",
    "BuyTradeData",
    "Outfitting",
    "Shipyard",
    "Repair",
    "RepairAll",
    "RefuelAll",
    "RefuelPartial",
    "RestockVehicle",
    "SellDrones",
    "SearchAndRescue",
];

const EXPLORATION_EVENTS: &[&str] = &[
    "DiscoveryScan",
    "FSSDiscoveryScan",
    "FSSBodySignals",
    "FSSSignalDiscovered",
    "FSSAllBodiesFound",
    "SAASignalsFound",
    "SAAScanComplete",
    "Scan",
    "ScanBaryCentre",
    "ScanOrganic",
    "SellExplorationData",
    "MultiSellExplorationData",
    "BuyExplorationData",
    "SellOrganicData",
    "DataScanned",
    "DatalinkScan",
    "DatalinkVoucher",
    "CodexEntry",
    "NavBeaconScan",
];

const NAVIGATION_EVENTS: &[&str] = &[
    "FSDTarget",
    "SupercruiseExit",
    "ApproachBody",
    "LeaveBody",
    "NavRoute",
    "NavRouteClear",
    "Touchdown",
    "Liftoff",
];

const CARGO_MATERIAL_EVENTS: &[&str] = &[
    "CargoDepot",
    "CargoTransfer",
    "CollectCargo",
    "CollectItems",
    "DropItems",
    "MiningRefined",
    "MaterialCollected",
    "MaterialDiscarded",
    "MaterialDiscovered",
    "MaterialTrade",
    "Backpack",
    "BackpackChange",
    "ShipLocker",
    "ShipLockerMaterials",
    "TransferMicroResources",
    "BuyMicroResources",
    "SellMicroResources",
    "RequestPowerMicroResources",
    "DeliverPowerMicroResources",
    "FCMaterials",
    "UseConsumable",
];

const SHIP_MODULE_EVENTS: &[&str] = &[
    "AfmuRepairs",
    "ModuleBuy",
    "ModuleBuyAndStore",
    "ModuleStore",
    "ModuleRetrieve",
    "ModuleSell",
    "ModuleSellRemote",
    "ModuleSwap",
    "MassModuleStore",
    "FetchRemoteModule",
    "EngineerContribution",
    "EngineerCraft",
    "EngineerLegacyConvert",
    "EngineerProgress",
    "TechnologyBroker",
    "ShipyardBuy",
    "ShipyardSell",
    "ShipyardNew",
    "ShipyardTransfer",
];

const MISSION_DETAIL_EVENTS: &[&str] = &[
    "MissionFactionEffects",
    "CommunityGoal",
    "CommunityGoalJoin",
    "CommunityGoalDiscard",
    "CommunityGoalReward",
    "ScientificResearch",
];

const COMBAT_DETAIL_EVENTS: &[&str] = &[
    "CapShipBond",
    "PVPKill",
    "EscapeInterdiction",
    "Interdicted",
    "Interdiction",
    "UnderAttack",
    "HeatDamage",
    "HeatWarning",
    "CommitCrime",
    "CrimeVictim",
    "PayBounties",
    "PayFines",
    "RedeemVoucher",
    "SelfDestruct",
    "RebootRepair",
    "FighterRebuilt",
    "CockpitBreached",
    "SystemsShutdown",
    "Resurrect",
    "SRVDestroyed",
    "Scanned",
];

const ODYSSEY_EVENTS: &[&str] = &[
    "Embark",
    "Disembark",
    "ApproachSettlement",
    "BookDropship",
    "CancelDropship",
    "BookTaxi",
    "CancelTaxi",
    "DropshipDeploy",
    "LaunchSRV",
    "DockSRV",
    "VehicleSwitch",
    "CreateSuitLoadout",
    "DeleteSuitLoadout",
    "RenameSuitLoadout",
    "SuitLoadout",
    "SwitchSuitLoadout",
    "LoadoutEquipModule",
    "LoadoutRemoveModule",
    "BuySuit",
    "BuyWeapon",
    "BuyAmmo",
    "BuyDrones",
    "SellSuit",
    "SellWeapon",
    "UpgradeSuit",
    "UpgradeWeapon",
    "USSDrop",
];

const SOCIAL_EVENTS: &[&str] = &[
    "SendText",
    "Friends",
    "WingJoin",
    "WingLeave",
    "WingAdd",
    "WingInvite",
    "JoinACrew",
    "QuitACrew",
    "EndCrewSession",
    "CrewAssign",
    "CrewHire",
    "CrewFire",
    "ChangeCrewRole",
    "CrewMemberJoins",
    "CrewMemberQuits",
    "CrewMemberRoleChange",
    "CrewLaunchFighter",
];

const POWERPLAY_EVENTS: &[&str] = &[
    "PowerplayJoin",
    "PowerplayLeave",
    "PowerplayDefect",
    "PowerplayCollect",
    "PowerplayDeliver",
    "PowerplayFastTrack",
    "PowerplayRank",
    "PowerplaySalary",
    "PowerplayVote",
    "PowerplayVoucher",
];

const SQUADRON_EVENTS: &[&str] = &[
    "InvitedToSquadron",
    "JoinedSquadron",
    "LeftSquadron",
    "KickedFromSquadron",
    "AppliedToSquadron",
    "CancelledSquadronApplication",
    "SquadronCreated",
    "SquadronDemotion",
    "SquadronPromotion",
    "SquadronStartup",
    "SharedBookmarkToSquadron",
    "SquadronApplicationApproved",
];

const CARRIER_EVENTS: &[&str] = &[
    "CarrierBankTransfer",
    "CarrierBuy",
    "CarrierCancelDecommission",
    "CarrierCrewServices",
    "CarrierDecommission",
    "CarrierDepositFuel",
    "CarrierDockingPermission",
    "CarrierFinance",
    "CarrierJump",
    "CarrierJumpCancelled",
    "CarrierJumpRequest",
    "CarrierLocation",
    "CarrierModulePack",
    "CarrierNameChange",
    "CarrierShipPack",
    "CarrierStats",
    "CarrierTradeOrder",
];

const COLONISATION_EVENTS: &[&str] = &[
    "ColonisationBeaconDeployed",
    "ColonisationConstructionDepot",
    "ColonisationContribution",
    "ColonisationSystemClaim",
    "ColonisationSystemClaimRelease",
];

pub fn parse_journal_line(line: &str) -> Result<JournalEvent, JournalParseError> {
    let value = serde_json::from_str::<Value>(line)
        .map_err(|source| JournalParseError::MalformedJson { source })?;
    parse_journal_value(&value)
}

pub fn parse_journal_value(value: &Value) -> Result<JournalEvent, JournalParseError> {
    let timestamp = parse_timestamp(value)?;
    let event = parse_event_name(value)?.to_string();

    match event.as_str() {
        "Commander" => commander_event(value, timestamp, event),
        "LoadGame" => load_game_event(value, timestamp, event),
        "Loadout" => loadout_event(value, timestamp, event),
        "Rank" => rank_event(value, timestamp, event),
        "Progress" => progress_event(value, timestamp, event),
        "Location" => location_event(value, timestamp, event),
        "SupercruiseDestinationDrop" => supercruise_destination_drop_event(value, timestamp, event),
        "SupercruiseEntry" => {
            travel_event(value, timestamp, event).map(JournalEvent::SupercruiseEntry)
        }
        "FSDJump" => travel_event(value, timestamp, event).map(JournalEvent::FSDJump),
        "ReceiveText" => receive_text_event(value, timestamp, event),
        "ShipTargeted" => ship_targeted_event(value, timestamp, event),
        "Bounty" => bounty_event(value, timestamp, event),
        "FactionKillBond" => faction_kill_bond_event(value, timestamp, event),
        "MissionRedirected" => {
            mission_event(value, timestamp, event).map(JournalEvent::MissionRedirected)
        }
        "Missions" => missions_event(value, timestamp, event),
        "MissionAccepted" => {
            mission_event(value, timestamp, event).map(JournalEvent::MissionAccepted)
        }
        "MissionCompleted" => {
            mission_event(value, timestamp, event).map(JournalEvent::MissionCompleted)
        }
        "MissionFailed" => mission_event(value, timestamp, event).map(JournalEvent::MissionFailed),
        "MissionAbandoned" => {
            mission_event(value, timestamp, event).map(JournalEvent::MissionAbandoned)
        }
        "ShieldState" => shield_state_event(value, timestamp, event),
        "HullDamage" => hull_damage_event(value, timestamp, event),
        "FighterDestroyed" => Ok(JournalEvent::FighterDestroyed(basic_event(
            timestamp, event,
        ))),
        "LaunchFighter" => launch_fighter_event(value, timestamp, event),
        "StartJump" => Ok(JournalEvent::StartJump(basic_event(timestamp, event))),
        "EjectCargo" => eject_cargo_event(value, timestamp, event),
        "ReservoirReplenished" => reservoir_replenished_event(value, timestamp, event),
        "PowerplayMerits" => powerplay_merits_event(value, timestamp, event),
        "Music" => music_event(value, timestamp, event),
        "ShipyardSwap" => shipyard_swap_event(value, timestamp, event),
        "Shutdown" => Ok(JournalEvent::Shutdown(basic_event(timestamp, event))),
        "Died" => Ok(JournalEvent::Died(basic_event(timestamp, event))),
        name if STARTUP_SNAPSHOT_EVENTS.contains(&name) => Ok(JournalEvent::StartupSnapshot(
            raw_event(value, timestamp, event),
        )),
        name if STATION_EVENTS.contains(&name) => {
            Ok(JournalEvent::Station(raw_event(value, timestamp, event)))
        }
        name if EXPLORATION_EVENTS.contains(&name) => Ok(JournalEvent::Exploration(raw_event(
            value, timestamp, event,
        ))),
        name if NAVIGATION_EVENTS.contains(&name) => {
            Ok(JournalEvent::Navigation(raw_event(value, timestamp, event)))
        }
        name if CARGO_MATERIAL_EVENTS.contains(&name) => Ok(JournalEvent::CargoMaterial(
            raw_event(value, timestamp, event),
        )),
        name if SHIP_MODULE_EVENTS.contains(&name) => {
            Ok(JournalEvent::ShipModule(raw_event(value, timestamp, event)))
        }
        name if MISSION_DETAIL_EVENTS.contains(&name) => Ok(JournalEvent::MissionDetail(
            raw_event(value, timestamp, event),
        )),
        name if COMBAT_DETAIL_EVENTS.contains(&name) => Ok(JournalEvent::CombatDetail(raw_event(
            value, timestamp, event,
        ))),
        name if ODYSSEY_EVENTS.contains(&name) => {
            Ok(JournalEvent::Odyssey(raw_event(value, timestamp, event)))
        }
        name if SOCIAL_EVENTS.contains(&name) => {
            Ok(JournalEvent::Social(raw_event(value, timestamp, event)))
        }
        name if POWERPLAY_EVENTS.contains(&name) => {
            Ok(JournalEvent::Powerplay(raw_event(value, timestamp, event)))
        }
        name if SQUADRON_EVENTS.contains(&name) => {
            Ok(JournalEvent::Squadron(raw_event(value, timestamp, event)))
        }
        name if CARRIER_EVENTS.contains(&name) => {
            Ok(JournalEvent::Carrier(raw_event(value, timestamp, event)))
        }
        name if COLONISATION_EVENTS.contains(&name) => Ok(JournalEvent::Colonisation(raw_event(
            value, timestamp, event,
        ))),
        _ => Ok(JournalEvent::Unknown {
            timestamp,
            event,
            raw: value.clone(),
        }),
    }
}

fn parse_timestamp(value: &Value) -> Result<DateTime<Utc>, JournalParseError> {
    let timestamp = value
        .get("timestamp")
        .and_then(Value::as_str)
        .ok_or(JournalParseError::MissingTimestamp)?;

    timestamp
        .parse::<DateTime<Utc>>()
        .map_err(|source| JournalParseError::InvalidTimestamp {
            value: timestamp.to_string(),
            source,
        })
}

fn parse_event_name(value: &Value) -> Result<&str, JournalParseError> {
    value
        .get("event")
        .and_then(Value::as_str)
        .ok_or(JournalParseError::MissingEvent)
}

fn basic_event(timestamp: DateTime<Utc>, event: String) -> BasicJournalEvent {
    BasicJournalEvent { timestamp, event }
}

fn raw_event(value: &Value, timestamp: DateTime<Utc>, event: String) -> RawJournalEvent {
    RawJournalEvent {
        timestamp,
        event,
        raw: value.clone(),
    }
}

fn event_fields<T: DeserializeOwned>(value: &Value, event: &str) -> Result<T, JournalParseError> {
    serde_json::from_value(value.clone()).map_err(|source| JournalParseError::InvalidEventFields {
        event: event.to_string(),
        source,
    })
}

fn commander_event(
    value: &Value,
    timestamp: DateTime<Utc>,
    event: String,
) -> Result<JournalEvent, JournalParseError> {
    let fields = event_fields::<CommanderFields>(value, &event)?;
    Ok(JournalEvent::Commander(CommanderEvent {
        timestamp,
        event,
        name: fields.name,
    }))
}

fn load_game_event(
    value: &Value,
    timestamp: DateTime<Utc>,
    event: String,
) -> Result<JournalEvent, JournalParseError> {
    let fields = event_fields::<LoadGameFields>(value, &event)?;
    Ok(JournalEvent::LoadGame(LoadGameEvent {
        timestamp,
        event,
        commander: fields.commander,
        ship: fields.ship,
        ship_localised: fields.ship_localised,
        game_mode: fields.game_mode,
    }))
}

fn loadout_event(
    value: &Value,
    timestamp: DateTime<Utc>,
    event: String,
) -> Result<JournalEvent, JournalParseError> {
    let fields = event_fields::<LoadoutFields>(value, &event)?;
    Ok(JournalEvent::Loadout(LoadoutEvent {
        timestamp,
        event,
        ship: fields.ship,
        ship_localised: fields.ship_localised,
        fuel_capacity_main: fields.fuel_capacity.and_then(|capacity| capacity.main),
    }))
}

fn rank_event(
    value: &Value,
    timestamp: DateTime<Utc>,
    event: String,
) -> Result<JournalEvent, JournalParseError> {
    let fields = event_fields::<RankFields>(value, &event)?;
    Ok(JournalEvent::Rank(RankEvent {
        timestamp,
        event,
        combat: fields.combat,
    }))
}

fn progress_event(
    value: &Value,
    timestamp: DateTime<Utc>,
    event: String,
) -> Result<JournalEvent, JournalParseError> {
    let fields = event_fields::<ProgressFields>(value, &event)?;
    Ok(JournalEvent::Progress(ProgressEvent {
        timestamp,
        event,
        combat: fields.combat,
    }))
}

fn location_event(
    value: &Value,
    timestamp: DateTime<Utc>,
    event: String,
) -> Result<JournalEvent, JournalParseError> {
    let fields = event_fields::<LocationFields>(value, &event)?;
    Ok(JournalEvent::Location(LocationEvent {
        timestamp,
        event,
        star_system: fields.star_system,
        body: fields.body,
        body_type: fields.body_type,
        docked: fields.docked,
    }))
}

fn supercruise_destination_drop_event(
    value: &Value,
    timestamp: DateTime<Utc>,
    event: String,
) -> Result<JournalEvent, JournalParseError> {
    let fields = event_fields::<SupercruiseDestinationDropFields>(value, &event)?;
    Ok(JournalEvent::SupercruiseDestinationDrop(
        SupercruiseDestinationDropEvent {
            timestamp,
            event,
            destination_type: fields.destination_type,
            destination_type_localised: fields.destination_type_localised,
        },
    ))
}

fn travel_event(
    value: &Value,
    timestamp: DateTime<Utc>,
    event: String,
) -> Result<TravelEvent, JournalParseError> {
    let fields = event_fields::<TravelFields>(value, &event)?;
    Ok(TravelEvent {
        timestamp,
        event,
        star_system: fields.star_system,
    })
}

fn receive_text_event(
    value: &Value,
    timestamp: DateTime<Utc>,
    event: String,
) -> Result<JournalEvent, JournalParseError> {
    let fields = event_fields::<ReceiveTextFields>(value, &event)?;
    Ok(JournalEvent::ReceiveText(ReceiveTextEvent {
        timestamp,
        event,
        from: fields.from,
        from_localised: fields.from_localised,
        message: fields.message,
        channel: fields.channel,
    }))
}

fn ship_targeted_event(
    value: &Value,
    timestamp: DateTime<Utc>,
    event: String,
) -> Result<JournalEvent, JournalParseError> {
    let fields = event_fields::<ShipTargetedFields>(value, &event)?;
    Ok(JournalEvent::ShipTargeted(ShipTargetedEvent {
        timestamp,
        event,
        target_locked: fields.target_locked,
        scan_stage: fields.scan_stage,
        ship: fields.ship,
        ship_localised: fields.ship_localised,
        pilot_name: fields.pilot_name,
        pilot_name_localised: fields.pilot_name_localised,
        pilot_rank: fields.pilot_rank,
        legal_status: fields.legal_status,
    }))
}

fn bounty_event(
    value: &Value,
    timestamp: DateTime<Utc>,
    event: String,
) -> Result<JournalEvent, JournalParseError> {
    let fields = event_fields::<BountyFields>(value, &event)?;
    Ok(JournalEvent::Bounty(BountyEvent {
        timestamp,
        event,
        total_reward: fields.total_reward,
        rewards: fields.rewards,
        victim_faction: fields.victim_faction,
        victim_faction_localised: fields.victim_faction_localised,
        target: fields.target,
        target_localised: fields.target_localised,
        pilot_name_localised: fields.pilot_name_localised,
    }))
}

fn faction_kill_bond_event(
    value: &Value,
    timestamp: DateTime<Utc>,
    event: String,
) -> Result<JournalEvent, JournalParseError> {
    let fields = event_fields::<FactionKillBondFields>(value, &event)?;
    Ok(JournalEvent::FactionKillBond(FactionKillBondEvent {
        timestamp,
        event,
        reward: fields.reward,
        awarding_faction: fields.awarding_faction,
        victim_faction: fields.victim_faction,
        victim_faction_localised: fields.victim_faction_localised,
    }))
}

fn mission_event(
    value: &Value,
    timestamp: DateTime<Utc>,
    event: String,
) -> Result<MissionEvent, JournalParseError> {
    let fields = event_fields::<MissionFields>(value, &event)?;
    Ok(MissionEvent {
        timestamp,
        event,
        mission_id: fields.mission_id,
        name: fields.name,
        localised_name: fields.localised_name,
    })
}

fn missions_event(
    value: &Value,
    timestamp: DateTime<Utc>,
    event: String,
) -> Result<JournalEvent, JournalParseError> {
    let fields = event_fields::<MissionsFields>(value, &event)?;
    Ok(JournalEvent::Missions(MissionsEvent {
        timestamp,
        event,
        active: fields
            .active
            .unwrap_or_default()
            .into_iter()
            .map(|item| MissionListItem {
                mission_id: item.mission_id,
                name: item.name,
                expires: item.expires,
            })
            .collect(),
    }))
}

fn shield_state_event(
    value: &Value,
    timestamp: DateTime<Utc>,
    event: String,
) -> Result<JournalEvent, JournalParseError> {
    let fields = event_fields::<ShieldStateFields>(value, &event)?;
    Ok(JournalEvent::ShieldState(ShieldStateEvent {
        timestamp,
        event,
        shields_up: fields.shields_up,
    }))
}

fn hull_damage_event(
    value: &Value,
    timestamp: DateTime<Utc>,
    event: String,
) -> Result<JournalEvent, JournalParseError> {
    let fields = event_fields::<HullDamageFields>(value, &event)?;
    Ok(JournalEvent::HullDamage(HullDamageEvent {
        timestamp,
        event,
        health: fields.health,
        player_pilot: fields.player_pilot,
        fighter: fields.fighter,
    }))
}

fn eject_cargo_event(
    value: &Value,
    timestamp: DateTime<Utc>,
    event: String,
) -> Result<JournalEvent, JournalParseError> {
    let fields = event_fields::<EjectCargoFields>(value, &event)?;
    Ok(JournalEvent::EjectCargo(EjectCargoEvent {
        timestamp,
        event,
        cargo_type: fields.cargo_type,
        cargo_type_localised: fields.cargo_type_localised,
        count: fields.count,
        abandoned: fields.abandoned,
    }))
}

fn reservoir_replenished_event(
    value: &Value,
    timestamp: DateTime<Utc>,
    event: String,
) -> Result<JournalEvent, JournalParseError> {
    let fields = event_fields::<ReservoirReplenishedFields>(value, &event)?;
    Ok(JournalEvent::ReservoirReplenished(
        ReservoirReplenishedEvent {
            timestamp,
            event,
            fuel_main: fields.fuel_main,
            fuel_reservoir: fields.fuel_reservoir,
        },
    ))
}

fn launch_fighter_event(
    value: &Value,
    timestamp: DateTime<Utc>,
    event: String,
) -> Result<JournalEvent, JournalParseError> {
    let fields = event_fields::<LaunchFighterFields>(value, &event)?;
    Ok(JournalEvent::LaunchFighter(LaunchFighterEvent {
        timestamp,
        event,
        player_controlled: fields.player_controlled,
    }))
}

fn powerplay_merits_event(
    value: &Value,
    timestamp: DateTime<Utc>,
    event: String,
) -> Result<JournalEvent, JournalParseError> {
    let fields = event_fields::<PowerplayMeritsFields>(value, &event)?;
    Ok(JournalEvent::PowerplayMerits(PowerplayMeritsEvent {
        timestamp,
        event,
        merits_gained: fields.merits_gained,
        power: fields.power,
    }))
}

fn shipyard_swap_event(
    value: &Value,
    timestamp: DateTime<Utc>,
    event: String,
) -> Result<JournalEvent, JournalParseError> {
    let fields = event_fields::<ShipyardSwapFields>(value, &event)?;
    Ok(JournalEvent::ShipyardSwap(ShipyardSwapEvent {
        timestamp,
        event,
        ship_type: fields.ship_type,
        ship_type_localised: fields.ship_type_localised,
    }))
}

fn music_event(
    value: &Value,
    timestamp: DateTime<Utc>,
    event: String,
) -> Result<JournalEvent, JournalParseError> {
    let fields = event_fields::<MusicFields>(value, &event)?;
    Ok(JournalEvent::Music(MusicEvent {
        timestamp,
        event,
        music_track: fields.music_track,
    }))
}

#[derive(Deserialize)]
struct CommanderFields {
    #[serde(rename = "Name")]
    name: Option<String>,
}

#[derive(Deserialize)]
struct LoadGameFields {
    #[serde(rename = "Commander")]
    commander: Option<String>,
    #[serde(rename = "Ship")]
    ship: Option<String>,
    #[serde(rename = "Ship_Localised")]
    ship_localised: Option<String>,
    #[serde(rename = "GameMode")]
    game_mode: Option<String>,
}

#[derive(Deserialize)]
struct LoadoutFields {
    #[serde(rename = "Ship")]
    ship: Option<String>,
    #[serde(rename = "Ship_Localised")]
    ship_localised: Option<String>,
    #[serde(rename = "FuelCapacity")]
    fuel_capacity: Option<FuelCapacityFields>,
}

#[derive(Deserialize)]
struct FuelCapacityFields {
    #[serde(rename = "Main")]
    main: Option<f64>,
}

#[derive(Deserialize)]
struct RankFields {
    #[serde(rename = "Combat")]
    combat: Option<u8>,
}

#[derive(Deserialize)]
struct ProgressFields {
    #[serde(rename = "Combat")]
    combat: Option<u8>,
}

#[derive(Deserialize)]
struct LocationFields {
    #[serde(rename = "StarSystem")]
    star_system: Option<String>,
    #[serde(rename = "Body")]
    body: Option<String>,
    #[serde(rename = "BodyType")]
    body_type: Option<String>,
    #[serde(rename = "Docked")]
    docked: Option<bool>,
}

#[derive(Deserialize)]
struct SupercruiseDestinationDropFields {
    #[serde(rename = "Type")]
    destination_type: Option<String>,
    #[serde(rename = "Type_Localised")]
    destination_type_localised: Option<String>,
}

#[derive(Deserialize)]
struct TravelFields {
    #[serde(rename = "StarSystem")]
    star_system: Option<String>,
}

#[derive(Deserialize)]
struct LaunchFighterFields {
    #[serde(rename = "PlayerControlled")]
    player_controlled: Option<bool>,
}

#[derive(Deserialize)]
struct ReceiveTextFields {
    #[serde(rename = "From")]
    from: Option<String>,
    #[serde(rename = "From_Localised")]
    from_localised: Option<String>,
    #[serde(rename = "Message")]
    message: Option<String>,
    #[serde(rename = "Channel")]
    channel: Option<String>,
}

#[derive(Deserialize)]
struct ShipTargetedFields {
    #[serde(rename = "TargetLocked")]
    target_locked: Option<bool>,
    #[serde(rename = "ScanStage")]
    scan_stage: Option<u8>,
    #[serde(rename = "Ship")]
    ship: Option<String>,
    #[serde(rename = "Ship_Localised")]
    ship_localised: Option<String>,
    #[serde(rename = "PilotName")]
    pilot_name: Option<String>,
    #[serde(rename = "PilotName_Localised")]
    pilot_name_localised: Option<String>,
    #[serde(rename = "PilotRank")]
    pilot_rank: Option<String>,
    #[serde(rename = "LegalStatus")]
    legal_status: Option<String>,
}

#[derive(Deserialize)]
struct BountyFields {
    #[serde(rename = "TotalReward")]
    total_reward: Option<u64>,
    #[serde(rename = "Rewards")]
    rewards: Option<Vec<BountyReward>>,
    #[serde(rename = "VictimFaction")]
    victim_faction: Option<String>,
    #[serde(rename = "VictimFaction_Localised")]
    victim_faction_localised: Option<String>,
    #[serde(rename = "Target")]
    target: Option<String>,
    #[serde(rename = "Target_Localised")]
    target_localised: Option<String>,
    #[serde(rename = "PilotName_Localised")]
    pilot_name_localised: Option<String>,
}

#[derive(Deserialize)]
struct FactionKillBondFields {
    #[serde(rename = "Reward")]
    reward: Option<u64>,
    #[serde(rename = "AwardingFaction")]
    awarding_faction: Option<String>,
    #[serde(rename = "VictimFaction")]
    victim_faction: Option<String>,
    #[serde(rename = "VictimFaction_Localised")]
    victim_faction_localised: Option<String>,
}

#[derive(Deserialize)]
struct MissionFields {
    #[serde(rename = "MissionID")]
    mission_id: Option<u64>,
    #[serde(rename = "Name")]
    name: Option<String>,
    #[serde(rename = "LocalisedName")]
    localised_name: Option<String>,
}

#[derive(Deserialize)]
struct MissionsFields {
    #[serde(rename = "Active")]
    active: Option<Vec<MissionItemFields>>,
}

#[derive(Deserialize)]
struct MissionItemFields {
    #[serde(rename = "MissionID")]
    mission_id: Option<u64>,
    #[serde(rename = "Name")]
    name: Option<String>,
    #[serde(rename = "Expires")]
    expires: Option<u64>,
}

#[derive(Deserialize)]
struct ShieldStateFields {
    #[serde(rename = "ShieldsUp")]
    shields_up: Option<bool>,
}

#[derive(Deserialize)]
struct HullDamageFields {
    #[serde(rename = "Health")]
    health: Option<f64>,
    #[serde(rename = "PlayerPilot")]
    player_pilot: Option<bool>,
    #[serde(rename = "Fighter")]
    fighter: Option<bool>,
}

#[derive(Deserialize)]
struct EjectCargoFields {
    #[serde(rename = "Type")]
    cargo_type: Option<String>,
    #[serde(rename = "Type_Localised")]
    cargo_type_localised: Option<String>,
    #[serde(rename = "Count")]
    count: Option<u64>,
    #[serde(rename = "Abandoned")]
    abandoned: Option<bool>,
}

#[derive(Deserialize)]
struct ShipyardSwapFields {
    #[serde(rename = "ShipType")]
    ship_type: Option<String>,
    #[serde(rename = "ShipType_Localised")]
    ship_type_localised: Option<String>,
}

#[derive(Deserialize)]
struct ReservoirReplenishedFields {
    #[serde(rename = "FuelMain")]
    fuel_main: Option<f64>,
    #[serde(rename = "FuelReservoir")]
    fuel_reservoir: Option<f64>,
}

#[derive(Deserialize)]
struct PowerplayMeritsFields {
    #[serde(rename = "MeritsGained")]
    merits_gained: Option<u64>,
    #[serde(rename = "Power")]
    power: Option<String>,
}

#[derive(Deserialize)]
struct MusicFields {
    #[serde(rename = "MusicTrack")]
    music_track: Option<String>,
}

#[cfg(test)]
mod event_parser {
    use super::*;
    use chrono::TimeZone;
    use serde_json::json;
    use std::fs;
    use std::path::Path;

    const PHASE_ONE_EVENTS: &[&str] = &[
        "Commander",
        "LoadGame",
        "Loadout",
        "Rank",
        "Progress",
        "Location",
        "SupercruiseDestinationDrop",
        "SupercruiseEntry",
        "FSDJump",
        "ReceiveText",
        "ShipTargeted",
        "Bounty",
        "FactionKillBond",
        "MissionRedirected",
        "Missions",
        "MissionAccepted",
        "MissionCompleted",
        "MissionFailed",
        "MissionAbandoned",
        "ShieldState",
        "HullDamage",
        "FighterDestroyed",
        "LaunchFighter",
        "StartJump",
        "EjectCargo",
        "ReservoirReplenished",
        "PowerplayMerits",
        "Music",
        "Shutdown",
        "Died",
    ];

    const FIXTURES: &[&str] = &[
        "journal_minimal_start.log",
        "journal_combat_bounty.log",
        "journal_missions.log",
        "journal_damage_fighter.log",
        "journal_malformed_unknown.log",
        "journal_warning_clock.log",
    ];

    fn fixture_dir() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
    }

    fn line_for_event(event: &str) -> String {
        let mut value = json!({
            "timestamp": "2035-02-03T04:05:06Z",
            "event": event,
        });

        match event {
            "ReceiveText" => {
                value["From"] = json!("npc_fixture_sender");
                value["From_Localised"] = json!("Fixture Sender");
                value["Message"] = json!("Synthetic parser coverage message.");
                value["Channel"] = json!("npc");
            }
            "ShipTargeted" => {
                value["TargetLocked"] = json!(true);
                value["ScanStage"] = json!(3);
                value["PilotName"] = json!("Fixture Pilot");
                value["LegalStatus"] = json!("Wanted");
            }
            "Bounty" => {
                value["TotalReward"] = json!(6400);
                value["Rewards"] = json!([{ "Faction": "Fixture Security", "Reward": 6400 }]);
                value["VictimFaction"] = json!("Fixture Raiders");
                value["Target"] = json!("viper");
            }
            "FactionKillBond" => {
                value["Reward"] = json!(12000);
                value["AwardingFaction"] = json!("Fixture Navy");
                value["VictimFaction"] = json!("Fixture Raiders");
            }
            "MissionRedirected" | "MissionAccepted" | "MissionCompleted" | "MissionFailed"
            | "MissionAbandoned" => {
                value["MissionID"] = json!(7001002);
                value["Name"] = json!("Mission_Delivery_name");
                value["LocalisedName"] = json!("Fixture Delivery Mission");
            }
            "ShieldState" => value["ShieldsUp"] = json!(true),
            "HullDamage" => {
                value["Health"] = json!(0.72);
                value["PlayerPilot"] = json!(true);
                value["Fighter"] = json!(false);
            }
            "EjectCargo" => {
                value["Type"] = json!("drones");
                value["Count"] = json!(1);
                value["Abandoned"] = json!(false);
            }
            "ReservoirReplenished" => {
                value["FuelMain"] = json!(16.0);
                value["FuelReservoir"] = json!(0.63);
            }
            "Music" => value["MusicTrack"] = json!("NoTrack"),
            _ => {}
        }

        value.to_string()
    }

    #[test]
    fn event_parser_parses_every_phase_one_event_name() {
        for event_name in PHASE_ONE_EVENTS {
            let parsed = parse_journal_line(&line_for_event(event_name)).unwrap();

            assert_eq!(parsed.event_name(), *event_name);
            assert_eq!(
                parsed.timestamp(),
                Utc.with_ymd_and_hms(2035, 2, 3, 4, 5, 6).single().unwrap()
            );
            assert!(
                !matches!(parsed, JournalEvent::Unknown { .. }),
                "{event_name} parsed as unknown"
            );
        }
    }

    #[test]
    fn event_parser_fixtures_parse_valid_lines_and_recover_malformed_line() {
        let mut parsed_valid_lines = 0;
        let mut malformed_lines = 0;
        let mut saw_unknown_event = false;

        for fixture in FIXTURES {
            let path = fixture_dir().join(fixture);
            let content = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));

            for line in content.lines() {
                match parse_journal_line(line) {
                    Ok(JournalEvent::Unknown {
                        event,
                        timestamp,
                        raw,
                    }) => {
                        assert_eq!(event, "FixtureUnknownEvent");
                        assert_eq!(
                            raw["Detail"],
                            json!("Valid JSON unknown event for parser tolerance.")
                        );
                        assert_eq!(
                            timestamp,
                            Utc.with_ymd_and_hms(2035, 1, 6, 16, 1, 0).single().unwrap()
                        );
                        saw_unknown_event = true;
                        parsed_valid_lines += 1;
                    }
                    Ok(event) => {
                        assert!(PHASE_ONE_EVENTS.contains(&event.event_name()));
                        parsed_valid_lines += 1;
                    }
                    Err(JournalParseError::MalformedJson { .. }) => malformed_lines += 1,
                    Err(error) => panic!("unexpected parser error for fixture line: {error}"),
                }
            }
        }

        assert!(parsed_valid_lines > PHASE_ONE_EVENTS.len());
        assert_eq!(malformed_lines, 1);
        assert!(saw_unknown_event);
    }

    #[test]
    fn event_parser_unknown_event_preserves_timestamp_and_event_name() {
        let parsed = parse_journal_line(
            r#"{"timestamp":"2035-01-06T16:01:00Z","event":"FixtureUnknownEvent","Detail":"ignored"}"#,
        )
        .unwrap();

        assert_eq!(parsed.event_name(), "FixtureUnknownEvent");
        assert_eq!(
            parsed.timestamp(),
            Utc.with_ymd_and_hms(2035, 1, 6, 16, 1, 0).single().unwrap()
        );
        assert!(matches!(parsed, JournalEvent::Unknown { .. }));
    }

    #[test]
    fn event_parser_categorizes_broad_journal_events_with_raw_payload() {
        assert_raw_event_category(STARTUP_SNAPSHOT_EVENTS, |event| {
            matches!(event, JournalEvent::StartupSnapshot(_))
        });
        assert_raw_event_category(STATION_EVENTS, |event| {
            matches!(event, JournalEvent::Station(_))
        });
        assert_raw_event_category(EXPLORATION_EVENTS, |event| {
            matches!(event, JournalEvent::Exploration(_))
        });
        assert_raw_event_category(NAVIGATION_EVENTS, |event| {
            matches!(event, JournalEvent::Navigation(_))
        });
        assert_raw_event_category(CARGO_MATERIAL_EVENTS, |event| {
            matches!(event, JournalEvent::CargoMaterial(_))
        });
        assert_raw_event_category(SHIP_MODULE_EVENTS, |event| {
            matches!(event, JournalEvent::ShipModule(_))
        });
        assert_raw_event_category(MISSION_DETAIL_EVENTS, |event| {
            matches!(event, JournalEvent::MissionDetail(_))
        });
        assert_raw_event_category(COMBAT_DETAIL_EVENTS, |event| {
            matches!(event, JournalEvent::CombatDetail(_))
        });
        assert_raw_event_category(ODYSSEY_EVENTS, |event| {
            matches!(event, JournalEvent::Odyssey(_))
        });
        assert_raw_event_category(SOCIAL_EVENTS, |event| {
            matches!(event, JournalEvent::Social(_))
        });
        assert_raw_event_category(POWERPLAY_EVENTS, |event| {
            matches!(event, JournalEvent::Powerplay(_))
        });
        assert_raw_event_category(SQUADRON_EVENTS, |event| {
            matches!(event, JournalEvent::Squadron(_))
        });
        assert_raw_event_category(CARRIER_EVENTS, |event| {
            matches!(event, JournalEvent::Carrier(_))
        });
        assert_raw_event_category(COLONISATION_EVENTS, |event| {
            matches!(event, JournalEvent::Colonisation(_))
        });
    }

    fn assert_raw_event_category(
        event_names: &[&str],
        matches_category: impl Fn(&JournalEvent) -> bool,
    ) {
        for event_name in event_names {
            let line = json!({
                "timestamp": "2035-02-03T04:05:06Z",
                "event": event_name,
                "FixtureField": { "Nested": 42 },
            })
            .to_string();
            let parsed = parse_journal_line(&line).unwrap();

            assert_eq!(parsed.event_name(), *event_name);
            assert_eq!(
                parsed.timestamp(),
                Utc.with_ymd_and_hms(2035, 2, 3, 4, 5, 6).single().unwrap()
            );
            assert!(
                matches_category(&parsed),
                "{event_name} parsed into {parsed:?}"
            );
            assert_eq!(
                parsed
                    .raw_payload()
                    .and_then(|raw| raw.pointer("/FixtureField/Nested")),
                Some(&json!(42))
            );
        }
    }

    #[test]
    fn event_parser_missing_required_event_returns_error() {
        let error = parse_journal_line(r#"{"timestamp":"2035-01-02T03:04:05Z"}"#)
            .expect_err("missing event should fail");

        assert!(matches!(error, JournalParseError::MissingEvent));
    }

    #[test]
    fn event_parser_missing_optional_fields_are_accepted() {
        for event_name in [
            "ReceiveText",
            "ShipTargeted",
            "Bounty",
            "FactionKillBond",
            "MissionRedirected",
            "MissionAccepted",
            "MissionCompleted",
            "MissionFailed",
            "MissionAbandoned",
            "ShieldState",
            "HullDamage",
            "EjectCargo",
            "ReservoirReplenished",
            "Music",
        ] {
            let parsed = parse_journal_line(
                &json!({"timestamp":"2035-02-03T04:05:06Z","event":event_name}).to_string(),
            )
            .unwrap();

            match parsed {
                JournalEvent::ReceiveText(event) => {
                    assert_eq!(event.from, None);
                    assert_eq!(event.message, None);
                    assert_eq!(event.channel, None);
                }
                JournalEvent::ShipTargeted(event) => {
                    assert_eq!(event.target_locked, None);
                    assert_eq!(event.scan_stage, None);
                    assert_eq!(event.pilot_name, None);
                    assert_eq!(event.legal_status, None);
                }
                JournalEvent::Bounty(event) => {
                    assert_eq!(event.total_reward, None);
                    assert_eq!(event.rewards, None);
                    assert_eq!(event.victim_faction, None);
                    assert_eq!(event.target, None);
                }
                JournalEvent::FactionKillBond(event) => {
                    assert_eq!(event.reward, None);
                    assert_eq!(event.awarding_faction, None);
                    assert_eq!(event.victim_faction, None);
                }
                JournalEvent::MissionRedirected(event)
                | JournalEvent::MissionAccepted(event)
                | JournalEvent::MissionCompleted(event)
                | JournalEvent::MissionFailed(event)
                | JournalEvent::MissionAbandoned(event) => {
                    assert_eq!(event.mission_id, None);
                    assert_eq!(event.name, None);
                    assert_eq!(event.localised_name, None);
                }
                JournalEvent::ShieldState(event) => assert_eq!(event.shields_up, None),
                JournalEvent::HullDamage(event) => {
                    assert_eq!(event.health, None);
                    assert_eq!(event.player_pilot, None);
                    assert_eq!(event.fighter, None);
                }
                JournalEvent::EjectCargo(event) => {
                    assert_eq!(event.cargo_type, None);
                    assert_eq!(event.count, None);
                    assert_eq!(event.abandoned, None);
                }
                JournalEvent::ReservoirReplenished(event) => {
                    assert_eq!(event.fuel_main, None);
                    assert_eq!(event.fuel_reservoir, None);
                }
                JournalEvent::Music(event) => assert_eq!(event.music_track, None),
                other => panic!("unexpected event for optional field test: {other:?}"),
            }
        }
    }

    #[test]
    fn event_parser_receive_text_and_ship_targeted_fields() {
        let receive_text = parse_journal_line(&line_for_event("ReceiveText")).unwrap();
        let ship_targeted = parse_journal_line(&line_for_event("ShipTargeted")).unwrap();

        match receive_text {
            JournalEvent::ReceiveText(event) => {
                assert_eq!(event.from.as_deref(), Some("npc_fixture_sender"));
                assert_eq!(event.from_localised.as_deref(), Some("Fixture Sender"));
                assert_eq!(
                    event.message.as_deref(),
                    Some("Synthetic parser coverage message.")
                );
                assert_eq!(event.channel.as_deref(), Some("npc"));
            }
            other => panic!("expected ReceiveText, got {other:?}"),
        }

        match ship_targeted {
            JournalEvent::ShipTargeted(event) => {
                assert_eq!(event.target_locked, Some(true));
                assert_eq!(event.scan_stage, Some(3));
                assert_eq!(event.pilot_name.as_deref(), Some("Fixture Pilot"));
                assert_eq!(event.legal_status.as_deref(), Some("Wanted"));
            }
            other => panic!("expected ShipTargeted, got {other:?}"),
        }
    }

    #[test]
    fn event_parser_reward_and_mission_fields() {
        let bounty = parse_journal_line(&line_for_event("Bounty")).unwrap();
        let kill_bond = parse_journal_line(&line_for_event("FactionKillBond")).unwrap();
        let mission = parse_journal_line(&line_for_event("MissionAccepted")).unwrap();

        match bounty {
            JournalEvent::Bounty(event) => {
                let rewards = event.rewards.unwrap();
                assert_eq!(event.total_reward, Some(6400));
                assert_eq!(event.victim_faction.as_deref(), Some("Fixture Raiders"));
                assert_eq!(event.target.as_deref(), Some("viper"));
                assert_eq!(rewards[0].faction.as_deref(), Some("Fixture Security"));
                assert_eq!(rewards[0].reward, Some(6400));
            }
            other => panic!("expected Bounty, got {other:?}"),
        }

        match kill_bond {
            JournalEvent::FactionKillBond(event) => {
                assert_eq!(event.reward, Some(12000));
                assert_eq!(event.awarding_faction.as_deref(), Some("Fixture Navy"));
                assert_eq!(event.victim_faction.as_deref(), Some("Fixture Raiders"));
            }
            other => panic!("expected FactionKillBond, got {other:?}"),
        }

        match mission {
            JournalEvent::MissionAccepted(event) => {
                assert_eq!(event.mission_id, Some(7001002));
                assert_eq!(event.name.as_deref(), Some("Mission_Delivery_name"));
                assert_eq!(
                    event.localised_name.as_deref(),
                    Some("Fixture Delivery Mission")
                );
            }
            other => panic!("expected MissionAccepted, got {other:?}"),
        }
    }

    #[test]
    fn event_parser_status_damage_cargo_fuel_and_music_fields() {
        let shield_state = parse_journal_line(&line_for_event("ShieldState")).unwrap();
        let hull_damage = parse_journal_line(&line_for_event("HullDamage")).unwrap();
        let eject_cargo = parse_journal_line(&line_for_event("EjectCargo")).unwrap();
        let fuel = parse_journal_line(&line_for_event("ReservoirReplenished")).unwrap();
        let music = parse_journal_line(&line_for_event("Music")).unwrap();

        match shield_state {
            JournalEvent::ShieldState(event) => assert_eq!(event.shields_up, Some(true)),
            other => panic!("expected ShieldState, got {other:?}"),
        }

        match hull_damage {
            JournalEvent::HullDamage(event) => {
                assert_eq!(event.health, Some(0.72));
                assert_eq!(event.player_pilot, Some(true));
                assert_eq!(event.fighter, Some(false));
            }
            other => panic!("expected HullDamage, got {other:?}"),
        }

        match eject_cargo {
            JournalEvent::EjectCargo(event) => {
                assert_eq!(event.cargo_type.as_deref(), Some("drones"));
                assert_eq!(event.count, Some(1));
                assert_eq!(event.abandoned, Some(false));
            }
            other => panic!("expected EjectCargo, got {other:?}"),
        }

        match fuel {
            JournalEvent::ReservoirReplenished(event) => {
                assert_eq!(event.fuel_main, Some(16.0));
                assert_eq!(event.fuel_reservoir, Some(0.63));
            }
            other => panic!("expected ReservoirReplenished, got {other:?}"),
        }

        match music {
            JournalEvent::Music(event) => assert_eq!(event.music_track.as_deref(), Some("NoTrack")),
            other => panic!("expected Music, got {other:?}"),
        }
    }
}

#[cfg(test)]
mod event_parser_malformed_json {
    use super::*;

    #[test]
    fn event_parser_malformed_json_returns_recoverable_error() {
        let error =
            parse_journal_line(r#"{"timestamp":"2035-01-06T16:04:00Z","event":"MalformedFixture""#)
                .expect_err("malformed JSON should fail without panic");

        assert!(matches!(error, JournalParseError::MalformedJson { .. }));
    }
}
