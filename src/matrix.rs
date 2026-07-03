use std::{future::Future, time::Duration};

use anyhow::{anyhow, Context};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use matrix_sdk::{
    authentication::matrix::MatrixSession,
    config::SyncSettings,
    reqwest::Url,
    room::edit::EditedContent,
    ruma::{
        events::{
            room::message::{RoomMessageEventContent, RoomMessageEventContentWithoutRelation},
            Mentions,
        },
        EventId, OwnedDeviceId, OwnedEventId, OwnedRoomAliasId, OwnedRoomId, OwnedUserId, UserId,
    },
    store::RoomLoadSettings,
    Client, Room, SessionMeta, SessionTokens,
};
use serde::Deserialize;
use tokio::time::timeout;

use crate::{
    config::MatrixRuntimeConfig, delivery::RemoteDelivery, notifier::Notification, text::line_safe,
};

const STARTUP_TIMEOUT: Duration = Duration::from_secs(10);
const OPERATION_TIMEOUT: Duration = Duration::from_secs(5);

#[async_trait]
trait MatrixRoomSender: Send {
    async fn send_message(
        &mut self,
        content: RoomMessageEventContent,
    ) -> anyhow::Result<OwnedEventId>;

    async fn edit_message(
        &mut self,
        original_event_id: &EventId,
        content: RoomMessageEventContentWithoutRelation,
    ) -> anyhow::Result<OwnedEventId>;

    async fn member_display_name(&self, user_id: &UserId) -> anyhow::Result<Option<String>>;
}

pub struct MatrixDelivery {
    sender: Box<dyn MatrixRoomSender>,
    mention_user_id: Option<OwnedUserId>,
    mention_display_name: Option<String>,
    status_update_interval: Duration,
    last_status_at: Option<DateTime<Utc>>,
    status_event_id: Option<OwnedEventId>,
    access_token_redactor: Option<String>,
}

struct MatrixSdkRoomSender {
    room: Room,
}

#[derive(Debug)]
enum MatrixRoomReference {
    Id(OwnedRoomId),
    Alias(OwnedRoomAliasId),
}

#[derive(Deserialize)]
struct MatrixWhoamiResponse {
    user_id: OwnedUserId,
    device_id: Option<OwnedDeviceId>,
}

impl MatrixDelivery {
    pub async fn connect(config: MatrixRuntimeConfig) -> anyhow::Result<Self> {
        Self::connect_with_startup_timeout(config, STARTUP_TIMEOUT).await
    }

    async fn connect_with_startup_timeout(
        config: MatrixRuntimeConfig,
        startup_timeout: Duration,
    ) -> anyhow::Result<Self> {
        let config_for_setup = config.clone();
        timeout_named(startup_timeout, "Matrix startup", async move {
            let sender = MatrixSdkRoomSender::connect(&config_for_setup).await?;
            Self::with_sender(Box::new(sender), &config_for_setup).await
        })
        .await
        .map_err(|error| redact_access_token(error, &config.access_token))
    }

    async fn with_sender(
        sender: Box<dyn MatrixRoomSender>,
        config: &MatrixRuntimeConfig,
    ) -> anyhow::Result<Self> {
        let mention_user_id = parse_optional_user_id(config.mention_user_id.as_deref())?;
        let mention_display_name = match mention_user_id.as_ref() {
            Some(user_id) => sender
                .member_display_name(user_id.as_ref())
                .await
                .unwrap_or(None),
            None => None,
        };

        Ok(Self {
            sender,
            mention_user_id,
            mention_display_name,
            status_update_interval: Duration::from_secs(config.status_update_interval_seconds),
            last_status_at: None,
            status_event_id: None,
            access_token_redactor: Some(config.access_token.clone()),
        })
    }

    async fn send_notification(&mut self, notification: &Notification) -> anyhow::Result<()> {
        let content = notification_content(
            notification,
            self.mention_user_id.as_ref(),
            self.mention_display_name.as_deref(),
        )?;
        timeout_named(OPERATION_TIMEOUT, "Matrix notification send", async {
            self.sender.send_message(content).await.map(|_| ())
        })
        .await
    }

