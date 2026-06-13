use std::cmp::Ordering;
use std::fmt;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use chrono::{DateTime, NaiveDateTime, Utc};

use crate::config::{JournalConfig, RuntimeConfig};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JournalFile {
    pub path: PathBuf,
    pub filename_timestamp: Option<DateTime<Utc>>,
    pub modified_at: SystemTime,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JournalFileChoice {
    pub number: usize,
    pub file: JournalFile,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PreloadOptions {
    pub reset_session_after_preload: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreloadResult<T> {
    pub records: Vec<PreloadRecord<T>>,
    pub eof_offset: u64,
    pub reset_session_after_preload: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StreamJournalResult {
    pub eof_offset: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreloadRecord<T> {
    pub line_number: usize,
    pub start_offset: u64,
    pub result: Result<T, JournalLineError>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveTailRecord<T> {
    pub start_offset: u64,
    pub result: Result<T, JournalLineError>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveTailPoll<T> {
    pub records: Vec<LiveTailRecord<T>>,
    pub warnings: Vec<LiveTailWarning>,
    pub offset: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LiveTailWarning {
    FileTruncated {
        previous_offset: u64,
        new_offset: u64,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveTail {
    path: PathBuf,
    offset: u64,
    buffered_start_offset: u64,
    buffer: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JournalLineError {
    pub message: String,
}

#[derive(Debug)]
pub enum JournalError {
    DefaultPathUnavailable,
    DirectoryRead { path: PathBuf, source: io::Error },
    MetadataRead { path: PathBuf, source: io::Error },
    NoJournalFiles { folder: PathBuf },
    FileOpen { path: PathBuf, source: io::Error },
    FileRead { path: PathBuf, source: io::Error },
    Callback { message: String },
}

impl fmt::Display for JournalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DefaultPathUnavailable => write!(
                formatter,
                "journal folder not configured and the Windows Saved Games known folder is unavailable"
            ),
            Self::DirectoryRead { path, source } => write!(
                formatter,
                "failed to read journal directory {}: {source}",
                path.display()
            ),
            Self::MetadataRead { path, source } => write!(
                formatter,
                "failed to read journal metadata {}: {source}",
                path.display()
            ),
            Self::NoJournalFiles { folder } => write!(
                formatter,
                "no Journal.*.log files found in {}",
                folder.display()
            ),
            Self::FileOpen { path, source } => write!(
                formatter,
                "failed to open journal file {}: {source}",
                path.display()
            ),
            Self::FileRead { path, source } => write!(
                formatter,
                "failed to read journal file {}: {source}",
                path.display()
            ),
            Self::Callback { message } => write!(formatter, "journal stream callback failed: {message}"),
        }
    }
}

impl std::error::Error for JournalError {}

impl fmt::Display for LiveTailWarning {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FileTruncated {
                previous_offset,
                new_offset,
            } => write!(
                formatter,
                "journal file truncated from byte offset {previous_offset} to {new_offset}; live tail reset to current EOF"
            ),
        }
    }
}

impl JournalError {
    pub const fn exit_code(&self) -> u8 {
        1
    }
}

impl JournalLineError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

pub fn journal_folder_from_saved_games(saved_games_folder: impl AsRef<Path>) -> PathBuf {
    saved_games_folder
        .as_ref()
        .join("Frontier Developments")
        .join("Elite Dangerous")
}

pub fn default_journal_folder() -> Result<PathBuf, JournalError> {
    default_saved_games_folder().map(journal_folder_from_saved_games)
}

#[cfg(windows)]
fn default_saved_games_folder() -> Result<PathBuf, JournalError> {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use std::ptr::null_mut;
    use std::slice;

    use windows_sys::Win32::System::Com::CoTaskMemFree;
    use windows_sys::Win32::UI::Shell::{
        FOLDERID_SavedGames, SHGetKnownFolderPath, KF_FLAG_DEFAULT,
    };

    let mut raw_path = null_mut();
    let result = unsafe {
        SHGetKnownFolderPath(
            &FOLDERID_SavedGames,
            KF_FLAG_DEFAULT as u32,
            null_mut(),
            &mut raw_path,
        )
    };

    if result < 0 || raw_path.is_null() {
        return Err(JournalError::DefaultPathUnavailable);
    }

    let mut len = 0;
    unsafe {
        while *raw_path.add(len) != 0 {
            len += 1;
        }
    }

    let path = unsafe {
        let wide = slice::from_raw_parts(raw_path, len);
        PathBuf::from(OsString::from_wide(wide))
    };
    unsafe {
        CoTaskMemFree(raw_path.cast());
    }

    Ok(path)
}

#[cfg(not(windows))]
fn default_saved_games_folder() -> Result<PathBuf, JournalError> {
    Err(JournalError::DefaultPathUnavailable)
}

pub fn resolve_journal_folder(config: &JournalConfig) -> Result<PathBuf, JournalError> {
    if config.folder.is_empty() {
        default_journal_folder()
    } else {
        Ok(PathBuf::from(&config.folder))
    }
}

pub fn discover_configured_journal_files(
    config: &JournalConfig,
) -> Result<Vec<JournalFile>, JournalError> {
    discover_journal_files(resolve_journal_folder(config)?)
}

pub fn discover_runtime_journal_files(
    config: &RuntimeConfig,
) -> Result<Vec<JournalFile>, JournalError> {
    discover_configured_journal_files(&config.journal)
}

pub fn discover_journal_files(folder: impl AsRef<Path>) -> Result<Vec<JournalFile>, JournalError> {
    let folder = folder.as_ref();
    let entries = fs::read_dir(folder).map_err(|source| JournalError::DirectoryRead {
        path: folder.to_path_buf(),
        source,
    })?;

    let mut files = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| JournalError::DirectoryRead {
            path: folder.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if !is_journal_log_path(&path) {
            continue;
        }

        let metadata = entry
            .metadata()
            .map_err(|source| JournalError::MetadataRead {
                path: path.clone(),
                source,
            })?;
        if !metadata.is_file() {
            continue;
        }

        files.push(JournalFile {
            filename_timestamp: parse_journal_filename_timestamp(&path),
            modified_at: metadata
                .modified()
                .map_err(|source| JournalError::MetadataRead {
                    path: path.clone(),
                    source,
                })?,
            path,
        });
    }

    if files.is_empty() {
        return Err(JournalError::NoJournalFiles {
            folder: folder.to_path_buf(),
        });
    }

    sort_journal_files_newest_first(&mut files);
    Ok(files)
}

pub fn select_newest_journal_file(folder: impl AsRef<Path>) -> Result<JournalFile, JournalError> {
    discover_journal_files(folder).map(|mut files| files.remove(0))
}

pub fn select_configured_journal_file(config: &RuntimeConfig) -> Result<PathBuf, JournalError> {
    if let Some(set_file) = &config.set_file {
        Ok(set_file.clone())
    } else {
        Ok(discover_runtime_journal_files(config)?.remove(0).path)
    }
}

pub fn recent_journal_file_choices(
    folder: impl AsRef<Path>,
    recent_files: u16,
) -> Result<Vec<JournalFileChoice>, JournalError> {
    Ok(discover_journal_files(folder)?
        .into_iter()
        .take(usize::from(recent_files))
        .enumerate()
        .map(|(index, file)| JournalFileChoice {
            number: index + 1,
            file,
        })
        .collect())
}

pub fn configured_recent_journal_file_choices(
    config: &RuntimeConfig,
) -> Result<Vec<JournalFileChoice>, JournalError> {
    recent_journal_file_choices(
        resolve_journal_folder(&config.journal)?,
        config.journal.recent_files,
    )
}

pub fn preload_journal_file<T, E, F>(
    path: impl AsRef<Path>,
    parser: F,
) -> Result<PreloadResult<T>, JournalError>
where
    F: FnMut(&str) -> Result<T, E>,
    E: fmt::Display,
{
    preload_journal_file_with_options(path, PreloadOptions::default(), parser)
}

pub fn preload_journal_file_with_options<T, E, F>(
    path: impl AsRef<Path>,
    options: PreloadOptions,
    parser: F,
) -> Result<PreloadResult<T>, JournalError>
where
    F: FnMut(&str) -> Result<T, E>,
    E: fmt::Display,
{
    let mut records = Vec::new();
    let stream = read_journal_records(path, parser, |record| {
        records.push(PreloadRecord {
            line_number: record.line_number,
            start_offset: record.start_offset,
            result: record.result,
        });
        Ok::<(), std::convert::Infallible>(())
    })?;

    Ok(PreloadResult {
        records,
        eof_offset: stream.eof_offset,
        reset_session_after_preload: options.reset_session_after_preload,
    })
}

pub fn stream_journal_file<T, E, F, C, CE>(
    path: impl AsRef<Path>,
    parser: F,
    on_record: C,
) -> Result<StreamJournalResult, JournalError>
where
    F: FnMut(&str) -> Result<T, E>,
    E: fmt::Display,
    C: FnMut(PreloadRecord<T>) -> Result<(), CE>,
    CE: fmt::Display,
{
    read_journal_records(path, parser, on_record)
}

fn read_journal_records<T, E, F, C, CE>(
    path: impl AsRef<Path>,
    mut parser: F,
    mut on_record: C,
) -> Result<StreamJournalResult, JournalError>
where
    F: FnMut(&str) -> Result<T, E>,
    E: fmt::Display,
    C: FnMut(PreloadRecord<T>) -> Result<(), CE>,
    CE: fmt::Display,
{
    let path = path.as_ref();
    let file = File::open(path).map_err(|source| JournalError::FileOpen {
        path: path.to_path_buf(),
        source,
    })?;
    let mut reader = BufReader::new(file);
    let mut offset = 0_u64;
    let mut line_number = 1_usize;
    let mut buffer = Vec::new();

    loop {
        buffer.clear();
        let bytes_read =
            reader
                .read_until(b'\n', &mut buffer)
                .map_err(|source| JournalError::FileRead {
                    path: path.to_path_buf(),
                    source,
                })?;
        if bytes_read == 0 {
            break;
        }

        let start_offset = offset;
        offset += u64::try_from(bytes_read).expect("usize always fits into u64");
        let result = std::str::from_utf8(trim_line_ending(&buffer)).map_or_else(
            |error| Err(JournalLineError::new(error.to_string())),
            |line| parser(line).map_err(|error| JournalLineError::new(error.to_string())),
        );
        on_record(PreloadRecord {
            line_number,
            start_offset,
            result,
        })
        .map_err(|error| JournalError::Callback {
            message: error.to_string(),
        })?;
        line_number += 1;
    }

    Ok(StreamJournalResult { eof_offset: offset })
}

pub fn live_poll_interval(config: &RuntimeConfig) -> Duration {
    Duration::from_millis(config.monitor.poll_interval_ms)
}

impl LiveTail {
    pub fn from_offset(path: impl AsRef<Path>, offset: u64) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            offset,
            buffered_start_offset: offset,
            buffer: Vec::new(),
        }
    }

    pub fn from_preload<T>(path: impl AsRef<Path>, preload: &PreloadResult<T>) -> Self {
        Self::from_offset(path, preload.eof_offset)
    }

    pub const fn offset(&self) -> u64 {
        self.offset
    }

    pub fn buffered_len(&self) -> usize {
        self.buffer.len()
    }

    pub fn poll<T, E, F>(&mut self, parser: F) -> Result<LiveTailPoll<T>, JournalError>
    where
        F: FnMut(&str) -> Result<T, E>,
        E: fmt::Display,
    {
        let mut warnings = Vec::new();
        let file_len = fs::metadata(&self.path)
            .map_err(|source| JournalError::MetadataRead {
                path: self.path.clone(),
                source,
            })?
            .len();

        if file_len < self.offset {
            let previous_offset = self.offset;
            self.offset = file_len;
            self.buffered_start_offset = file_len;
            self.buffer.clear();
            warnings.push(LiveTailWarning::FileTruncated {
                previous_offset,
                new_offset: file_len,
            });
            return Ok(LiveTailPoll {
                records: Vec::new(),
                warnings,
                offset: self.offset,
            });
        }

        if file_len > self.offset {
            self.read_appended_bytes()?;
        }

        let records = self.drain_complete_lines(parser);
        Ok(LiveTailPoll {
            records,
            warnings,
            offset: self.offset,
        })
    }

    fn read_appended_bytes(&mut self) -> Result<(), JournalError> {
        let mut file = File::open(&self.path).map_err(|source| JournalError::FileOpen {
            path: self.path.clone(),
            source,
        })?;
        file.seek(SeekFrom::Start(self.offset))
            .map_err(|source| JournalError::FileRead {
                path: self.path.clone(),
                source,
            })?;

        if self.buffer.is_empty() {
            self.buffered_start_offset = self.offset;
        }

        let mut appended = Vec::new();
        file.read_to_end(&mut appended)
            .map_err(|source| JournalError::FileRead {
                path: self.path.clone(),
                source,
            })?;
        self.offset += u64::try_from(appended.len()).expect("usize always fits into u64");
        self.buffer.extend_from_slice(&appended);
        Ok(())
    }

    fn drain_complete_lines<T, E, F>(&mut self, mut parser: F) -> Vec<LiveTailRecord<T>>
    where
        F: FnMut(&str) -> Result<T, E>,
        E: fmt::Display,
    {
        let mut records = Vec::new();
        let mut processed_until = 0_usize;
        let mut line_start_offset = self.buffered_start_offset;

        while let Some(newline_offset) = self.buffer[processed_until..]
            .iter()
            .position(|byte| *byte == b'\n')
        {
            let line_end = processed_until + newline_offset + 1;
            let line_bytes = &self.buffer[processed_until..line_end];
            let start_offset = line_start_offset;
            line_start_offset +=
                u64::try_from(line_end - processed_until).expect("usize always fits into u64");

            let result = parse_live_tail_line(line_bytes, start_offset, &mut parser);
            records.push(LiveTailRecord {
                start_offset,
                result,
            });
            processed_until = line_end;
        }

        if processed_until > 0 {
            self.buffer.drain(..processed_until);
            self.buffered_start_offset = line_start_offset;
        }
        if self.buffer.is_empty() {
            self.buffered_start_offset = self.offset;
        }

        records
    }
}

fn parse_live_tail_line<T, E, F>(
    bytes: &[u8],
    start_offset: u64,
    parser: &mut F,
) -> Result<T, JournalLineError>
where
    F: FnMut(&str) -> Result<T, E>,
    E: fmt::Display,
{
    std::str::from_utf8(trim_line_ending(bytes)).map_or_else(
        |error| {
            Err(JournalLineError::new(format!(
                "invalid UTF-8 in journal line at byte offset {start_offset}: {error}"
            )))
        },
        |line| parser(line).map_err(|error| JournalLineError::new(error.to_string())),
    )
}

pub fn parse_journal_filename_timestamp(path: impl AsRef<Path>) -> Option<DateTime<Utc>> {
    let file_name = path.as_ref().file_name()?.to_str()?;
    let timestamp = file_name
        .strip_prefix("Journal.")?
        .strip_suffix(".log")?
        .split('.')
        .next()?;

    parse_legacy_timestamp(timestamp).or_else(|| parse_iso_timestamp(timestamp))
}

fn is_journal_log_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            let Some(rest) = name.strip_prefix("Journal.") else {
                return false;
            };
            let Some(rest) = rest.strip_suffix(".log") else {
                return false;
            };
            let mut parts = rest.split('.');
            let Some(timestamp) = parts.next() else {
                return false;
            };
            let Some(sequence) = parts.next() else {
                return false;
            };
            parts.next().is_none()
                && sequence.len() == 2
                && sequence.bytes().all(|byte| byte.is_ascii_digit())
                && NaiveDateTime::parse_from_str(timestamp, "%Y-%m-%dT%H%M%S").is_ok()
        })
}

fn parse_legacy_timestamp(timestamp: &str) -> Option<DateTime<Utc>> {
    if timestamp.len() != 12 || !timestamp.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }

    NaiveDateTime::parse_from_str(&format!("20{timestamp}"), "%Y%m%d%H%M%S")
        .ok()
        .map(|datetime| datetime.and_utc())
}

fn parse_iso_timestamp(timestamp: &str) -> Option<DateTime<Utc>> {
    NaiveDateTime::parse_from_str(timestamp, "%Y-%m-%dT%H%M%S")
        .ok()
        .map(|datetime| datetime.and_utc())
}

fn sort_journal_files_newest_first(files: &mut [JournalFile]) {
    files.sort_by(compare_journal_files_newest_first);
}

fn compare_journal_files_newest_first(left: &JournalFile, right: &JournalFile) -> Ordering {
    right
        .effective_timestamp()
        .cmp(&left.effective_timestamp())
        .then_with(|| right.filename_timestamp.cmp(&left.filename_timestamp))
        .then_with(|| left.path.cmp(&right.path))
}

fn trim_line_ending(bytes: &[u8]) -> &[u8] {
    bytes
        .strip_suffix(b"\n")
        .unwrap_or(bytes)
        .strip_suffix(b"\r")
        .unwrap_or_else(|| bytes.strip_suffix(b"\n").unwrap_or(bytes))
}

impl JournalFile {
    fn effective_timestamp(&self) -> DateTime<Utc> {
        self.filename_timestamp
            .unwrap_or_else(|| DateTime::<Utc>::from(self.modified_at))
    }
}
