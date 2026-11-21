use crate::{Result, State};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use reqwest::{
    Client, Response,
    cookie::{CookieStore, Jar},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{HashMap, HashSet},
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
static UPLOADS_LOCK: Mutex<()> = Mutex::const_new(());

#[derive(Clone)]
struct SiteSession {
    client: Client,
    jar: Arc<Jar>,
}

#[derive(Clone, Serialize, Deserialize)]
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
	let response = session
		.client
		.get(format!("{SITE}/skins"))
		.send()
		.await?
		.error_for_status()?;
	if !matches_user(&current_user(&response.text().await?)?, &username) {
		tracing::debug!("Ely.by skin session is no longer authenticated");
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
) -> Result<Option<CatalogPage>> {
    let username = expected_username(user).await?;
	let Some(session) = verified_session(user).await? else {
		tracing::debug!("Ely.by personal collection requested without an active skin session");
		return Ok(None);
	};
    let (model, tags) = if uploaded { ("", "") } else { (model, tags) };
    let mut result =
        personal_remote(&session, &username, page, model, tags, uploaded)
            .await?;
    if uploaded {
        let state = State::get().await?;
        let pending = pending_uploads(&state.pool, user).await?;
        if !pending.is_empty() {
            let mut remote = Vec::new();
            for remote_page in 1..=result.last {
                if remote_page == result.current {
                    remote.extend(result.items.iter().cloned());
                } else {
                    remote.extend(
                        personal_remote(
                            &session,
                            &username,
                            remote_page,
                            model,
                            tags,
                            true,
                        )
                        .await?
                        .items,
                    );
                }
            }
            let remote_ids: HashSet<_> =
                remote.iter().map(|skin| skin.id).collect();
            let _lock = UPLOADS_LOCK.lock().await;
            let pending: Vec<_> = pending_uploads(&state.pool, user)
                .await?
                .into_iter()
                .filter(|skin| !remote_ids.contains(&skin.id))
                .collect();
            store_pending_uploads(&state.pool, user, &pending).await?;
            result = uploaded_page(remote, pending, page, result.equipped_id);
        }
    }
    Ok(Some(result))
}

async fn personal_remote(
    session: &SiteSession,
    username: &str,
    page: u32,
    model: &str,
    tags: &str,
    uploaded: bool,
) -> Result<CatalogPage> {
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
        url.query_pairs_mut().append_pair("uploader", username);
    }
    let response = session.client.get(url).send().await?;
    if !response.status().is_success() {
        return Err(error(
            "Ely.by не открыл личную коллекцию. Переподключите управление скинами.",
        ));
    }
    personal_page(&response.text().await?, username)
}

async fn pending_uploads(
    pool: &sqlx::SqlitePool,
    user: Uuid,
) -> Result<Vec<CatalogSkin>> {
    let value: Option<String> =
        sqlx::query_scalar("SELECT value FROM app_metadata WHERE key = ?")
            .bind(format!("ely.pending-uploads:{user}"))
            .fetch_optional(pool)
            .await?;
    Ok(value
        .map(|value| serde_json::from_str(&value))
        .transpose()?
        .unwrap_or_default())
}

async fn store_pending_uploads(
    pool: &sqlx::SqlitePool,
    user: Uuid,
    skins: &[CatalogSkin],
) -> Result<()> {
    sqlx::query("INSERT INTO app_metadata(key, value) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = unixepoch()")
		.bind(format!("ely.pending-uploads:{user}"))
		.bind(serde_json::to_string(skins)?)
		.execute(pool).await?;
    Ok(())
}

