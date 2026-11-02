//! NyanRinth orchestration around Modrinth v0.22.1's option catalogue and codecs.
//! The upstream codecs retain the GPL-3.0 licence of this repository.

macro_rules! bail {
	($error:expr) => {
		return Err(($error).into())
	};
}

mod api_types;
#[allow(dead_code)]
mod catalog;
mod options_file;
mod value;

use std::collections::{BTreeMap, HashMap};
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, RwLock};

use crate::state::{InstanceInstallStage, InstanceMetadata, State};
use catalog::*;
use options_file::{GameOptionsDocument, read_document, sha1_bytes};
pub use value::CanonicalValue;

const MAX_OPTIONS_BYTES: usize = 2 * 1024 * 1024;
const MAX_OPTIONS_LINES: usize = 16_384;
const MAX_KEY_BYTES: usize = 1_024;
const MAX_VALUE_BYTES: usize = 32 * 1024;
static SYNC_LOCK: Mutex<()> = Mutex::const_new(());
static LAST_ERROR: RwLock<Option<String>> = RwLock::const_new(None);

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
struct Config {
	enabled: bool,
	source_instance: Option<String>,
	excluded_instances: Vec<String>,
	disabled_options: Vec<String>,
	values: BTreeMap<String, CanonicalValue>,
}

#[derive(Default, Serialize, Deserialize)]
struct Snapshot {
	hash: String,
	values: BTreeMap<String, CanonicalValue>,
}

#[derive(Deserialize)]
pub struct SyncUpdate {
	pub enabled: bool,
	pub source_instance: Option<String>,
	pub excluded_instances: Vec<String>,
	pub disabled_options: Vec<String>,
}

#[derive(Serialize)]
pub struct SyncInstance {
	id: String,
	name: String,
	game_version: String,
	status: &'static str,
	excluded: bool,
}

#[derive(Serialize)]
pub struct SyncOption {
	id: String,
	key: String,
	category: String,
	enabled: bool,
	value: Option<CanonicalValue>,
}

#[derive(Serialize)]
pub struct SyncView {
	last_error: Option<String>,
	enabled: bool,
	source_instance: Option<String>,
	instances: Vec<SyncInstance>,
	options: Vec<SyncOption>,
}

fn eligible_option(id: &str) -> bool {
	!matches!(id, "fullscreen" | "window_width" | "window_height")
}

fn read_values(
	document: &GameOptionsDocument,
	version: &str,
) -> BTreeMap<String, CanonicalValue> {
	if !supported_settings_cover_game_version(version) {
		return BTreeMap::new();
	}
	all_supported_settings()
		.filter_map(|definition| {
			if !eligible_option(definition.id) {
				return None;
			}
			let key = observed_physical_key(definition, document, version)?;
			let raw = document.value(&key)?;
			let value =
				decode_value_for_version(definition, &key, raw, version)?;
			Some((definition.id.to_string(), value))
		})
		.collect()
}

fn apply_values(
	document: &mut GameOptionsDocument,
	version: &str,
	config: &Config,
) -> crate::Result<bool> {
	let mut changed = false;
	for (id, value) in &config.values {
		if config.disabled_options.contains(id) || !eligible_option(id) {
			continue;
		}
		let Some(definition) = setting_by_id(id) else {
			continue;
		};
		let Some(observed) =
			observed_physical_key(definition, document, version)
		else {
			continue;
		};
		let Some(key) = target_key_for_version(definition, version) else {
			continue;
		};
		let raw = document.value(&observed);
		let Some(encoded) = encode_value(definition, key, value, version, raw)
		else {
			continue;
		};
		changed |= document.set(key, &encoded, true)?;
	}
	Ok(changed)
}

async fn load_config(state: &State) -> crate::Result<Config> {
	let payload: Option<String> = sqlx::query_scalar(
		"SELECT payload FROM nyan_game_sync_config WHERE id=1",
	)
	.fetch_optional(&state.pool)
	.await?;
	Ok(match payload {
		Some(payload) => serde_json::from_str(&payload)?,
		None => Config::default(),
	})
}

async fn save_config(state: &State, config: &Config) -> crate::Result<()> {
	sqlx::query("INSERT INTO nyan_game_sync_config(id,payload) VALUES(1,?) ON CONFLICT(id) DO UPDATE SET payload=excluded.payload")
		.bind(serde_json::to_string(config)?).execute(&state.pool).await?;
	Ok(())
}

fn path(metadata: &InstanceMetadata, state: &State) -> std::path::PathBuf {
	state
		.directories
		.instances_dir()
		.join(&metadata.instance.path)
		.join("options.txt")
}

