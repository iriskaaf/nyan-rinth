use crate::nyanrinth::authentication::ExternalAuthLibrary;
use crate::{Result, State};

use serde::Deserialize;
use std::path::{Path, PathBuf};
use tokio::fs;

#[derive(Deserialize)]
struct ExternalAuthLibraryRelease {
	assets: Vec<ExternalAuthLibraryAsset>,
}

#[derive(Deserialize)]
struct ExternalAuthLibraryAsset {
	name: String,
	browser_download_url: String,
	digest: Option<String>,
}

/// Installs the verified release shipped with NyanRinth, including when GitHub is unavailable.
pub async fn install_bundled_ely_library(
	library: ExternalAuthLibrary,
) -> Result<PathBuf> {
	use sha2::{Digest, Sha256};
	const JAR: &[u8] =
		include_bytes!("../../resources/authlib-injector-1.2.8.jar");
	const NAME: &str = "authlib-injector-1.2.8.jar";
	if hex_digest(&Sha256::digest(JAR))
		!= "9c7f4343e6c82034958ffb48c14a2cb0c85928be7283103ce17da00c6d5a7b10"
	{
		return Err(crate::ErrorKind::InputError(
			"Bundled authlib-injector checksum mismatch".into(),
		)
		.into());
	}
	let state = State::get().await?;
	let directory =
		authlib_injector_dir(&state.directories.libraries_dir(), library);
	fs::create_dir_all(&directory).await?;
	let path = directory.join(NAME);
	fs::write(&path, JAR).await?;
	save_authlib_injector_library_selection("elyby", NAME).await?;
	Ok(path)
}

/// Resolves the library selected in SQLite and verifies its local file.
pub async fn get_authlib_injector_library(
	provider_id: &str,
	provider_name: &str,
	library: ExternalAuthLibrary,
) -> Result<PathBuf> {
	let state = State::get().await?;
	let libraries_dir = state.directories.libraries_dir();
	let asset_name = sqlx::query_scalar::<_, String>(
		"SELECT asset_name FROM external_auth_libraries WHERE provider_id = ?",
	)
	.bind(provider_id)
	.fetch_optional(&state.pool)
	.await?;
	let Some(asset_name) = asset_name else {
		return Err(missing_external_auth_library(provider_name));
	};

	if validate_library_asset_name(&asset_name).is_err() {
		return Err(missing_external_auth_library(provider_name));
	}
	let path = authlib_injector_path(&libraries_dir, library, &asset_name);
	if !path.is_file() {
		return Err(missing_external_auth_library(provider_name));
	}

	tracing::debug!(
		provider = provider_id,
		asset = %asset_name,
		"[NyanRinth] Auth library selected"
	);
	Ok(path)
}

/// Lists valid library files installed for a provider.
pub async fn local_authlib_injector_libraries(
	library: ExternalAuthLibrary,
) -> Result<Vec<String>> {
	let state = State::get().await?;
	let mut entries = match fs::read_dir(authlib_injector_dir(
		&state.directories.libraries_dir(),
		library,
	))
	.await
	{
		Ok(entries) => entries,
		Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
			return Ok(Vec::new());
		}
		Err(error) => return Err(error.into()),
	};
	let mut asset_names = Vec::new();

	while let Some(entry) = entries.next_entry().await? {
		let file_name = entry.file_name();
		let Some(asset_name) = file_name.to_str() else {
			continue;
		};
		if entry.file_type().await?.is_file()
			&& validate_library_asset_name(asset_name).is_ok()
		{
			asset_names.push(asset_name.to_string());
		}
	}

	Ok(asset_names)
}

/// Downloads an exact remote asset and stores it as the provider selection.
pub async fn install_authlib_injector_library(
	provider_id: &str,
	library: ExternalAuthLibrary,
	asset_name: &str,
) -> Result<()> {
	validate_library_asset_name(asset_name)?;
	let asset = fetch_external_auth_library_release(library)
		.await?
		.assets
		.into_iter()
		.find(|asset| asset.name == asset_name)
		.ok_or_else(|| {
			crate::ErrorKind::InputError(format!(
				"Library asset not found: {asset_name}"
			))
		})?;
	install_authlib_injector_asset(provider_id, library, asset).await?;

	Ok(())
}

/// Downloads and selects the newest provider library available remotely.
pub async fn install_latest_authlib_injector_library(
	provider_id: &str,
	library: ExternalAuthLibrary,
) -> Result<PathBuf> {
	let asset = fetch_external_auth_library_release(library)
		.await?
		.assets
		.into_iter()
		.filter_map(|asset| {
			library_version(&asset.name).map(|version| (asset, version))
		})
		.max_by(|(left_asset, left_version), (right_asset, right_version)| {
			left_version
				.cmp(right_version)
				.then_with(|| left_asset.name.cmp(&right_asset.name))
		})
		.map(|(asset, _)| asset)
		.ok_or_else(|| {
			crate::ErrorKind::InputError(
				"No compatible external authentication library was found"
					.to_string(),
			)
		})?;

	install_authlib_injector_asset(provider_id, library, asset).await
}