fn uploaded_page(
    remote: Vec<CatalogSkin>,
    pending: Vec<CatalogSkin>,
    page: u32,
    equipped_id: Option<u64>,
) -> CatalogPage {
    let mut seen = HashSet::new();
    let items: Vec<_> = pending
        .into_iter()
        .chain(remote)
        .filter(|skin| seen.insert(skin.id))
        .collect();
    let total_items = items.len() as u32;
    let last = total_items.div_ceil(40).max(1);
    let current = page.clamp(1, last);
    CatalogPage {
        items: items
            .into_iter()
            .skip(((current - 1) * 40) as usize)
            .take(40)
            .collect(),
        current,
        last,
        total_items,
        equipped_id,
    }
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

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SkinRemoval {
    ElyBy,
    LocalOnly,
    Saved,
}

/// Saves a skin without changing the account's equipped skin.
pub async fn save(user: Uuid, skin_id: u64) -> Result<()> {
    if skin_id == 0 {
        return Err(error("Выберите скин из каталога."));
    }
    let session = verified_session(user)
        .await?
        .ok_or_else(|| error("Подключите управление скинами Ely.by."))?;
    let html = skin_page(&session, skin_id).await?;
    let already_saved = saved_skin_state(&html)?;
    tracing::debug!(
        skin_id,
        already_saved,
        "Ely.by favorite state before saving"
    );
    if already_saved {
        return Ok(());
    }
    skin_response(
        session
            .client
            .put(format!("{SITE}/api/legacy/cubes/skins/{skin_id}"))
            .send()
            .await?,
    )
    .await?;
    let html = skin_page(&session, skin_id).await?;
    let saved = saved_skin_state(&html)?;
    tracing::debug!(skin_id, saved, "Ely.by favorite state after saving");
    if !saved {
        return Err(error(
            "Ely.by не подтвердил сохранение скина. Обновите коллекцию и повторите попытку.",
        ));
    }
    save_session(user, &session).await?;
    Ok(())
}

fn saved_skin_state(html: &str) -> Result<bool> {
    static STATE: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(
            r#"al-init\s*=\s*["']\s*skin\.is_cubed\s*=\s*([01])\s*;?\s*["']"#,
        )
        .unwrap()
    });
    if let Some(state) = STATE.captures(html) {
        return Ok(&state[1] == "1");
    }
    if let Some(saved) =
        skin_data(html)?.get("is_cubed").and_then(Value::as_bool)
    {
        return Ok(saved);
    }
    tracing::warn!(
        "Ely.by favorite state missing from skin page; refusing to toggle"
    );
    Err(error(
        "Не удалось определить, сохранён ли скин на Ely.by. Обновите каталог и повторите попытку.",
    ))
}

/// Removes an owned upload, a stale local entry, or a skin from the saved collection.
pub async fn remove(
    user: Uuid,
    skin_id: u64,
    uploaded: bool,
) -> Result<SkinRemoval> {
    if skin_id == 0 {
        return Err(error("Выберите скин из личной коллекции."));
    }
    let username = expected_username(user).await?;
    let session = verified_session(user)
        .await?
        .ok_or_else(|| error("Подключите управление скинами Ely.by."))?;
    let removal = if uploaded {
        let response = session
            .client
            .get(format!("{SITE}/skins/s{skin_id}"))
            .send()
            .await?;
        let details = optional_skin_page(response)
            .await?
            .map(|html| -> Result<_> {
                Ok((current_user(&html)?, skin_data(&html)?))
            })
            .transpose()?;
        let owned = match &details {
            Some((owner, skin)) => {
                is_owned_skin(owner, skin, &username, skin_id)?
            }
            None => false,
        };
        let state = State::get().await?;
        if let Some((owner, skin)) = details.filter(|_| owned) {
            validate_removal(&owner, &skin, &username, skin_id)?;
            skin_response(
                session
                    .client
                    .delete(format!("{SITE}/api/legacy/skins/{skin_id}"))
                    .send()
                    .await?,
            )
            .await?;
        }
        let _lock = UPLOADS_LOCK.lock().await;
        let mut pending = pending_uploads(&state.pool, user).await?;
        if !owned && !pending.iter().any(|skin| skin.id == skin_id) {
            return Err(error("Можно удалить только свои загрузки Ely.by."));
        }
        pending.retain(|skin| skin.id != skin_id);
        store_pending_uploads(&state.pool, user, &pending).await?;
        if owned {
            SkinRemoval::ElyBy
        } else {
            SkinRemoval::LocalOnly
        }
    } else {
        let mut page =
            personal_remote(&session, &username, 1, "", "", false).await?;
        loop {
            if page.items.iter().any(|skin| skin.id == skin_id) {
                break;
            }
            if page.current >= page.last {
                return Ok(SkinRemoval::Saved);
            }
            page = personal_remote(
                &session,
                &username,
                page.current + 1,
                "",
                "",
                false,
            )
            .await?;
        }
        skin_response(
            session
                .client
                .put(format!("{SITE}/api/legacy/cubes/skins/{skin_id}"))
                .send()
                .await?,
        )
        .await?;
        SkinRemoval::Saved
    };
    save_session(user, &session).await?;
    Ok(removal)
}