    async fn send_status(
        &mut self,
        status: &str,
        now: DateTime<Utc>,
        force: bool,
    ) -> anyhow::Result<()> {
        if !self.should_publish_status(now, force) {
            return Ok(());
        }

        let body = line_safe(status);
        let status_event_id = self.status_event_id.clone();
        let result = timeout_named(OPERATION_TIMEOUT, "Matrix status publish", async {
            if let Some(event_id) = status_event_id.as_deref() {
                let content = RoomMessageEventContentWithoutRelation::text_plain(body);
                self.sender.edit_message(event_id, content).await
            } else {
                let content = RoomMessageEventContent::text_plain(body);
                self.sender.send_message(content).await
            }
        })
        .await?;

        if self.status_event_id.is_none() {
            self.status_event_id = Some(result);
        }
        self.last_status_at = Some(now);
        Ok(())
    }

    fn should_publish_status(&self, now: DateTime<Utc>, force: bool) -> bool {
        force
            || self
                .last_status_at
                .and_then(|last_status_at| (now - last_status_at).to_std().ok())
                .is_none_or(|elapsed| elapsed >= self.status_update_interval)
    }

    fn redact_error(&self, error: anyhow::Error) -> anyhow::Error {
        match self.access_token_redactor.as_deref() {
            Some(access_token) => redact_access_token(error, access_token),
            None => error,
        }
    }
}

#[async_trait]
impl RemoteDelivery for MatrixDelivery {
    async fn send(&mut self, notification: &Notification) -> anyhow::Result<()> {
        self.send_notification(notification)
            .await
            .map_err(|error| self.redact_error(error))
    }

    async fn publish_status(
        &mut self,
        status: &str,
        now: DateTime<Utc>,
        force: bool,
    ) -> anyhow::Result<()> {
        self.send_status(status, now, force)
            .await
            .map_err(|error| self.redact_error(error))
    }
}

impl MatrixSdkRoomSender {
    async fn connect(config: &MatrixRuntimeConfig) -> anyhow::Result<Self> {
        let room_reference = parse_room_reference(&config.room_id)?;
        let client = Client::builder()
            .homeserver_url(&config.homeserver)
            .build()
            .await
            .context("failed to build Matrix client")?;
        let session_meta = discover_session_meta(config).await?;

        client
            .matrix_auth()
            .restore_session(
                matrix_session(config, session_meta),
                RoomLoadSettings::default(),
            )
            .await
            .context("failed to restore Matrix access-token session")?;

        client
            .sync_once(SyncSettings::default())
            .await
            .context("failed to perform initial Matrix sync")?;

        let room_id = resolve_room_reference(&client, room_reference).await?;
        let room = client.get_room(&room_id).with_context(|| {
            format!("configured Matrix room {room_id} was not found after sync")
        })?;

        Ok(Self { room })
    }
}

#[async_trait]
impl MatrixRoomSender for MatrixSdkRoomSender {
    async fn send_message(
        &mut self,
        content: RoomMessageEventContent,
    ) -> anyhow::Result<OwnedEventId> {
        Ok(self.room.send(content).await?.response.event_id)
    }

    async fn edit_message(
        &mut self,
        original_event_id: &EventId,
        content: RoomMessageEventContentWithoutRelation,
    ) -> anyhow::Result<OwnedEventId> {
        let edit_event = self
            .room
            .make_edit_event(original_event_id, EditedContent::RoomMessage(content))
            .await
            .context("failed to build Matrix status edit event")?;
        Ok(self.room.send(edit_event).await?.response.event_id)
    }

    async fn member_display_name(&self, user_id: &UserId) -> anyhow::Result<Option<String>> {
        Ok(self
            .room
            .get_member(user_id)
            .await?
            .and_then(|member| member.display_name().map(str::to_owned)))
    }
}

async fn timeout_named<T, F>(duration: Duration, label: &str, future: F) -> anyhow::Result<T>
where
    F: Future<Output = anyhow::Result<T>>,
{
    timeout(duration, future)
        .await
        .map_err(|_| anyhow!("{label} timed out after {}s", duration.as_secs()))?
}