fn running(metadata: &InstanceMetadata, state: &State) -> bool {
	state
		.process_manager
		.get_all()
		.iter()
		.any(|process| process.instance_id == metadata.instance.id)
}

fn installed(metadata: &InstanceMetadata) -> bool {
	metadata.instance.install_stage == InstanceInstallStage::Installed
		&& !metadata.quarantined
}

pub async fn get_config() -> crate::Result<SyncView> {
	let _guard = SYNC_LOCK.lock().await;
	let state = State::get().await?;
	let config = load_config(&state).await?;
	let mut instances = Vec::new();
	for metadata in crate::state::list_instances(&state.pool).await? {
		let version = &metadata.applied_content_set.game_version;
		let status = if !installed(&metadata) {
			"installing"
		} else if running(&metadata, &state) {
			"running"
		} else if !supported_settings_cover_game_version(version) {
			"unsupported"
		} else if !path(&metadata, &state).exists() {
			"waiting"
		} else if read_document(&path(&metadata, &state)).await.is_err() {
			"unreadable"
		} else {
			"ready"
		};
		instances.push(SyncInstance {
			excluded: config.excluded_instances.contains(&metadata.instance.id),
			id: metadata.instance.id,
			name: metadata.instance.name,
			game_version: version.clone(),
			status,
		});
	}
	let options = all_supported_settings()
		.filter(|definition| eligible_option(definition.id))
		.map(|definition| SyncOption {
			id: definition.id.to_string(),
			key: definition
				.keys
				.first()
				.unwrap_or(&definition.id)
				.to_string(),
			category: definition.category.to_string(),
			enabled: if config.source_instance.is_none() {
				definition.default_on
			} else {
				!config.disabled_options.iter().any(|id| id == definition.id)
			},
			value: config.values.get(definition.id).cloned(),
		})
		.collect();
	Ok(SyncView {
		last_error: LAST_ERROR.read().await.clone(),
		enabled: config.enabled,
		source_instance: config.source_instance,
		instances,
		options,
	})
}

/// Enabling sync always asks the caller for a source; existing users remain opted out.
pub async fn update_config(update: SyncUpdate) -> crate::Result<SyncView> {
	{
		let _guard = SYNC_LOCK.lock().await;
		let state = State::get().await?;
		let previous = load_config(&state).await?;
		let mut config = Config {
			enabled: update.enabled,
			source_instance: update.source_instance,
			excluded_instances: update.excluded_instances,
			disabled_options: update.disabled_options,
			values: previous.values.clone(),
		};
		if config.enabled
			&& (!previous.enabled
				|| config.source_instance != previous.source_instance)
		{
			let id = config.source_instance.as_deref().ok_or_else(|| {
				crate::ErrorKind::InputError("Выберите исходную сборку".into())
			})?;
			let metadata = crate::state::get_instance(id, &state.pool)
				.await?
				.ok_or_else(|| {
				crate::ErrorKind::InputError("Сборка не найдена".into())
			})?;
			if !installed(&metadata)
				|| running(&metadata, &state)
				|| config
					.excluded_instances
					.iter()
					.any(|excluded| excluded == id)
			{
				return Err(crate::ErrorKind::InputError("Исходная сборка должна быть установлена, включена в синхронизацию и закрыта".into()).into());
			}
			let (document, _) = read_document(&path(&metadata, &state)).await?;
			config.values = read_values(
				&document,
				&metadata.applied_content_set.game_version,
			);
			if config.values.is_empty() {
				return Err(crate::ErrorKind::InputError("В этой сборке ещё нет поддерживаемых настроек. Запустите игру и закройте её.".into()).into());
			}
		}
		if previous.enabled != config.enabled
			|| previous.source_instance != config.source_instance
			|| previous.excluded_instances != config.excluded_instances
			|| previous.disabled_options != config.disabled_options
		{
			sqlx::query("DELETE FROM nyan_game_sync_instances")
				.execute(&state.pool)
				.await?;
		}
		save_config(&state, &config).await?;
	}
	sync_now().await?;
	get_config().await
}