fn is_owned_skin(
    owner: &Value,
    skin: &Value,
    username: &str,
    skin_id: u64,
) -> Result<bool> {
    if !matches_user(owner, username) || skin["id"].as_u64() != Some(skin_id) {
        return Err(error("Не удалось подтвердить аккаунт или скин Ely.by."));
    }
    Ok(skin.pointer("/uploader/id").and_then(Value::as_u64)
        == owner["id"].as_u64())
}

fn validate_removal(
    owner: &Value,
    skin: &Value,
    username: &str,
    skin_id: u64,
) -> Result<()> {
    if !is_owned_skin(owner, skin, username, skin_id)? {
        return Err(error("Можно удалить только свои загрузки Ely.by."));
    }
    if owner.pointer("/skin/id").and_then(Value::as_u64) == Some(skin_id) {
        return Err(error("Перед удалением примените другой скин."));
    }
    Ok(())
}

/// Uploads a PNG to Ely.by without applying it until the user explicitly selects Apply.
pub async fn upload(
    user: Uuid,
    bytes: Vec<u8>,
    slim: bool,
) -> Result<CatalogSkin> {
    validate_upload(&bytes, slim)?;
    let username = expected_username(user).await?;
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
            .multipart(
                reqwest::multipart::Form::new()
                    .part("file", file)
                    .text("isSlim", if slim { "1" } else { "0" }),
            )
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
    let html = skin_page(&session, id).await?;
    let owner = current_user(&html)?;
    let mut skin = skin_data(&html)?;
    if !is_owned_skin(&owner, &skin, &username, id)? {
        return Err(error(
            "Ely.by вернул скин другого автора. Он не добавлен в «Мои загрузки». Его можно найти в каталоге Ely.by.",
        ));
    }
    if skin["is_slim"].as_bool() != Some(slim) {
        skin_response(
            session
                .client
                .put(format!("{SITE}/api/legacy/skins/{id}"))
                .form(&skin_model_form(&skin, slim))
                .send()
                .await?,
        )
        .await
        .map_err(|err| {
            error(&format!(
                "Скин загружен в Ely.by, но тип модели не сохранён: {err}"
            ))
        })?;
        let html = skin_page(&session, id).await?;
        let owner = current_user(&html)?;
        skin = skin_data(&html)?;
        if !is_owned_skin(&owner, &skin, &username, id)?
            || skin["is_slim"].as_bool() != Some(slim)
        {
            return Err(error(
                "Скин загружен в Ely.by, но сайт не подтвердил выбранный тип Steve/Alex. Обновите «Мои загрузки» и проверьте скин.",
            ));
        }
    }
    let skin: CatalogSkin = serde_json::from_value(skin)?;
    let state = State::get().await?;
    let _lock = UPLOADS_LOCK.lock().await;
    let mut pending = pending_uploads(&state.pool, user).await?;
    pending.retain(|existing| existing.id != skin.id);
    pending.insert(0, skin.clone());
    store_pending_uploads(&state.pool, user, &pending).await?;
    Ok(skin)
}

fn skin_model_form(skin: &Value, slim: bool) -> Vec<(String, String)> {
    let mut form =
        vec![("isSlim".to_owned(), if slim { "1" } else { "0" }.to_owned())];
    for key in ["name", "description", "kind", "color"] {
        if let Some(value) = skin.get(key) {
            if let Some(value) = value.as_str() {
                form.push((key.to_owned(), value.to_owned()));
            } else if value.is_number() {
                form.push((key.to_owned(), value.to_string()));
            }
        }
    }
    if let Some(tags) = skin["tags"].as_array() {
        form.push((
            "tags".to_owned(),
            tags.iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(","),
        ));
    }
    form
}

fn validate_upload(bytes: &[u8], slim: bool) -> Result<()> {
    validate_png(bytes)?;
    if slim && u32::from_be_bytes(bytes[20..24].try_into().unwrap()) != 64 {
        return Err(error(
            "Для модели Alex нужен PNG 64×64. Формат 64×32 поддерживает только Steve.",
        ));
    }
    Ok(())
}