fn notification_content(
    notification: &Notification,
    mention_user_id: Option<&OwnedUserId>,
    mention_display_name: Option<&str>,
) -> anyhow::Result<RoomMessageEventContent> {
    let configured_mention_user_id = mention_user_id;
    let active_mention_user_id = notification
        .mention
        .then_some(configured_mention_user_id)
        .flatten();
    let mention_label =
        active_mention_user_id.map(|user_id| mention_label(user_id, mention_display_name));
    let mut parts = Vec::new();
    if let Some(label) = mention_label {
        parts.push(label.to_string());
    }
    if let Some(emoji) = notification.emoji.as_deref() {
        parts.push(emoji.to_string());
    }
    parts.push(notification.remote_text.clone());

    let body = matrix_message_safe(&parts.join(" "));
    let mut content = if notification.event_type == "matrix_startup" {
        let body = matrix_startup_body(
            notification,
            configured_mention_user_id,
            mention_display_name,
        );
        RoomMessageEventContent::text_html(
            body.clone(),
            matrix_startup_html(
                &body,
                startup_notification_target_html(configured_mention_user_id, mention_display_name),
            ),
        )
    } else if let (Some(user_id), Some(label)) = (active_mention_user_id, mention_label) {
        RoomMessageEventContent::text_html(
            body.clone(),
            matrix_mention_html(notification, user_id, label),
        )
    } else {
        RoomMessageEventContent::text_plain(body)
    };
    let content_mention_user_id = if notification.event_type == "matrix_startup" {
        configured_mention_user_id
    } else {
        active_mention_user_id
    };
    if let Some(user_id) = content_mention_user_id {
        content.mentions = Some(Mentions::with_user_ids([user_id.clone()]));
    }
    Ok(content)
}

fn mention_label<'a>(user_id: &'a UserId, display_name: Option<&'a str>) -> &'a str {
    display_name
        .filter(|name| !name.trim().is_empty())
        .unwrap_or_else(|| user_id.as_str())
}

fn matrix_startup_body(
    notification: &Notification,
    mention_user_id: Option<&OwnedUserId>,
    mention_display_name: Option<&str>,
) -> String {
    let notification_target = mention_user_id
        .map(|user_id| mention_label(user_id, mention_display_name))
        .unwrap_or("[not configured]");
    matrix_message_safe(&format!(
        "{}\nNotification target: {}",
        notification.remote_text, notification_target
    ))
}

fn matrix_message_safe(text: &str) -> String {
    text.chars()
        .map(|character| match character {
            '\r' => ' ',
            '\n' => '\n',
            character if character.is_control() => ' ',
            character => character,
        })
        .collect()
}

fn matrix_startup_html(body: &str, notification_target_html: Option<String>) -> String {
    let mut lines = body.lines();
    let title = lines.next().unwrap_or("ed-sentry started");
    let details = lines
        .filter_map(|line| line.split_once(':'))
        .map(|(label, value)| {
            let label = label.trim();
            let value_html = if label == "Notification target" {
                notification_target_html
                    .as_deref()
                    .map(ToOwned::to_owned)
                    .unwrap_or_else(|| html_escape(value.trim()))
            } else {
                html_escape(value.trim())
            };
            format!(
                "<li><strong>{}:</strong> {}</li>",
                html_escape(label),
                value_html
            )
        })
        .collect::<String>();

    format!(
        "<h2><strong>{}</strong></h2><ul>{details}</ul>",
        html_escape(title)
    )
}

fn startup_notification_target_html(
    mention_user_id: Option<&OwnedUserId>,
    mention_display_name: Option<&str>,
) -> Option<String> {
    mention_user_id
        .map(|user_id| matrix_mention_anchor(user_id, mention_label(user_id, mention_display_name)))
}

fn matrix_mention_html(
    notification: &Notification,
    user_id: &UserId,
    mention_label: &str,
) -> String {
    let mut parts = vec![matrix_mention_anchor(user_id, mention_label)];
    if let Some(emoji) = notification.emoji.as_deref() {
        parts.push(html_text(&matrix_message_safe(emoji)));
    }
    parts.push(html_text(&matrix_message_safe(&notification.remote_text)));
    parts.join(" ")
}