async fn install_authlib_injector_asset(
	provider_id: &str,
	library: ExternalAuthLibrary,
	asset: ExternalAuthLibraryAsset,
) -> Result<PathBuf> {
	validate_library_asset_name(&asset.name)?;
	let state = State::get().await?;
	let libraries_dir = state.directories.libraries_dir();
	let directory = authlib_injector_dir(&libraries_dir, library);
	fs::create_dir_all(&directory).await?;
	let path = directory.join(&asset.name);

	tracing::debug!(
		provider = provider_id,
		asset = %asset.name,
		"[NyanRinth] Auth library download started"
	);
	let bytes = fetch_bytes_from_url(&asset.browser_download_url).await?;
	use sha2::{Digest, Sha256};
	let expected = asset
		.digest
		.as_deref()
		.and_then(|digest| digest.strip_prefix("sha256:"))
		.ok_or_else(|| {
			crate::ErrorKind::InputError(
				"Authentication library has no SHA-256 checksum".into(),
			)
		})?;
	if hex_digest(&Sha256::digest(&bytes)) != expected {
		return Err(crate::ErrorKind::InputError(
			"Authentication library checksum mismatch".into(),
		)
		.into());
	}
	let relative_path = path
		.strip_prefix(&libraries_dir)?
		.to_string_lossy()
		.into_owned();
	write_file_to_libraries(&relative_path, &bytes).await?;
	save_authlib_injector_library_selection(provider_id, &asset.name).await?;

	Ok(path)
}

/// Stores a selection only when the exact local file exists.
pub async fn select_local_authlib_injector_library(
	provider_id: &str,
	library: ExternalAuthLibrary,
	asset_name: &str,
) -> Result<bool> {
	validate_library_asset_name(asset_name)?;
	let state = State::get().await?;
	let path = authlib_injector_path(
		&state.directories.libraries_dir(),
		library,
		asset_name,
	);
	if !path.is_file() {
		return Ok(false);
	}

	save_authlib_injector_library_selection(provider_id, asset_name).await?;

	Ok(true)
}

async fn save_authlib_injector_library_selection(
	provider_id: &str,
	asset_name: &str,
) -> Result<()> {
	let state = State::get().await?;
	sqlx::query(
		r#"
		INSERT INTO external_auth_libraries (provider_id, asset_name)
		VALUES (?, ?)
		ON CONFLICT(provider_id) DO UPDATE SET asset_name = excluded.asset_name
		"#,
	)
	.bind(provider_id)
	.bind(asset_name)
	.execute(&state.pool)
	.await?;

	Ok(())
}

fn missing_external_auth_library(provider_name: &str) -> crate::Error {
	crate::ErrorKind::ExternalAuthLibraryNotInstalled {
		provider_name: provider_name.to_string(),
	}
	.into()
}

fn validate_library_asset_name(asset_name: &str) -> Result<()> {
	let path = Path::new(asset_name);
	let is_file_name = path
		.file_name()
		.and_then(|name| name.to_str())
		.is_some_and(|name| name == asset_name);
	if !is_file_name || !asset_name.ends_with(".jar") {
		return Err(crate::ErrorKind::InputError(format!(
			"Invalid external authentication library asset: {asset_name}",
		))
		.into());
	}

	Ok(())
}

fn authlib_injector_dir(
	libraries_dir: &Path,
	library: ExternalAuthLibrary,
) -> PathBuf {
	libraries_dir
		.join("nyanrinth")
		.join(library.cache_directory)
}

fn authlib_injector_path(
	libraries_dir: &Path,
	library: ExternalAuthLibrary,
	asset_name: &str,
) -> PathBuf {
	authlib_injector_dir(libraries_dir, library).join(asset_name)
}

async fn fetch_external_auth_library_release(
	library: ExternalAuthLibrary,
) -> Result<ExternalAuthLibraryRelease> {
	Ok(reqwest::Client::new()
		.get(library.release_url)
		.header("User-Agent", "NyanRinth/1.0.0")
		.timeout(std::time::Duration::from_secs(15))
		.send()
		.await?
		.error_for_status()?
		.json::<ExternalAuthLibraryRelease>()
		.await?)
}

fn library_version(asset_name: &str) -> Option<Vec<u64>> {
	if validate_library_asset_name(asset_name).is_err()
		|| !asset_name.contains("authlib-injector")
	{
		return None;
	}

	asset_name
		.split(|character: char| {
			!character.is_ascii_digit() && character != '.'
		})
		.find_map(|candidate| {
			let version = candidate.trim_matches('.');
			if !version.contains('.') {
				return None;
			}

			version
				.split('.')
				.map(str::parse)
				.collect::<std::result::Result<Vec<u64>, _>>()
				.ok()
		})
}

async fn fetch_bytes_from_url(url: &str) -> Result<bytes::Bytes> {
	Ok(reqwest::Client::builder()
		.https_only(true)
		.timeout(std::time::Duration::from_secs(60))
		.build()?
		.get(url)
		.header("User-Agent", "NyanRinth")
		.send()
		.await?
		.error_for_status()?
		.bytes()
		.await?)
}

fn hex_digest(bytes: &[u8]) -> String {
	bytes.iter().map(|b| format!("{b:02x}")).collect()
}
async fn write_file_to_libraries(
	relative_path: &str,
	bytes: &bytes::Bytes,
) -> Result<()> {
	let state = State::get().await?;
	fs::write(state.directories.libraries_dir().join(relative_path), bytes)
		.await?;
	Ok(())
}