/// Updates only observed, supported keys. A concurrently saved file is retried next time.
fn replace_document(
	path: &Path,
	expected: &[u8],
	bytes: &[u8],
	backup: &Path,
) -> crate::Result<bool> {
	if std::fs::read(path)? != expected {
		return Ok(false);
	}
	std::fs::create_dir_all(backup)?;
	let backup_path = backup.join(format!("{}.txt", sha1_bytes(expected)));
	if !backup_path.exists() {
		std::fs::write(backup_path, expected)?;
	}
	let mut temporary = tempfile::NamedTempFile::new_in(
		path.parent().expect("options parent"),
	)?;
	temporary.write_all(bytes)?;
	temporary.as_file().sync_all()?;
	if std::fs::read(path)? != expected {
		return Ok(false);
	}
	temporary.persist(path).map_err(|error| error.error)?;
	Ok(true)
}

fn backup_directory(root: &Path, instance_id: &str) -> PathBuf {
	root.join("game-settings-backups")
		.join(sha1_bytes(instance_id.as_bytes()))
}

async fn sync_round() -> crate::Result<()> {
	let _guard = SYNC_LOCK.lock().await;
	let state = State::get().await?;
	let mut config = load_config(&state).await?;
	if !config.enabled {
		return Ok(());
	}
	let mut observations = Vec::new();
	for metadata in crate::state::list_instances(&state.pool).await? {
		if !installed(&metadata)
			|| config.excluded_instances.contains(&metadata.instance.id)
		{
			continue;
		}
		let file = path(&metadata, &state);
		if !file.exists() {
			continue;
		}
		let Ok((document, bytes)) = read_document(&file).await else {
			continue;
		};
		let modified = tokio::fs::metadata(&file)
			.await?
			.modified()
			.unwrap_or(SystemTime::UNIX_EPOCH);
		if modified
			.elapsed()
			.is_ok_and(|age| age < Duration::from_millis(800))
		{
			continue;
		}
		observations.push((modified, metadata, file, document, bytes));
	}
	observations.sort_by(|left, right| left.0.cmp(&right.0));
	let mut snapshots = HashMap::new();
	let mut changed = false;
	for (_, metadata, _, document, bytes) in &observations {
		let payload: Option<String> = sqlx::query_scalar(
			"SELECT payload FROM nyan_game_sync_instances WHERE instance_id=?",
		)
		.bind(&metadata.instance.id)
		.fetch_optional(&state.pool)
		.await?;
		let values =
			read_values(document, &metadata.applied_content_set.game_version);
		if let Some(payload) = payload {
			let previous: Snapshot = serde_json::from_str(&payload)?;
			if previous.hash != sha1_bytes(bytes) {
				for (id, value) in &values {
					if !config.disabled_options.contains(id)
						&& previous.values.get(id) != Some(value)
					{
						config.values.insert(id.clone(), value.clone());
						changed = true;
					}
				}
			}
		}
		snapshots.insert(
			metadata.instance.id.clone(),
			Snapshot {
				hash: sha1_bytes(bytes),
				values,
			},
		);
	}
	if changed {
		save_config(&state, &config).await?;
	}
	for (_, metadata, file, mut document, bytes) in observations {
		let mut snapshot = snapshots
			.remove(&metadata.instance.id)
			.expect("observed snapshot");
		if !running(&metadata, &state)
			&& apply_values(
				&mut document,
				&metadata.applied_content_set.game_version,
				&config,
			)? {
			let output = document.serialize()?;
			let backup = backup_directory(
				&state.directories.config_dir,
				&metadata.instance.id,
			);
			if replace_document(&file, &bytes, &output, &backup)? {
				snapshot.hash = sha1_bytes(&output);
				snapshot.values = read_values(
					&document,
					&metadata.applied_content_set.game_version,
				);
			}
		}
		sqlx::query("INSERT INTO nyan_game_sync_instances(instance_id,payload) VALUES(?,?) ON CONFLICT(instance_id) DO UPDATE SET payload=excluded.payload")
			.bind(&metadata.instance.id).bind(serde_json::to_string(&snapshot)?).execute(&state.pool).await?;
	}
	Ok(())
}

pub(crate) fn start_worker() {
	tokio::spawn(async {
		loop {
			tokio::time::sleep(Duration::from_secs(3)).await;
			if let Err(error) = sync_now().await {
				tracing::warn!("Game settings sync: {error}");
			}
		}
	});
}

#[cfg(test)]
mod tests;

pub(crate) async fn launch_guard() -> tokio::sync::MutexGuard<'static, ()> {
	if let Err(error) = sync_now().await {
		tracing::warn!("Game settings before launch: {error}");
	}
	SYNC_LOCK.lock().await
}

pub async fn sync_now() -> crate::Result<()> {
	let result = sync_round().await;
	*LAST_ERROR.write().await = result.as_ref().err().map(ToString::to_string);
	result
}