fn matrix_mention_anchor(user_id: &UserId, mention_label: &str) -> String {
    format!(
        "<a href=\"https://matrix.to/#/{}\">{}</a>",
        html_escape(user_id.as_str()),
        html_text(&matrix_message_safe(mention_label))
    )
}

fn html_text(text: &str) -> String {
    let mut escaped = String::new();
    for character in text.chars() {
        match character {
            '\n' => escaped.push_str("<br>"),
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            character => escaped.push(character),
        }
    }
    escaped
}

fn html_escape(text: &str) -> String {
    let mut escaped = String::new();
    for character in text.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            character => escaped.push(character),
        }
    }
    escaped
}

async fn discover_session_meta(config: &MatrixRuntimeConfig) -> anyhow::Result<SessionMeta> {
    let homeserver = Url::parse(&config.homeserver).context("invalid Matrix homeserver URL")?;
    let whoami_url = homeserver
        .join("/_matrix/client/v3/account/whoami")
        .context("failed to build Matrix account identity URL")?;
    let whoami_response = matrix_sdk::reqwest::Client::new()
        .get(whoami_url)
        .bearer_auth(&config.access_token)
        .send()
        .await
        .context("failed to query Matrix account identity")?
        .error_for_status()
        .context("Matrix account identity request failed")?;
    let whoami_bytes = whoami_response
        .bytes()
        .await
        .context("failed to read Matrix account identity response")?;
    let whoami: MatrixWhoamiResponse = serde_json::from_slice(&whoami_bytes)
        .context("failed to parse Matrix account identity response")?;

    Ok(SessionMeta {
        user_id: whoami.user_id,
        device_id: whoami
            .device_id
            .unwrap_or_else(|| OwnedDeviceId::from(config.device_id())),
    })
}

fn matrix_session(config: &MatrixRuntimeConfig, meta: SessionMeta) -> MatrixSession {
    MatrixSession {
        meta,
        tokens: SessionTokens {
            access_token: config.access_token.clone(),
            refresh_token: None,
        },
    }
}

fn parse_user_id(user_id: &str) -> anyhow::Result<OwnedUserId> {
    OwnedUserId::try_from(user_id).with_context(|| format!("invalid Matrix user ID {user_id}"))
}

fn parse_optional_user_id(user_id: Option<&str>) -> anyhow::Result<Option<OwnedUserId>> {
    user_id.map(parse_user_id).transpose()
}

async fn resolve_room_reference(
    client: &Client,
    room_reference: MatrixRoomReference,
) -> anyhow::Result<OwnedRoomId> {
    match room_reference {
        MatrixRoomReference::Id(room_id) => Ok(room_id),
        MatrixRoomReference::Alias(alias) => client
            .resolve_room_alias(&alias)
            .await
            .with_context(|| format!("failed to resolve Matrix room alias {alias}"))
            .map(|response| response.room_id),
    }
}

fn parse_room_reference(room: &str) -> anyhow::Result<MatrixRoomReference> {
    if let Ok(room_id) = OwnedRoomId::try_from(room) {
        return Ok(MatrixRoomReference::Id(room_id));
    }
    if let Ok(alias) = OwnedRoomAliasId::try_from(room) {
        return Ok(MatrixRoomReference::Alias(alias));
    }

    anyhow::bail!("invalid Matrix room ID or alias {room}")
}

