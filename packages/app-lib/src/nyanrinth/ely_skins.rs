use crate::{Result, State};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use reqwest::{
	Client, Response,
	cookie::{CookieStore, Jar},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
	collections::HashMap,
	sync::{Arc, LazyLock},
	time::Duration,
};
use tokio::sync::Mutex;
use url::Url;
use uuid::Uuid;

const SITE: &str = "https://ely.by";
const ACCOUNTS: &str = "https://account.ely.by";
static SESSIONS: LazyLock<Mutex<HashMap<Uuid, SiteSession>>> =
	LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Clone)]
struct SiteSession {
	client: Client,
	jar: Arc<Jar>,
}

#[derive(Serialize, Deserialize)]
pub struct CatalogSkin {
	pub id: u64,
	pub skin_url: String,
	pub is_slim: bool,
	#[serde(default)]
	pub tags: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct CatalogPage {
	pub items: Vec<CatalogSkin>,
	pub current: u32,
	pub last: u32,
	#[serde(default)]
	pub total_items: u32,
	#[serde(default)]
	pub equipped_id: Option<u64>,
}

fn error(message: &str) -> crate::Error {
	anyhow::anyhow!("{}", message)
}

fn new_session() -> Result<SiteSession> {
	let jar = Arc::new(Jar::default());
	let client = Client::builder()
		.cookie_provider(jar.clone())
		.https_only(true)
		.redirect(reqwest::redirect::Policy::none())
		.timeout(Duration::from_secs(25))
		.user_agent("NyanRinth/1.0.5")
		.build()?;
	Ok(SiteSession { client, jar })
}

async fn expected_username(user: Uuid) -> Result<String> {
	let state = State::get().await?;
	sqlx::query_scalar("SELECT username FROM minecraft_users WHERE uuid = ? AND account_type = 'elyby'")
		.bind(user.to_string()).fetch_optional(&state.pool).await?
		.ok_or_else(|| error("Сначала добавьте аккаунт Ely.by в лаунчер."))
}

fn site_url(value: &str) -> Result<Url> {
	let url = Url::parse(SITE)
		.unwrap()
		.join(value)
		.map_err(|_| error("Некорректный адрес Ely.by."))?;
	if url.scheme() != "https"
		|| url.host_str() != Some("ely.by")
		|| !url.username().is_empty()
		|| url.password().is_some()
		|| url.port().is_some()
	{
		return Err(error("Ely.by вернул неожиданный адрес перенаправления."));
	}
	Ok(url)
}

fn current_user(html: &str) -> Result<Value> {
	let value = html
		.split("alight.service.currentUser = ")
		.nth(1)
		.ok_or_else(|| {
			error("Изменился формат ответа Ely.by. Обновите NyanRinth.")
		})?;
	serde_json::Deserializer::from_str(value)
		.into_iter::<Value>()
		.next()
		.ok_or_else(|| error("Некорректный профиль Ely.by."))?
		.map_err(Into::into)
}

fn matches_user(value: &Value, username: &str) -> bool {
	value["id"].as_u64().is_some_and(|id| id > 0)
		&& value["nickname"]
			.as_str()
			.is_some_and(|name| name.eq_ignore_ascii_case(username))
}

async fn session_path(user: Uuid) -> Result<std::path::PathBuf> {
	Ok(State::get()
		.await?
		.directories
		.settings_dir
		.join("ely-skin-sessions")
		.join(format!("{user}.dpapi")))
}

async fn save_session(user: Uuid, session: &SiteSession) -> Result<()> {
	let cookie = session
		.jar
		.cookies(&Url::parse(SITE).unwrap())
		.ok_or_else(|| error("Ely.by не создал сессию сайта."))?;
	let bytes = protect(cookie.as_bytes(), false)?;
	let path = session_path(user).await?;
	tokio::fs::create_dir_all(path.parent().unwrap()).await?;
	tokio::fs::write(path, bytes).await?;
	Ok(())
}

async fn load_session(user: Uuid) -> Result<Option<SiteSession>> {
	if let Some(session) = SESSIONS.lock().await.get(&user).cloned() {
		return Ok(Some(session));
	}
	let bytes = match tokio::fs::read(session_path(user).await?).await {
		Ok(bytes) => bytes,
		Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
		Err(e) => return Err(e.into()),
	};
	let Ok(bytes) = protect(&bytes, true) else {
		return Ok(None);
	};
	let cookie = String::from_utf8(bytes).map_err(|_| {
		error("Повреждена сессия Ely.by. Подключите аккаунт снова.")
	})?;
	let session = new_session()?;
	for pair in cookie.split("; ") {
		session.jar.add_cookie_str(
			&format!("{pair}; Path=/; Secure; HttpOnly"),
			&Url::parse(SITE).unwrap(),
		);
	}
	SESSIONS.lock().await.insert(user, session.clone());
	Ok(Some(session))
}

pub async fn disconnect(user: Uuid) -> Result<()> {
	SESSIONS.lock().await.remove(&user);
	match tokio::fs::remove_file(session_path(user).await?).await {
		Ok(()) => Ok(()),
		Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
		Err(e) => Err(e.into()),
	}
}

async fn verified_session(user: Uuid) -> Result<Option<SiteSession>> {
	let username = expected_username(user).await?;
	let Some(session) = load_session(user).await? else {
		return Ok(None);
	};
	let response = session.client.get(format!("{SITE}/skins")).send().await?;
	if !response.status().is_success()
		|| !matches_user(&current_user(&response.text().await?)?, &username)
	{
		disconnect(user).await?;
		return Ok(None);
	}
	save_session(user, &session).await?;
	Ok(Some(session))
}

pub async fn connected(user: Uuid) -> Result<bool> {
	Ok(verified_session(user).await?.is_some())
}

/// Uses the same account API and authorization-code flow as Ely.by's own frontend, without a WebView.
pub async fn connect(
	user: Uuid,
	login: &str,
	password: &str,
	totp: Option<&str>,
) -> Result<()> {
	let username = expected_username(user).await?;
	let session = new_session()?;
	let response = session
		.client
		.get(format!("{SITE}/authorization/login"))
		.send()
		.await?;
	let location = response
		.headers()
		.get(reqwest::header::LOCATION)
		.and_then(|v| v.to_str().ok())
		.ok_or_else(|| error("Ely.by не предоставил адрес авторизации."))?;
	let location = Url::parse(location)
		.map_err(|_| error("Некорректный адрес авторизации Ely.by."))?;
	if location.scheme() != "https"
		|| location.host_str() != Some("account.ely.by")
		|| location.path() != "/oauth2/v1/ely"
	{
		return Err(error("Изменился протокол авторизации Ely.by."));
	}
	let params: HashMap<_, _> = location.query_pairs().into_owned().collect();
	let redirect = params
		.get("redirect_uri")
		.ok_or_else(|| error("Отсутствует адрес возврата Ely.by."))?;
	if site_url(redirect)?.as_str() != "https://ely.by/authorization/oauth" {
		return Err(error("Неожиданный адрес возврата Ely.by."));
	}
	let state = params
		.get("state")
		.ok_or_else(|| error("Отсутствует state Ely.by."))?;
	let response = session.client.post(format!("{ACCOUNTS}/api/authentication/login"))
		.json(&json!({"login":login.trim(), "password":password, "totp":totp.unwrap_or(""), "rememberMe":true})).send().await?;
	let token = account_response(response).await?;
	let token = token["access_token"]
		.as_str()
		.ok_or_else(|| error("Ely.by не вернул токен авторизации."))?;
	let subject = token_subject(token)?;
	let profile: Value = session
		.client
		.get(format!("{ACCOUNTS}/api/v1/accounts/{subject}"))
		.bearer_auth(token)
		.send()
		.await?
		.error_for_status()?
		.json()
		.await?;
	if profile["uuid"]
		.as_str()
		.and_then(|v| Uuid::parse_str(v).ok())
		!= Some(user)
		|| profile["username"].as_str() != Some(&username)
	{
		return Err(error(
			"Войдите именно в выбранный аккаунт Ely.by, а не в другой профиль.",
		));
	}
	let mut complete =
		Url::parse(&format!("{ACCOUNTS}/api/oauth2/v1/complete")).unwrap();
	complete.query_pairs_mut().extend_pairs([
		("client_id", "ely"),
		("redirect_uri", redirect),
		("response_type", "code"),
		("scope", "account_info account_email"),
		("state", state),
	]);
	let result = account_response(
		session
			.client
			.post(complete)
			.bearer_auth(token)
			.json(&json!({"accept":true}))
			.send()
			.await?,
	)
	.await?;
	let callback = site_url(
		result["redirectUri"]
			.as_str()
			.ok_or_else(|| error("Ely.by не завершил авторизацию сайта."))?,
	)?;
	if callback.path() != "/authorization/oauth"
		|| callback
			.query_pairs()
			.find(|(k, _)| k == "state")
			.map(|(_, v)| v.into_owned())
			!= Some(state.clone())
	{
		return Err(error("Не совпадает состояние авторизации Ely.by."));
	}
	let response = session.client.get(callback).send().await.map_err(|_| {
		error(
			"Не удалось завершить авторизацию сайта Ely.by. Повторите подключение.",
		)
	})?;
	if !response.status().is_redirection() && !response.status().is_success() {
		return Err(error("Не удалось создать сессию сайта Ely.by."));
	}
	let html = session
		.client
		.get(format!("{SITE}/skins"))
		.send()
		.await?
		.error_for_status()?
		.text()
		.await?;
	if !matches_user(&current_user(&html)?, &username) {
		return Err(error("Сайт Ely.by не подтвердил выбранный аккаунт."));
	}
	save_session(user, &session).await?;
	SESSIONS.lock().await.insert(user, session);
	Ok(())
}

fn token_subject(token: &str) -> Result<u64> {
	let body = token
		.split('.')
		.nth(1)
		.and_then(|part| URL_SAFE_NO_PAD.decode(part).ok())
		.ok_or_else(|| error("Некорректный токен Ely.by."))?;
	let claims: Value = serde_json::from_slice(&body)?;
	claims["sub"]
		.as_u64()
		.or_else(|| {
			let subject = claims["sub"].as_str()?;
			subject.strip_prefix("ely|").unwrap_or(subject).parse().ok()
		})
		.ok_or_else(|| {
			error("В токене Ely.by отсутствует идентификатор аккаунта.")
		})
}

async fn account_response(response: Response) -> Result<Value> {
	let status = response.status();
	let value: Value = response.json().await.map_err(|_| {
		error("Ely.by вернул неожиданный ответ. Повторите позже.")
	})?;
	if status.is_success() && value["success"].as_bool() == Some(true) {
		return Ok(value);
	}
	let errors = value["errors"].to_string();
	Err(error(if errors.contains("totp_required") {
		"Введите шестизначный код 2FA."
	} else if errors.contains("totp_incorrect") {
		"Неверный код 2FA. Введите новый код."
	} else if errors.contains("password_incorrect")
		|| errors.contains("login_not_exist")
	{
		"Проверьте логин и пароль Ely.by."
	} else {
		"Ely.by отклонил вход. Проверьте данные, активацию аккаунта и попробуйте позже."
	}))
}

pub async fn catalog(
	page: u32,
	model: &str,
	tags: &str,
) -> Result<CatalogPage> {
	let session = new_session()?;
	let mut url = Url::parse(&format!("{SITE}/api/legacy/skins")).unwrap();
	url.query_pairs_mut()
		.append_pair("page", &page.clamp(1, 250).to_string())
		.append_pair("sort", "time");
	if model == "slim" {
		url.query_pairs_mut().append_pair("type", "slim");
	}
	if !tags.trim().is_empty() {
		url.query_pairs_mut()
			.append_pair("tags[]", &tags.chars().take(100).collect::<String>());
	}
	Ok(session
		.client
		.get(url)
		.send()
		.await?
		.error_for_status()?
		.json()
		.await?)
}

async fn skin_response(response: Response) -> Result<Value> {
	if matches!(response.status().as_u16(), 401 | 403) {
		return Err(error(
			"Сессия управления скинами истекла. Подключите её заново.",
		));
	}
	let value: Value = response.error_for_status()?.json().await?;
	let successful = match value.get("error") {
		None | Some(Value::Null) | Some(Value::Bool(false)) => true,
		Some(Value::String(message)) => {
			message.is_empty() || message.contains("success")
		}
		_ => false,
	};
	if !value.is_object()
		|| value["success"].as_bool() == Some(false)
		|| !successful
		|| value["url"]
			.as_str()
			.is_some_and(|url| url.contains("/authorization/"))
	{
		return Err(error(
			"Ely.by не применил изменение. Сессия могла истечь или файл не прошёл проверку.",
		));
	}
	Ok(value)
}

/// Reads the selected account's saved collection or uploads, never the anonymous global catalog.
pub async fn personal(
	user: Uuid,
	page: u32,
	model: &str,
	tags: &str,
	uploaded: bool,
) -> Result<CatalogPage> {
	let username = expected_username(user).await?;
	let session = verified_session(user).await?.ok_or_else(|| {
		error("Подключите скины, чтобы открыть личную коллекцию Ely.by.")
	})?;
	let mut url = Url::parse(if uploaded {
		"https://ely.by/skins"
	} else {
		"https://ely.by/skins/stored"
	})
	.unwrap();
	url.query_pairs_mut()
		.append_pair("page", &page.clamp(1, 250).to_string())
		.append_pair("sort", "time");
	if model == "slim" {
		url.query_pairs_mut().append_pair("type", "slim");
	}
	if !tags.trim().is_empty() {
		url.query_pairs_mut()
			.append_pair("tags[]", &tags.chars().take(100).collect::<String>());
	}
	if uploaded {
		url.query_pairs_mut().append_pair("uploader", &username);
	}
	let response = session.client.get(url).send().await?;
	if !response.status().is_success() {
		return Err(error(
			"Ely.by не открыл личную коллекцию. Переподключите управление скинами.",
		));
	}
	personal_page(&response.text().await?, &username)
}

fn personal_page(html: &str, username: &str) -> Result<CatalogPage> {
	let user = current_user(html)?;
	if !matches_user(&user, username) {
		return Err(error(
			"Сессия личной коллекции истекла. Подключите скины заново.",
		));
	}
	let value = html
		.split("alight.service.skins = ")
		.nth(1)
		.ok_or_else(|| error("Изменился формат личной коллекции Ely.by."))?;
	let mut page = serde_json::Deserializer::from_str(value)
		.into_iter::<CatalogPage>()
		.next()
		.ok_or_else(|| {
			error("Не удалось прочитать личную коллекцию Ely.by.")
		})??;
	page.equipped_id = user.pointer("/skin/id").and_then(Value::as_u64);
	Ok(page)
}

/// Changes only the skin of the explicitly selected, verified Ely.by account.
pub async fn apply(user: Uuid, skin_id: u64) -> Result<()> {
	if skin_id == 0 {
		return Err(error("Выберите скин из каталога."));
	}
	let session = verified_session(user)
		.await?
		.ok_or_else(|| error("Подключите управление скинами Ely.by."))?;
	skin_response(
		session
			.client
			.put(format!("{SITE}/api/legacy/users/skin"))
			.form(&[("skinId", skin_id.to_string())])
			.send()
			.await?,
	)
	.await?;
	let html = session
		.client
		.get(format!("{SITE}/skins"))
		.send()
		.await?
		.error_for_status()?
		.text()
		.await?;
	if current_user(&html)?
		.pointer("/skin/id")
		.and_then(Value::as_u64)
		!= Some(skin_id)
	{
		return Err(error(
			"Ely.by не подтвердил применение скина. Обновите профиль и повторите попытку.",
		));
	}
	save_session(user, &session).await?;
	Ok(())
}

/// Uploads a PNG to Ely.by without applying it until the user explicitly selects Apply.
pub async fn upload(user: Uuid, bytes: Vec<u8>) -> Result<CatalogSkin> {
	validate_png(&bytes)?;
	let session = verified_session(user)
		.await?
		.ok_or_else(|| error("Подключите управление скинами Ely.by."))?;
	let file = reqwest::multipart::Part::bytes(bytes)
		.file_name("nyanrinth-skin.png")
		.mime_str("image/png")?;
	let value = skin_response(
		session
			.client
			.post(format!("{SITE}/api/legacy/skins"))
			.multipart(reqwest::multipart::Form::new().part("file", file))
			.send()
			.await?,
	)
	.await?;
	let id = value["url"]
		.as_str()
		.and_then(|url| url.split("/skins/s").nth(1))
		.and_then(|id| id.split('/').next())
		.and_then(|id| id.parse::<u64>().ok())
		.or_else(|| value.pointer("/extra/id").and_then(Value::as_u64))
		.ok_or_else(|| {
			error("Файл загружен, но Ely.by не вернул идентификатор скина.")
		})?;
	detail(id).await
}

async fn detail(id: u64) -> Result<CatalogSkin> {
	let session = new_session()?;
	let html = session
		.client
		.get(format!("{SITE}/skins/s{id}"))
		.send()
		.await?
		.error_for_status()?
		.text()
		.await?;
	let json = html
		.split("alight.service.skin = ")
		.nth(1)
		.ok_or_else(|| error("Не удалось получить скин Ely.by."))?;
	serde_json::Deserializer::from_str(json)
		.into_iter::<CatalogSkin>()
		.next()
		.ok_or_else(|| error("Не удалось прочитать скин Ely.by."))?
		.map_err(Into::into)
}

#[derive(Serialize)]
pub struct SkinPreview {
	pub skin: CatalogSkin,
	pub texture: String,
}

pub async fn preview(id: u64) -> Result<SkinPreview> {
	let skin = detail(id).await?;
	let url = site_url(&skin.skin_url)?;
	if !url.path().starts_with("/storage/skins/") {
		return Err(error("Неожиданный адрес текстуры Ely.by."));
	}
	let bytes = new_session()?
		.client
		.get(url)
		.send()
		.await?
		.error_for_status()?
		.bytes()
		.await?;
	validate_png(&bytes)?;
	Ok(SkinPreview {
		skin,
		texture: format!(
			"data:image/png;base64,{}",
			base64::engine::general_purpose::STANDARD.encode(bytes)
		),
	})
}

fn validate_png(bytes: &[u8]) -> Result<()> {
	if bytes.len() > 196608
		|| bytes.len() < 33
		|| &bytes[..8] != b"\x89PNG\r\n\x1a\n"
		|| &bytes[12..16] != b"IHDR"
	{
		return Err(error("Нужен PNG-скин размером до 192 КиБ."));
	}
	let width = u32::from_be_bytes(bytes[16..20].try_into().unwrap());
	let height = u32::from_be_bytes(bytes[20..24].try_into().unwrap());
	if width != 64 || !matches!(height, 32 | 64) {
		return Err(error("Размер скина должен быть 64×64 или 64×32 пикселя."));
	}
	Ok(())
}

#[cfg(windows)]
fn protect(bytes: &[u8], decrypt: bool) -> Result<Vec<u8>> {
	use windows::Win32::{
		Foundation::{HLOCAL, LocalFree},
		Security::Cryptography::{
			CRYPT_INTEGER_BLOB, CRYPTPROTECT_UI_FORBIDDEN, CryptProtectData,
			CryptUnprotectData,
		},
	};
	let input = CRYPT_INTEGER_BLOB {
		cbData: bytes.len() as u32,
		pbData: bytes.as_ptr() as *mut u8,
	};
	let mut output = CRYPT_INTEGER_BLOB::default();
	unsafe {
		let result = if decrypt {
			CryptUnprotectData(
				&input,
				None,
				None,
				None,
				None,
				CRYPTPROTECT_UI_FORBIDDEN,
				&mut output,
			)
		} else {
			CryptProtectData(
				&input,
				windows::core::PCWSTR::null(),
				None,
				None,
				None,
				CRYPTPROTECT_UI_FORBIDDEN,
				&mut output,
			)
		};
		result.map_err(|_| {
			error("Не удалось защитить сессию средствами Windows.")
		})?;
		let value = if output.cbData == 0 {
			Vec::new()
		} else {
			std::slice::from_raw_parts(output.pbData, output.cbData as usize)
				.to_vec()
		};
		let _ = LocalFree(Some(HLOCAL(output.pbData as *mut _)));
		Ok(value)
	}
}

#[cfg(not(windows))]
fn protect(_bytes: &[u8], _decrypt: bool) -> Result<Vec<u8>> {
	Err(error(
		"Защищённое хранение сессии скинов пока поддерживается только в Windows.",
	))
}

#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn personal_collection_preserves_all_uploads_and_equipped_identity() {
		let html = r#"alight.service.currentUser = {"id":42,"nickname":"Player","skin":{"id":101}}; alight.service.skins = {"items":[{"id":100,"skin_url":"https://ely.by/storage/skins/a.png","is_slim":true},{"id":101,"skin_url":"https://ely.by/storage/skins/b.png","is_slim":false},{"id":102,"skin_url":"https://ely.by/storage/skins/c.png","is_slim":true}],"total_items":3,"current":1,"last":1};"#;
		let page = personal_page(html, "Player").unwrap();
		assert_eq!(page.total_items, 3);
		assert_eq!(
			page.items.iter().map(|skin| skin.id).collect::<Vec<_>>(),
			[100, 101, 102]
		);
		assert_eq!(page.equipped_id, Some(101));
	}
	#[test]
	fn personal_collection_rejects_anonymous_global_fallback() {
		let html = r#"alight.service.currentUser = {"id":0,"nickname":"Guest"}; alight.service.skins = {"items":[],"current":1,"last":1};"#;
		assert!(personal_page(html, "Player").is_err());
	}
	#[test]
	fn personal_collection_accepts_only_matching_owner() {
		let html = r#"alight.service.currentUser = {"id":42,"nickname":"Player"}; alight.service.skins = {"items":[],"current":1,"last":1};"#;
		assert_eq!(personal_page(html, "Player").unwrap().items.len(), 0);
		assert!(personal_page(html, "AnotherPlayer").is_err());
	}
	#[test]
	fn account_token_subject_follows_ely_tokens_factory() {
		let token = format!(
			"header.{}.signature",
			URL_SAFE_NO_PAD.encode(br#"{"sub":"ely|42"}"#)
		);
		assert_eq!(token_subject(&token).unwrap(), 42);
		let wrong = format!(
			"header.{}.signature",
			URL_SAFE_NO_PAD.encode(br#"{"sub":"other|42"}"#)
		);
		assert!(token_subject(&wrong).is_err());
	}
	#[test]
	fn callback_is_restricted_to_ely_https() {
		assert!(
			site_url("https://ely.by/authorization/oauth?code=test").is_ok()
		);
		for url in [
			"https://ely.by.attacker.test/",
			"http://ely.by/",
			"https://ely.by:4433/",
			"https://x@ely.by/",
		] {
			assert!(site_url(url).is_err());
		}
	}
	#[test]
	fn current_user_parsing_does_not_execute_site_javascript() {
		let user = current_user(
			"alight.service.currentUser = {\"id\":1,\"nickname\":\"Player\"}; evil();",
		)
		.unwrap();
		assert!(matches_user(&user, "Player"));
		assert!(!matches_user(&user, "AnotherPlayer"));
		assert!(!matches_user(
			&json!({"id":0,"nickname":"Player"}),
			"Player"
		));
	}
	#[test]
	fn malformed_and_wrong_size_uploads_are_rejected() {
		assert!(validate_png(b"fake.png").is_err());
		let mut png = vec![0; 33];
		png[..8].copy_from_slice(b"\x89PNG\r\n\x1a\n");
		png[12..16].copy_from_slice(b"IHDR");
		png[16..20].copy_from_slice(&64u32.to_be_bytes());
		png[20..24].copy_from_slice(&64u32.to_be_bytes());
		assert!(validate_png(&png).is_ok());
		png[16..20].copy_from_slice(&128u32.to_be_bytes());
		assert!(validate_png(&png).is_err());
	}
	#[test]
	#[cfg(windows)]
	fn windows_session_protection_roundtrips_and_rejects_corruption() {
		let encrypted = protect(b"cookie=private", false).unwrap();
		assert_ne!(encrypted, b"cookie=private");
		assert_eq!(protect(&encrypted, true).unwrap(), b"cookie=private");
		assert!(protect(b"corrupted", true).is_err());
		assert_eq!(protect(&protect(b"", false).unwrap(), true).unwrap(), b"");
	}
}
