use std::fmt;

use toml::Value;

use super::{read_bool, read_optional_string, value_read, RuntimeConfig};

pub const MATRIX_DEVICE_ID: &str = "EDAFKDASHBOARD";

#[derive(Clone, PartialEq, Eq)]
pub struct MatrixConfig {
    pub enabled: bool,
    pub homeserver: Option<String>,
    pub user_id: Option<String>,
    pub room_id: Option<String>,
    pub access_token: Option<String>,
    pub mention_user_id: Option<String>,
    pub status_update_interval_seconds: u64,
}

#[derive(Clone, PartialEq, Eq)]
pub struct MatrixRuntimeConfig {
    pub homeserver: String,
    pub user_id: String,
    pub room_id: String,
    pub access_token: String,
    pub mention_user_id: Option<String>,
    pub status_update_interval_seconds: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MatrixRuntimeConfigResult {
    pub config: Option<MatrixRuntimeConfig>,
    pub warnings: Vec<String>,
}

impl Default for MatrixConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            homeserver: None,
            user_id: None,
            room_id: None,
            access_token: None,
            mention_user_id: None,
            status_update_interval_seconds: 60,
        }
    }
}

impl fmt::Debug for MatrixConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MatrixConfig")
            .field("enabled", &self.enabled)
            .field("homeserver", &self.homeserver)
            .field("user_id", &self.user_id)
            .field("room_id", &self.room_id)
            .field(
                "access_token",
                &self.access_token.as_ref().map(|_| "<redacted>"),
            )
            .field("mention_user_id", &self.mention_user_id)
            .field(
                "status_update_interval_seconds",
                &self.status_update_interval_seconds,
            )
            .finish()
    }
}

impl fmt::Debug for MatrixRuntimeConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MatrixRuntimeConfig")
            .field("homeserver", &self.homeserver)
            .field("user_id", &self.user_id)
            .field("room_id", &self.room_id)
            .field("access_token", &"<redacted>")
            .field("mention_user_id", &self.mention_user_id)
            .field(
                "status_update_interval_seconds",
                &self.status_update_interval_seconds,
            )
            .field("device_id", &MATRIX_DEVICE_ID)
            .finish()
    }
}

impl MatrixRuntimeConfig {
    pub fn device_id(&self) -> &'static str {
        MATRIX_DEVICE_ID
    }
}

impl MatrixConfig {
    pub fn to_runtime_config(&self) -> MatrixRuntimeConfigResult {
        if !self.enabled {
            return MatrixRuntimeConfigResult::default();
        }

        let homeserver = self.homeserver.clone();
        let user_id = self.user_id.clone();
        let room_id = self.room_id.clone();
        let access_token = self.access_token.clone();
        let missing_fields =
            Self::missing_runtime_fields(&homeserver, &user_id, &room_id, &access_token);
        if !missing_fields.is_empty() {
            return MatrixRuntimeConfigResult {
                config: None,
                warnings: vec![format!(
                    "Matrix delivery disabled for this run: missing required matrix config field(s): {}",
                    missing_fields.join(", ")
                )],
            };
        }

        let (Some(homeserver), Some(user_id), Some(room_id), Some(access_token)) =
            (homeserver, user_id, room_id, access_token)
        else {
            return MatrixRuntimeConfigResult::default();
        };

        MatrixRuntimeConfigResult {
            config: Some(MatrixRuntimeConfig {
                homeserver,
                user_id,
                room_id,
                access_token,
                mention_user_id: self.mention_user_id.clone(),
                status_update_interval_seconds: self.status_update_interval_seconds,
            }),
            warnings: Vec::new(),
        }
    }

    fn missing_runtime_fields(
        homeserver: &Option<String>,
        user_id: &Option<String>,
        room_id: &Option<String>,
        access_token: &Option<String>,
    ) -> Vec<&'static str> {
        let mut missing_fields = Vec::new();
        if homeserver.is_none() {
            missing_fields.push("homeserver");
        }
        if user_id.is_none() {
            missing_fields.push("user_id");
        }
        if room_id.is_none() {
            missing_fields.push("room_id");
        }
        if access_token.is_none() {
            missing_fields.push("access_token");
        }
        missing_fields
    }
}

pub fn matrix_runtime_config(matrix: &Option<MatrixConfig>) -> MatrixRuntimeConfigResult {
    match matrix {
        Some(matrix) => matrix.to_runtime_config(),
        None => MatrixRuntimeConfigResult::default(),
    }
}

impl RuntimeConfig {
    pub fn matrix_runtime(&self) -> MatrixRuntimeConfigResult {
        matrix_runtime_config(&self.matrix)
    }
}

pub(super) fn read_matrix_config(
    table: &toml::map::Map<String, Value>,
    warnings: &mut Vec<String>,
) -> Option<MatrixConfig> {
    let mut enabled = false;
    read_bool(
        table.get("enabled"),
        "matrix.enabled",
        &mut enabled,
        warnings,
    );
    if !enabled {
        return None;
    }

    let mut matrix = MatrixConfig::default();
    read_optional_string(
        table.get("homeserver"),
        "matrix.homeserver",
        &mut matrix.homeserver,
        warnings,
    );
    read_optional_string(
        table.get("user_id"),
        "matrix.user_id",
        &mut matrix.user_id,
        warnings,
    );
    read_optional_string(
        table.get("room_id"),
        "matrix.room_id",
        &mut matrix.room_id,
        warnings,
    );
    read_optional_string(
        table.get("access_token"),
        "matrix.access_token",
        &mut matrix.access_token,
        warnings,
    );
    read_optional_string(
        table.get("mention_user_id"),
        "matrix.mention_user_id",
        &mut matrix.mention_user_id,
        warnings,
    );
    value_read::read_u64(
        table.get("status_update_interval_seconds"),
        "matrix.status_update_interval_seconds",
        &mut matrix.status_update_interval_seconds,
        warnings,
    );

    Some(matrix)
}