fn redact_access_token(error: anyhow::Error, access_token: &str) -> anyhow::Error {
    let message = error.to_string();
    if access_token.is_empty() {
        return anyhow!(message);
    }
    anyhow!(message.replace(access_token, "<redacted>"))
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{Arc, Mutex},
        time::Duration,
    };

    use super::*;
    use crate::notifier::AlertLevel;
    use matrix_sdk::ruma::{
        events::room::message::{MessageType, Relation},
        owned_event_id,
    };

    struct FakeMatrixRoomSender {
        state: Arc<Mutex<FakeMatrixRoomSenderState>>,
        fail_with: Option<String>,
        delay: Option<Duration>,
        member_display_name: Option<String>,
    }

    #[derive(Default)]
    struct FakeMatrixRoomSenderState {
        next_event_number: usize,
        member_display_name_requests: usize,
        sent: Vec<RoomMessageEventContent>,
        edits: Vec<(OwnedEventId, RoomMessageEventContentWithoutRelation)>,
    }

    impl FakeMatrixRoomSender {
        fn new() -> (Self, Arc<Mutex<FakeMatrixRoomSenderState>>) {
            let state = Arc::new(Mutex::new(FakeMatrixRoomSenderState {
                next_event_number: 1,
                ..FakeMatrixRoomSenderState::default()
            }));
            (
                Self {
                    state: state.clone(),
                    fail_with: None,
                    delay: None,
                    member_display_name: None,
                },
                state,
            )
        }

        fn with_member_display_name(
            display_name: &str,
        ) -> (Self, Arc<Mutex<FakeMatrixRoomSenderState>>) {
            let (sender, state) = Self::new();
            (
                Self {
                    member_display_name: Some(display_name.to_string()),
                    ..sender
                },
                state,
            )
        }

        fn failing(message: &str) -> (Self, Arc<Mutex<FakeMatrixRoomSenderState>>) {
            let (sender, state) = Self::new();
            (
                Self {
                    fail_with: Some(message.to_string()),
                    ..sender
                },
                state,
            )
        }
    }

    impl FakeMatrixRoomSenderState {
        fn event_id(&mut self) -> OwnedEventId {
            let event_id =
                OwnedEventId::try_from(format!("$fake-{}:matrix.invalid", self.next_event_number))
                    .unwrap();
            self.next_event_number += 1;
            event_id
        }
    }

    impl Default for FakeMatrixRoomSender {
        fn default() -> Self {
            Self {
                state: Arc::new(Mutex::new(FakeMatrixRoomSenderState {
                    next_event_number: 1,
                    ..FakeMatrixRoomSenderState::default()
                })),
                fail_with: None,
                delay: None,
                member_display_name: None,
            }
        }
    }

    #[async_trait]
    impl MatrixRoomSender for FakeMatrixRoomSender {
        async fn send_message(
            &mut self,
            content: RoomMessageEventContent,
        ) -> anyhow::Result<OwnedEventId> {
            if let Some(delay) = self.delay {
                tokio::time::sleep(delay).await;
            }
            if let Some(message) = &self.fail_with {
                anyhow::bail!(message.clone());
            }
            let mut state = self.state.lock().unwrap();
            state.sent.push(content);
            Ok(state.event_id())
        }

        async fn edit_message(
            &mut self,
            original_event_id: &EventId,
            content: RoomMessageEventContentWithoutRelation,
        ) -> anyhow::Result<OwnedEventId> {
            if let Some(delay) = self.delay {
                tokio::time::sleep(delay).await;
            }
            if let Some(message) = &self.fail_with {
                anyhow::bail!(message.clone());
            }
            let mut state = self.state.lock().unwrap();
            state.edits.push((original_event_id.to_owned(), content));
            Ok(state.event_id())
        }

        async fn member_display_name(&self, _user_id: &UserId) -> anyhow::Result<Option<String>> {
            let mut state = self.state.lock().unwrap();
            state.member_display_name_requests += 1;
            Ok(self.member_display_name.clone())
        }
    }

    fn config(mention_user_id: Option<&str>, access_token: &str) -> MatrixRuntimeConfig {
        MatrixRuntimeConfig {
            homeserver: "https://matrix.invalid".to_string(),
            room_id: "!room:matrix.invalid".to_string(),
            access_token: access_token.to_string(),
            mention_user_id: mention_user_id.map(ToOwned::to_owned),
            status_update_interval_seconds: 60,
        }
    }

    fn notification(level: u8, emoji: Option<&str>, remote_text: &str) -> Notification {
        Notification::new(
            "matrix_fixture",
            level,
            AlertLevel::Warn,
            emoji.map(ToOwned::to_owned),
            "terminal text should not be sent",
            remote_text,
            DateTime::parse_from_rfc3339("2035-06-09T16:30:00Z")
                .unwrap()
                .with_timezone(&Utc),
        )
    }

    fn text_body(content: &RoomMessageEventContent) -> &str {
        match &content.msgtype {
            MessageType::Text(text) => &text.body,
            other => panic!("unexpected Matrix message type: {other:?}"),
        }
    }

    fn edit_text_body(content: &RoomMessageEventContentWithoutRelation) -> &str {
        match &content.msgtype {
            MessageType::Text(text) => &text.body,
            other => panic!("unexpected Matrix edit message type: {other:?}"),
        }
    }

    fn formatted_text_body(content: &RoomMessageEventContent) -> Option<&str> {
        match &content.msgtype {
            MessageType::Text(text) => text
                .formatted
                .as_ref()
                .map(|formatted| formatted.body.as_str()),
            other => panic!("unexpected Matrix message type: {other:?}"),
        }
    }

    #[test]
    fn matrix_formats_level_one_without_mention() {
        let content = notification_content(
            &notification(1, Some("!"), "Remote line\nnot raw"),
            Some(&parse_user_id("@commander:matrix.invalid").unwrap()),
            None,
        )
        .unwrap();

        assert_eq!(text_body(&content), "! Remote line\nnot raw");
        assert!(content.mentions.is_none());
    }

    #[test]
    fn matrix_formats_level_two_with_mentions_metadata() {
        let mentioned_user_id = parse_user_id("@commander:matrix.invalid").unwrap();
        let content = notification_content(
            &notification(2, Some("!"), "Ship hull critical"),
            Some(&mentioned_user_id),
            Some("Commander"),
        )
        .unwrap();

        assert_eq!(text_body(&content), "Commander ! Ship hull critical");
        assert_eq!(
            formatted_text_body(&content),
            Some(
                "<a href=\"https://matrix.to/#/@commander:matrix.invalid\">Commander</a> ! Ship hull critical"
            )
        );
        let mentions = content.mentions.unwrap();
        assert!(!mentions.room);
        assert_eq!(
            mentions.user_ids.into_iter().collect::<Vec<_>>(),
            vec![mentioned_user_id]
        );
    }

    #[test]
    fn matrix_formats_startup_header_as_html() {
        let mentioned_user_id = parse_user_id("@controlnet:controlnet.space").unwrap();
        let content = notification_content(
            &Notification::new(
                "matrix_startup",
                1,
                AlertLevel::Info,
                None,
                "terminal text should not be sent",
                "🛰️ ed-sentry started\nVersion: 0.1.0\nStarted at: 2035-06-09T16:30:00Z\nJournal folder: D:\\Saved Games\\Elite & Dangerous\nJournal file: Journal.test.log\nMatrix room: #ed-sentry:example.org",
                DateTime::parse_from_rfc3339("2035-06-09T16:30:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
            Some(&mentioned_user_id),
            Some("ControlNet"),
        )
        .unwrap();

        assert!(text_body(&content).starts_with("🛰️ ed-sentry started\nVersion: 0.1.0"));
        assert!(text_body(&content).contains("\nNotification target: ControlNet"));
        let formatted = formatted_text_body(&content).unwrap();
        assert!(formatted.starts_with("<h2><strong>🛰️ ed-sentry started</strong></h2>"));
        assert!(formatted.contains("<li><strong>Version:</strong> 0.1.0</li>"));
        assert!(formatted.contains("Elite &amp; Dangerous"));
        assert!(formatted.contains("<li><strong>Matrix room:</strong> #ed-sentry:example.org</li>"));
        assert!(formatted.contains(
            "<li><strong>Notification target:</strong> <a href=\"https://matrix.to/#/@controlnet:controlnet.space\">ControlNet</a></li>"
        ));
        let mentions = content.mentions.unwrap();
        assert!(!mentions.room);
        assert_eq!(
            mentions.user_ids.into_iter().collect::<Vec<_>>(),
            vec![mentioned_user_id]
        );
    }

    #[tokio::test]
    async fn matrix_status_edits_original_event_id() {
        let (sender, state) = FakeMatrixRoomSender::new();
        let mut delivery =
            MatrixDelivery::with_sender(Box::new(sender), &config(None, "fixture-access"))
                .await
                .unwrap();
        let now = DateTime::parse_from_rfc3339("2035-06-09T16:30:00Z")
            .unwrap()
            .with_timezone(&Utc);

        delivery.publish_status("Kills 1", now, true).await.unwrap();
        let original_event_id = delivery.status_event_id.clone().unwrap();
        delivery
            .publish_status(
                "Kills 2\nBounties 3",
                now + chrono::Duration::seconds(5),
                true,
            )
            .await
            .unwrap();

        let fake = state.lock().unwrap();
        assert_eq!(fake.sent.len(), 1);
        assert_eq!(text_body(&fake.sent[0]), "Kills 1");
        assert_eq!(fake.edits.len(), 1);
        assert_eq!(fake.edits[0].0, original_event_id);
        assert_eq!(edit_text_body(&fake.edits[0].1), "Kills 2 Bounties 3");
        assert_eq!(delivery.status_event_id.as_ref(), Some(&fake.edits[0].0));
    }

    #[tokio::test]
    async fn matrix_errors_redact_access_token() {
        let token = concat!("fixture-", "secret-value");
        let (sender, _) = FakeMatrixRoomSender::failing(&format!("server rejected {token}"));
        let mut delivery = MatrixDelivery::with_sender(Box::new(sender), &config(None, token))
            .await
            .unwrap();

        let error = delivery
            .send(&notification(1, None, "Fuel stable"))
            .await
            .unwrap_err();
        let message = error.to_string();

        assert!(message.contains("<redacted>"), "{message}");
        assert!(!message.contains(token), "{message}");
    }

    #[test]
    fn matrix_missing_mention_config_sends_normal_text() {
        let content =
            notification_content(&notification(2, None, "No mention config"), None, None).unwrap();

        assert_eq!(text_body(&content), "No mention config");
        assert!(content.mentions.is_none());
    }

    #[tokio::test]
    async fn matrix_caches_room_member_display_name_for_mentions() {
        let (sender, state) = FakeMatrixRoomSender::with_member_display_name("ControlNet");
        let mut delivery = MatrixDelivery::with_sender(
            Box::new(sender),
            &config(Some("@controlnet:controlnet.space"), "fixture-access"),
        )
        .await
        .unwrap();

        delivery
            .send(&notification(2, Some("🏅"), "Rank promotion: Federation 8"))
            .await
            .unwrap();
        delivery
            .send(&notification(2, Some("!"), "Second alert"))
            .await
            .unwrap();

        let fake = state.lock().unwrap();
        assert_eq!(fake.member_display_name_requests, 1);
        assert_eq!(fake.sent.len(), 2);
        assert_eq!(
            text_body(&fake.sent[0]),
            "ControlNet 🏅 Rank promotion: Federation 8"
        );
        assert_eq!(
            formatted_text_body(&fake.sent[0]),
            Some(
                "<a href=\"https://matrix.to/#/@controlnet:controlnet.space\">ControlNet</a> 🏅 Rank promotion: Federation 8"
            )
        );
    }

    #[test]
    fn matrix_room_reference_accepts_alias_with_colon_server_name() {
        let reference = parse_room_reference("#alerts:matrix.invalid").unwrap();

        match reference {
            MatrixRoomReference::Alias(alias) => {
                assert_eq!(alias.as_str(), "#alerts:matrix.invalid")
            }
            MatrixRoomReference::Id(room_id) => panic!("unexpected room id: {room_id}"),
        }
    }

    #[test]
    fn matrix_room_reference_rejects_alias_with_at_server_separator() {
        let error = parse_room_reference("#alerts@matrix.invalid").unwrap_err();

        assert!(
            error
                .to_string()
                .contains("invalid Matrix room ID or alias"),
            "{error}"
        );
    }

    #[test]
    fn matrix_status_edit_content_targets_original_event_id() {
        let original = owned_event_id!("$original:matrix.invalid");
        let content = RoomMessageEventContentWithoutRelation::text_plain("Status update")
            .make_replacement(
                matrix_sdk::ruma::events::room::message::ReplacementMetadata::new(
                    original.clone(),
                    None,
                ),
            );

        assert_eq!(text_body(&content), "* Status update");
        match content.relates_to.unwrap() {
            Relation::Replacement(replacement) => assert_eq!(replacement.event_id, original),
            other => panic!("unexpected relation: {other:?}"),
        }
    }
}
