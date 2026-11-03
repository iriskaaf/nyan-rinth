use crate::api::Result;
use anyhow::Context;
use chrono::{Duration, Utc};
use tauri::plugin::TauriPlugin;
use tauri::{Manager, Runtime, UserAttentionType};
use theseus::prelude::*;

pub fn init<R: Runtime>() -> TauriPlugin<R> {
	tauri::plugin::Builder::<R>::new("auth")
		.invoke_handler(tauri::generate_handler![
			ely_login,
			ely_skin_texture,
			ely_skins_connect,
			ely_skins_connected,
			ely_skins_disconnect,
			ely_skins_catalog,
			ely_skins_personal,
			ely_skins_preview,
			ely_skins_apply,
			ely_skins_upload,
			offline_login,
			get_external_auth_providers,
			get_external_auth_library_states,
			install_external_auth_library,
			select_external_auth_library,
			check_reachable,
			login,
			remove_user,
			get_default_user,
			set_default_user,
			get_users,
		])
		.build()
}

/// Checks if the authentication servers are reachable.
#[tauri::command]
pub async fn check_reachable() -> Result<()> {
	minecraft_auth::check_reachable().await?;
	Ok(())
}

/// Authenticate a user with Hydra - part 1
/// This begins the authentication flow quasi-synchronously, returning a URL to visit (that the user will sign in at)
#[tauri::command]
pub async fn login<R: Runtime>(
	app: tauri::AppHandle<R>,
) -> Result<Option<Credentials>> {
	let flow = minecraft_auth::begin_login().await?;

	let start = Utc::now();

	if let Some(window) = app.get_webview_window("signin") {
		window.close()?;
	}

	let window = tauri::WebviewWindowBuilder::new(
		&app,
		"signin",
		tauri::WebviewUrl::External(
			flow.auth_request_uri
				.parse()
				.context("error parsing auth redirect URL")?,
		),
	)
	.title("Sign into Modrinth")
	.always_on_top(true)
	.min_inner_size(500.0, 500.0)
	.inner_size(1000.0, 700.0)
	.focused(true)
	.center()
	.build()?;

	window.request_user_attention(Some(UserAttentionType::Critical))?;

	while (Utc::now() - start) < Duration::minutes(10) {
		if window.title().is_err() {
			// user closed window, cancelling flow
			return Ok(None);
		}

		if window
			.url()?
			.as_str()
			.starts_with("https://login.live.com/oauth20_desktop.srf")
			&& let Some((_, code)) =
				window.url()?.query_pairs().find(|x| x.0 == "code")
		{
			window.close()?;
			let val = minecraft_auth::finish_login(&code.clone(), flow).await?;

			return Ok(Some(val));
		}

		tokio::time::sleep(std::time::Duration::from_millis(50)).await;
	}

	window.close()?;
	Ok(None)
}

#[tauri::command]
pub async fn remove_user(user: uuid::Uuid) -> Result<()> {
	theseus::nyanrinth::ely_skins::disconnect(user).await?;
	Ok(minecraft_auth::remove_user(user).await?)
}

#[tauri::command]
pub async fn get_default_user() -> Result<Option<uuid::Uuid>> {
	Ok(minecraft_auth::get_default_user().await?)
}

#[tauri::command]
pub async fn set_default_user(user: uuid::Uuid) -> Result<()> {
	Ok(minecraft_auth::set_default_user(user).await?)
}

/// Get a copy of the list of all user credentials
#[tauri::command]
pub async fn get_users() -> Result<Vec<Credentials>> {
	Ok(minecraft_auth::users().await?)
}

#[tauri::command]
pub async fn ely_skins_connect(
	user: uuid::Uuid,
	login: &str,
	password: &str,
	totp: Option<&str>,
) -> Result<()> {
	Ok(
		theseus::nyanrinth::ely_skins::connect(user, login, password, totp)
			.await?,
	)
}
#[tauri::command]
pub async fn ely_skins_connected(user: uuid::Uuid) -> Result<bool> {
	Ok(theseus::nyanrinth::ely_skins::connected(user).await?)
}
#[tauri::command]
pub async fn ely_skins_disconnect(user: uuid::Uuid) -> Result<()> {
	Ok(theseus::nyanrinth::ely_skins::disconnect(user).await?)
}
#[tauri::command]
pub async fn ely_skins_catalog(
	page: u32,
	model: &str,
	tags: &str,
) -> Result<theseus::nyanrinth::ely_skins::CatalogPage> {
	Ok(theseus::nyanrinth::ely_skins::catalog(page, model, tags).await?)
}
#[tauri::command]
pub async fn ely_skins_personal(
	user: uuid::Uuid,
	page: u32,
	model: &str,
	tags: &str,
	uploaded: bool,
) -> Result<theseus::nyanrinth::ely_skins::CatalogPage> {
	Ok(theseus::nyanrinth::ely_skins::personal(
		user, page, model, tags, uploaded,
	)
	.await?)
}
#[tauri::command]
pub async fn ely_skins_preview(
	skin_id: u64,
) -> Result<theseus::nyanrinth::ely_skins::SkinPreview> {
	Ok(theseus::nyanrinth::ely_skins::preview(skin_id).await?)
}
#[tauri::command]
pub async fn ely_skins_apply(user: uuid::Uuid, skin_id: u64) -> Result<()> {
	Ok(theseus::nyanrinth::ely_skins::apply(user, skin_id).await?)
}
#[tauri::command]
pub async fn ely_skins_upload(
	user: uuid::Uuid,
	bytes: Vec<u8>,
) -> Result<theseus::nyanrinth::ely_skins::CatalogSkin> {
	Ok(theseus::nyanrinth::ely_skins::upload(user, bytes).await?)
}

#[tauri::command]
pub async fn ely_login(
	username: &str,
	password: &str,
	totp: Option<&str>,
) -> Result<Credentials> {
	Ok(theseus::nyanrinth::authentication::authenticate_ely(
		username, password, totp,
	)
	.await?)
}

#[tauri::command]
pub async fn ely_skin_texture(
	username: &str,
) -> Result<Option<theseus::nyanrinth::authentication::ElySkinPreview>> {
	Ok(theseus::nyanrinth::authentication::ely_skin_texture(username).await?)
}

/// This code is modified by AstralRinth
/// Create new offline user
#[tauri::command]
pub async fn offline_login(name: &str) -> Result<Credentials> {
	let credentials = minecraft_auth::offline_auth(name).await?;
	Ok(credentials)
}

#[tauri::command]
pub fn get_external_auth_providers()
-> Vec<theseus::nyanrinth::authentication::ExternalAuthProviderMetadata> {
	theseus::nyanrinth::authentication::external_auth_providers()
		.iter()
		.map(|p| p.metadata())
		.collect()
}
#[tauri::command]
pub async fn get_external_auth_library_states()
-> Result<Vec<theseus::nyanrinth::authentication::ExternalAuthLibraryState>> {
	Ok(
		theseus::nyanrinth::authentication::external_auth_library_states()
			.await?,
	)
}
#[tauri::command]
pub async fn install_external_auth_library(
	provider: &str,
	asset_name: &str,
) -> Result<()> {
	Ok(theseus::nyanrinth::authentication::install_external_auth_library_version(provider, asset_name).await?)
}
#[tauri::command]
pub async fn select_external_auth_library(
	provider: &str,
	asset_name: &str,
) -> Result<bool> {
	Ok(theseus::nyanrinth::authentication::select_external_auth_library_version(provider, asset_name).await?)
}