async fn skin_page(session: &SiteSession, id: u64) -> Result<String> {
    Ok(session
        .client
        .get(format!("{SITE}/skins/s{id}"))
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?)
}

async fn optional_skin_page(
    response: reqwest::Response,
) -> Result<Option<String>> {
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }
    Ok(Some(response.error_for_status()?.text().await?))
}

fn skin_data(html: &str) -> Result<Value> {
    let json = html
        .split("alight.service.skin = ")
        .nth(1)
        .ok_or_else(|| error("Не удалось получить скин Ely.by."))?;
    serde_json::Deserializer::from_str(json)
        .into_iter::<Value>()
        .next()
        .ok_or_else(|| error("Не удалось прочитать скин Ely.by."))?
        .map_err(Into::into)
}

async fn detail(id: u64) -> Result<CatalogSkin> {
    let html = skin_page(&new_session()?, id).await?;
    Ok(serde_json::from_value(skin_data(&html)?)?)
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
    fn favorite_state_reads_ely_html_initializer() {
        for (value, expected) in [("0", false), ("1", true)] {
            let html = format!(
                r#"<div id="user-profile-container" al-init="skin.is_cubed = {value}">
                <script>alight.service.skin = {{"id": 42}};</script>"#
            );
            assert_eq!(saved_skin_state(&html).unwrap(), expected);
        }
        assert!(
            saved_skin_state(r#"<div al-init=' skin.is_cubed = 1; '></div>"#)
                .unwrap()
        );
    }

    #[test]
    fn favorite_state_rejects_unknown_state_instead_of_toggling() {
        for value in
            [json!({}), json!({"is_cubed": null}), json!({"is_cubed": 2})]
        {
            assert!(
                saved_skin_state(&format!("alight.service.skin = {value};"))
                    .is_err()
            );
        }
        for saved in [true, false] {
            let html =
                format!("alight.service.skin = {{\"is_cubed\": {saved}}};");
            assert_eq!(saved_skin_state(&html).unwrap(), saved);
        }
    }

    #[tokio::test]
    async fn missing_skin_is_distinct_from_session_and_server_errors() {
        use std::io::{Read, Write};
        crate::install_crypto_provider();
        for status in [200, 404, 401, 403, 500] {
            let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            let address = listener.local_addr().unwrap();
            let server = std::thread::spawn(move || {
                let (mut stream, _) = listener.accept().unwrap();
                stream
                    .set_read_timeout(Some(std::time::Duration::from_secs(5)))
                    .unwrap();
                let mut request = [0; 4096];
                assert!(stream.read(&mut request).unwrap() > 0);
                write!(stream, "HTTP/1.1 {status} Test\r\nContent-Length: 4\r\nConnection: close\r\n\r\nskin").unwrap();
            });
            let response = reqwest::Client::builder()
                .no_proxy()
                .timeout(std::time::Duration::from_secs(5))
                .build()
                .unwrap()
                .get(format!("http://{address}"))
                .send()
                .await
                .unwrap();
            let result = optional_skin_page(response).await;
            server.join().unwrap();
            match status {
                200 => assert_eq!(result.unwrap().as_deref(), Some("skin")),
                404 => assert!(result.unwrap().is_none()),
                _ => assert!(result.is_err()),
            }
        }
    }
    #[test]
    fn deletion_requires_matching_owner_and_an_unequipped_skin() {
        let owner =
            json!({"id": 42, "nickname": "Player", "skin": {"id": 100}});
        let skin = json!({"id": 101, "uploader": {"id": 42}});
        assert!(
            !is_owned_skin(
                &owner,
                &json!({"id": 101, "uploader": {"id": 7}}),
                "Player",
                101
            )
            .unwrap()
        );
        assert!(
            !is_owned_skin(&owner, &json!({"id": 101}), "Player", 101).unwrap()
        );
        assert!(validate_removal(&owner, &skin, "Player", 101).is_ok());
        assert!(validate_removal(&owner, &skin, "OtherPlayer", 101).is_err());
        assert!(validate_removal(&owner, &skin, "Player", 102).is_err());
        assert!(
            validate_removal(
                &owner,
                &json!({"id": 101, "uploader": {"id": 7}}),
                "Player",
                101
            )
            .is_err()
        );
        assert!(
            validate_removal(
                &owner,
                &json!({"id": 100, "uploader": {"id": 42}}),
                "Player",
                100
            )
            .is_err()
        );
        assert!(
            validate_removal(
                &json!({"id": 0, "nickname": "Guest"}),
                &skin,
                "Guest",
                101
            )
            .is_err()
        );
    }

    fn skin(id: u64) -> CatalogSkin {
        CatalogSkin {
            id,
            skin_url: format!("https://ely.by/storage/skins/{id}.png"),
            is_slim: false,
            tags: Vec::new(),
        }
    }

    #[tokio::test]
    async fn successful_upload_is_available_before_remote_index_and_is_account_scoped()
     {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query(include_str!(
            "../../migrations/20260623120000_app-metadata.sql"
        ))
        .execute(&pool)
        .await
        .unwrap();
        let owner = Uuid::new_v4();
        store_pending_uploads(&pool, owner, &[skin(103)])
            .await
            .unwrap();
        assert!(
            pending_uploads(&pool, Uuid::new_v4())
                .await
                .unwrap()
                .is_empty()
        );
        let pending = pending_uploads(&pool, owner).await.unwrap();
        let page =
            uploaded_page(vec![skin(102), skin(101)], pending, 1, Some(101));
        assert_eq!(
            page.items.iter().map(|skin| skin.id).collect::<Vec<_>>(),
            [103, 102, 101]
        );
        assert_eq!(page.total_items, 3);
        assert_eq!(page.equipped_id, Some(101));
    }

    #[test]
    fn uploads_are_deduplicated_when_indexed_and_paginate_without_losing_skins()
    {
        let remote = || (1..=41).rev().map(skin).collect();
        let first = uploaded_page(remote(), vec![skin(42), skin(41)], 1, None);
        let second = uploaded_page(remote(), vec![skin(42), skin(41)], 2, None);
        assert_eq!(first.total_items, 42);
        assert_eq!(first.last, 2);
        assert_eq!(first.items.len(), 40);
        assert_eq!(
            second.items.iter().map(|skin| skin.id).collect::<Vec<_>>(),
            [2, 1]
        );
        assert_eq!(first.items[0].id, 42);
        assert_eq!(first.items[1].id, 41);
    }
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
        assert!(validate_upload(b"fake.png", true).is_err());
        let mut png = vec![0; 33];
        png[..8].copy_from_slice(b"\x89PNG\r\n\x1a\n");
        png[12..16].copy_from_slice(b"IHDR");
        png[16..20].copy_from_slice(&64u32.to_be_bytes());
        png[20..24].copy_from_slice(&64u32.to_be_bytes());
        assert!(validate_png(&png).is_ok());
        assert!(validate_upload(&png, true).is_ok());
        assert!(validate_upload(&png, false).is_ok());
        png[20..24].copy_from_slice(&32u32.to_be_bytes());
        assert!(validate_upload(&png, false).is_ok());
        assert!(validate_upload(&png, true).is_err());
        png[16..20].copy_from_slice(&128u32.to_be_bytes());
        assert!(validate_png(&png).is_err());
    }
    #[test]
    fn changing_model_preserves_existing_skin_metadata() {
        let skin = json!({"is_slim": false, "name": "Test skin", "description": "Описание", "kind": 2, "color": "ff00aa", "tags": ["Персонаж", "custom"]});
        for slim in [false, true] {
            let form = skin_model_form(&skin, slim);
            assert!(form.contains(&(
                "isSlim".into(),
                if slim { "1" } else { "0" }.into()
            )));
            assert!(form.contains(&("kind".into(), "2".into())));
            assert!(form.contains(&("color".into(), "ff00aa".into())));
            assert!(form.contains(&("name".into(), "Test skin".into())));
            assert!(form.contains(&("description".into(), "Описание".into())));
            assert!(form.contains(&("tags".into(), "Персонаж,custom".into())));
            assert!(
                !form
                    .iter()
                    .any(|(key, _)| key == "is_slim" || key == "tags[]")
            );
        }
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
