use crate::api::Result;
use crate::api::astralrinth::{
	authenticate_external_provider, get_external_auth_library_states,
	get_external_auth_providers, install_external_auth_library,
	select_external_auth_library,
};
use chrono::{Duration, Utc};
use tauri::plugin::TauriPlugin;
use tauri::{Manager, Runtime, UserAttentionType};
use theseus::prelude::*;

pub fn init<R: Runtime>() -> TauriPlugin<R> {
	tauri::plugin::Builder::<R>::new("auth")
		.invoke_handler(tauri::generate_handler![
			ely_login,
			open_ely_skins,
			ely_skin_texture,
			offline_login,
			get_external_auth_providers,
			get_external_auth_library_states,
			install_external_auth_library,
			select_external_auth_library,
			authenticate_external_provider,
			check_reachable,
			login,
			remove_user,
			get_default_user,
			set_default_user,
			get_users,
		])
		.build()
}

#[tauri::command]
pub async fn ely_login(
	username: &str,
	password: &str,
	totp: Option<&str>,
) -> Result<Credentials> {
	Ok(
		theseus::models::astralrinth::authentication::authenticate_ely(
			username, password, totp,
		)
		.await?,
	)
}

#[tauri::command]
pub async fn ely_skin_texture(
	username: &str,
) -> Result<Option<theseus::models::astralrinth::authentication::ElySkinPreview>>
{
	Ok(
		theseus::models::astralrinth::authentication::ely_skin_texture(
			username,
		)
		.await?,
	)
}

#[tauri::command]
pub async fn open_ely_skins<R: Runtime>(
	app: tauri::AppHandle<R>,
	upload: bool,
) -> Result<()> {
	if let Some(window) = app.get_webview_window("ely-skins") {
		if upload {
			window.navigate("https://ely.by/skins/add".parse().unwrap())?;
		}
		window.show()?;
		window.set_focus()?;
		return Ok(());
	}
	let url = if upload {
		"https://ely.by/skins/add"
	} else {
		"https://ely.by/skins"
	};
	let window = tauri::WebviewWindowBuilder::new(
		&app,
		"ely-skins",
		tauri::WebviewUrl::External(url.parse().unwrap()),
	)
	.title("NyanRinth — скины Ely.by")
	.inner_size(1100.0, 800.0)
	.center()
	.build()?;
	if let Some(main) = app.get_webview_window("main") {
		let catalog = window.clone();
		main.on_window_event(move |event| {
			if matches!(event, tauri::WindowEvent::Destroyed) {
				let _ = catalog.destroy();
			}
		});
	}
	let retained_window = window.clone();
	window.on_window_event(move |event| {
		if let tauri::WindowEvent::CloseRequested { api, .. } = event {
			api.prevent_close();
			let _ = retained_window.hide();
		}
	});
	Ok(())
}

/// This code is modified by AstralRinth
/// Create new offline user
#[tauri::command]
pub async fn offline_login(name: &str) -> Result<Credentials> {
	let credentials = minecraft_auth::offline_auth(name).await?;
	Ok(credentials)
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
		tauri::WebviewUrl::External(flow.auth_request_uri.parse().map_err(
			|_| {
				theseus::ErrorKind::OtherError(
					"Error parsing auth redirect URL".to_string(),
				)
				.as_error()
			},
		)?),
	)
	.title("Sign into NyanRinth")
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
