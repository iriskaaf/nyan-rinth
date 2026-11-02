use std::path::{Component, Path, PathBuf};

use sqlx::{Connection, Row, SqliteConnection, sqlite::SqliteConnectOptions};

use crate::{
	State,
	install::{InstallPhaseDetails, InstallProgressReporter},
	pack::install_from::{self, CreatePackDescription, PackDependency},
};

#[derive(Debug)]
pub(super) struct SourceInstance {
	pub path: String,
	name: String,
	directory: PathBuf,
	game_version: String,
	loader: String,
	loader_version: Option<String>,
}

fn safe_folder(path: &str) -> bool {
	!path.is_empty()
		&& Path::new(path)
			.components()
			.all(|component| matches!(component, Component::Normal(_)))
}

/// Reads only public instance metadata. No migration is run on the source DB.
pub(super) async fn read_instances(
	base: &Path,
) -> crate::Result<Vec<SourceInstance>> {
	let options = SqliteConnectOptions::new()
		.filename(base.join("app.db"))
		.read_only(true)
		.create_if_missing(false);
	let mut connection = SqliteConnection::connect_with(&options).await?;
	let modern: bool = sqlx::query_scalar(
		"SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='instances')",
	)
	.fetch_one(&mut connection)
	.await?;
	let query = if modern {
		"SELECT i.path, i.name, c.game_version, c.loader, c.loader_version
		 FROM instances i JOIN instance_content_sets c ON c.id=i.applied_content_set_id"
	} else {
		"SELECT path, name, game_version, mod_loader AS loader, mod_loader_version AS loader_version FROM profiles"
	};
	let data_root: PathBuf = sqlx::query_scalar::<_, Option<String>>(
		"SELECT custom_dir FROM settings LIMIT 1",
	)
	.fetch_optional(&mut connection)
	.await
	.ok()
	.flatten()
	.flatten()
	.map(PathBuf::from)
	.unwrap_or_else(|| base.to_path_buf());
	let rows = sqlx::query(query).fetch_all(&mut connection).await?;
	let mut instances = Vec::new();
	for row in rows {
		let path: String = row.try_get("path")?;
		if safe_folder(&path) && data_root.join("profiles").join(&path).is_dir()
		{
			instances.push(SourceInstance {
				directory: data_root.join("profiles").join(&path),
				path,
				name: row.try_get("name")?,
				game_version: row.try_get("game_version")?,
				loader: row.try_get("loader")?,
				loader_version: row.try_get("loader_version")?,
			});
		}
	}
	Ok(instances)
}

pub(super) async fn import_rinth(
	base: PathBuf,
	folder: String,
	instance_id: &str,
	reporter: InstallProgressReporter,
	details: InstallPhaseDetails,
) -> crate::Result<()> {
	let source = read_instances(&base)
		.await?
		.into_iter()
		.find(|instance| instance.path == folder)
		.ok_or_else(|| {
			crate::ErrorKind::InputError("Source instance was not found".into())
		})?;
	let loader = match source.loader.to_ascii_lowercase().as_str() {
		"vanilla" => None,
		"fabric" => Some(PackDependency::FabricLoader),
		"forge" => Some(PackDependency::Forge),
		"neoforge" | "neo-forge" => Some(PackDependency::NeoForge),
		"quilt" => Some(PackDependency::QuiltLoader),
		other => {
			return Err(crate::ErrorKind::InputError(format!(
				"Unsupported loader: {other}"
			))
			.into());
		}
	};
	let mut dependencies = std::collections::HashMap::from([(
		PackDependency::Minecraft,
		source.game_version,
	)]);
	if let Some(loader) = loader {
		let version = source.loader_version.ok_or_else(|| {
			crate::ErrorKind::InputError(
				"Source instance has no loader version".into(),
			)
		})?;
		dependencies.insert(loader, version);
	}
	let description = CreatePackDescription {
		icon: None,
		override_title: Some(source.name),
		project_id: None,
		version_id: None,
		instance_id: instance_id.to_string(),
		source_filename: None,
	};
	install_from::set_instance_information(
		instance_id.to_string(),
		&description,
		&folder,
		None,
		&dependencies,
		false,
	)
	.await?;
	let state = State::get().await?;
	super::finish_import(
		instance_id,
		source.directory,
		&state.io_semaphore,
		reporter,
		details,
	)
	.await
}

#[cfg(test)]
mod tests {
	use super::*;

	#[tokio::test]
	async fn reads_legacy_and_modern_without_migrating_source() {
		for modern in [false, true] {
			let dir = tempfile::tempdir().unwrap();
			std::fs::create_dir_all(dir.path().join("profiles/test")).unwrap();
			let options = SqliteConnectOptions::new()
				.filename(dir.path().join("app.db"))
				.create_if_missing(true);
			let mut db =
				SqliteConnection::connect_with(&options).await.unwrap();
			let schema = if modern {
				"CREATE TABLE instances(path TEXT,name TEXT,applied_content_set_id TEXT); CREATE TABLE instance_content_sets(id TEXT,game_version TEXT,loader TEXT,loader_version TEXT); INSERT INTO instances VALUES('test','Test','1'); INSERT INTO instance_content_sets VALUES('1','1.21.1','fabric','0.16.0');"
			} else {
				"CREATE TABLE profiles(path TEXT,name TEXT,game_version TEXT,mod_loader TEXT,mod_loader_version TEXT); INSERT INTO profiles VALUES('test','Test','1.21.1','fabric','0.16.0'); INSERT INTO profiles VALUES('../escape','Escape','1.21.1','vanilla',NULL);"
			};
			sqlx::raw_sql(schema).execute(&mut db).await.unwrap();
			db.close().await.unwrap();
			let before = std::fs::read(dir.path().join("app.db")).unwrap();
			let sources = read_instances(dir.path()).await.unwrap();
			assert_eq!(sources.len(), 1);
			assert_eq!(sources[0].name, "Test");
			assert_eq!(sources[0].loader_version.as_deref(), Some("0.16.0"));
			assert_eq!(
				before,
				std::fs::read(dir.path().join("app.db")).unwrap()
			);
		}
	}
}
