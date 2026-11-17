use anyhow::{Context, bail};
use sqlx::{
    AssertSqlSafe, Connection, Row, SqliteConnection,
    sqlite::SqliteConnectOptions,
};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

const CONVERTED: &str = "app-modrinth-v0221.db";
const COMMIT: &str = "b3a202687e9a33e5f5da676ae7b62181a2011702";

/// Converts divergent NyanRinth schemas into a separately migrated database.
/// The source, its SQLx ledger and the WAL-consistent snapshot are retained.
pub(super) async fn select_database(
    directory: &Path,
) -> crate::Result<PathBuf> {
    let directory = directory.to_path_buf();
    tokio::task::spawn_blocking(move || {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?
            .block_on(select_database_inner(&directory))
    })
    .await
    .context("database converter worker failed")?
}

async fn select_database_inner(directory: &Path) -> crate::Result<PathBuf> {
    let converted = directory.join(CONVERTED);
    if converted.try_exists()? {
        let mut conn = read_only(&converted).await?;
        let marker: Option<String> = sqlx::query_scalar(
			"SELECT value FROM app_metadata WHERE key='nyanrinth_upstream_commit'",
		)
		.fetch_optional(&mut conn)
		.await?;
        if marker.as_deref() != Some(COMMIT) {
            bail!(
                "Незавершённая миграция NyanRinth: требуется восстановить копию базы."
            );
        }
        return Ok(converted);
    }
    let source = directory.join("app.db");
    if !source.try_exists()? {
        return Ok(source);
    }
    let mut conn = read_only(&source).await?;
    let has_ledger: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM sqlite_master WHERE name='_sqlx_migrations'",
    )
    .fetch_one(&mut conn)
    .await?;
    if has_ledger == 0 {
        return Ok(source);
    }
    let external: i64 = sqlx::query_scalar("SELECT count(*) FROM _sqlx_migrations WHERE version=20250716135844 AND success=1").fetch_one(&mut conn).await?;
    let official: i64 = sqlx::query_scalar("SELECT count(*) FROM _sqlx_migrations WHERE version=20260819120000 AND success=1").fetch_one(&mut conn).await?;
    if external == 0 || official > 0 {
        return Ok(source);
    }
    let instances: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM sqlite_master WHERE name='instances'",
    )
    .fetch_one(&mut conn)
    .await?;
    if instances == 0 {
        bail!(
            "Эта база NyanRinth требует промежуточного обновления до 1.1.0 перед переносом."
        );
    }
    let snapshot = directory
        .join(format!("app-before-modrinth-{}.db", uuid::Uuid::new_v4()));
    sqlx::query("VACUUM INTO ?")
        .bind(snapshot.to_str().context("invalid snapshot path")?)
        .execute(&mut conn)
        .await?;
    conn.close().await?;
    let temporary =
        directory.join(format!(".upgrade-{}.db", uuid::Uuid::new_v4()));
    convert_snapshot(&snapshot, &temporary).await?;
    tokio::fs::rename(&temporary, &converted).await?;
    Ok(converted)
}

async fn read_only(path: &Path) -> crate::Result<SqliteConnection> {
    Ok(SqliteConnection::connect_with(
        &SqliteConnectOptions::new().filename(path).read_only(true),
    )
    .await?)
}

fn quote(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}

/// Schema names are fixed by callers; identifiers are SQLite-quoted and checked against the destination schema.
async fn columns(
    conn: &mut SqliteConnection,
    schema: &str,
    table: &str,
) -> crate::Result<Vec<String>> {
    Ok(sqlx::query(AssertSqlSafe(format!(
        "PRAGMA {schema}.table_info({})",
        quote(table)
    )))
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(|row| row.get("name"))
    .collect())
}

async fn convert_snapshot(
    snapshot: &Path,
    destination: &Path,
) -> crate::Result<()> {
    let mut conn = SqliteConnection::connect_with(
        &SqliteConnectOptions::new()
            .filename(destination)
            .create_if_missing(true)
            .foreign_keys(false),
    )
    .await?;
    sqlx::migrate!().run(&mut conn).await?;
    sqlx::query("ATTACH DATABASE ? AS legacy")
        .bind(snapshot.to_str().context("invalid legacy path")?)
        .execute(&mut conn)
        .await?;
    let source_tables: Vec<String> = sqlx::query_scalar("SELECT name FROM legacy.sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' AND name != '_sqlx_migrations'").fetch_all(&mut conn).await?;
    let target_tables: HashSet<String> = sqlx::query_scalar::<_, String>(
        "SELECT name FROM main.sqlite_master WHERE type='table'",
    )
    .fetch_all(&mut conn)
    .await?
    .into_iter()
    .collect();
    let mut tx = conn.begin().await?;
    for table in source_tables {
        if !target_tables.contains(&table)
            || table.starts_with("synced_")
            || table.starts_with("instance_sync_")
            || table.starts_with("instance_game_option_")
            || table.starts_with("instance_synced_")
            || table == "sync_feature_settings"
        {
            continue;
        }
        let source_columns = columns(&mut tx, "legacy", &table).await?;
        let target_columns = columns(&mut tx, "main", &table).await?;
        let missing: Vec<_> = source_columns
            .iter()
            .filter(|name| !target_columns.contains(name))
            .collect();
        if !missing.is_empty() {
            bail!("Неподдерживаемая схема таблицы {table}: {missing:?}");
        }
        let names = source_columns
            .iter()
            .map(|name| quote(name))
            .collect::<Vec<_>>()
            .join(",");
        sqlx::query(AssertSqlSafe(format!(
            "DELETE FROM main.{}",
            quote(&table)
        )))
        .execute(&mut *tx)
        .await?;
        sqlx::query(AssertSqlSafe(format!(
            "INSERT INTO main.{} ({names}) SELECT {names} FROM legacy.{}",
            quote(&table),
            quote(&table)
        )))
        .execute(&mut *tx)
        .await
        .with_context(|| format!("converting table {table}"))?;
        let original: i64 = sqlx::query_scalar(AssertSqlSafe(format!(
            "SELECT count(*) FROM legacy.{}",
            quote(&table)
        )))
        .fetch_one(&mut *tx)
        .await?;
        let copied: i64 = sqlx::query_scalar(AssertSqlSafe(format!(
            "SELECT count(*) FROM main.{}",
            quote(&table)
        )))
        .fetch_one(&mut *tx)
        .await?;
        if original != copied {
            bail!("Количество строк изменилось при переносе {table}");
        }
    }
    for script in [
        include_str!(
            "../../migrations/20260902130000_promote-instance-navigation-settings.sql"
        ),
        include_str!(
            "../../migrations/20260918120000_promote-behavior-settings.sql"
        ),
    ] {
        if let Some(start) = script.find("UPDATE settings") {
            sqlx::raw_sql(&script[start..]).execute(&mut *tx).await?;
        }
    }
    migrate_sync(&mut tx).await?;
    sqlx::query("INSERT INTO app_metadata(key,value,updated_at) VALUES('nyanrinth_upstream_commit',?,unixepoch()) ON CONFLICT(key) DO UPDATE SET value=excluded.value").bind(COMMIT).execute(&mut *tx).await?;
    let violations = sqlx::query("PRAGMA main.foreign_key_check")
        .fetch_all(&mut *tx)
        .await?;
    if !violations.is_empty() {
        bail!("Нарушены связи SQLite после переноса: {}", violations.len());
    }
    tx.commit().await?;
    let integrity: String = sqlx::query_scalar("PRAGMA integrity_check")
        .fetch_one(&mut conn)
        .await?;
    if integrity != "ok" {
        bail!("Проверка новой базы: {integrity}");
    }
    conn.close().await?;
    Ok(())
}

async fn migrate_sync(conn: &mut SqliteConnection) -> crate::Result<()> {
    sqlx::query("UPDATE sync_feature_settings SET globally_enabled=0, new_instance_default=0 WHERE feature!='screenshots'").execute(&mut *conn).await?;
    sqlx::query("INSERT OR IGNORE INTO instance_sync_preferences(instance_id,feature,enabled) SELECT i.id,f.feature,0 FROM instances i CROSS JOIN sync_feature_settings f").execute(&mut *conn).await?;
    let exists: i64 = sqlx::query_scalar("SELECT count(*) FROM legacy.sqlite_master WHERE name='nyan_game_sync_config'").fetch_one(&mut *conn).await?;
    if exists == 0 {
        return Ok(());
    }
    let payload: Option<String> = sqlx::query_scalar(
        "SELECT payload FROM legacy.nyan_game_sync_config WHERE id=1",
    )
    .fetch_optional(&mut *conn)
    .await?;
    let Some(payload) = payload else {
        return Ok(());
    };
    let config: serde_json::Value = serde_json::from_str(&payload)?;
    let enabled = config
        .get("enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    sqlx::query("UPDATE sync_feature_settings SET globally_enabled=?,new_instance_default=1 WHERE feature='game_options'").bind(enabled).execute(&mut *conn).await?;
    let excluded: Vec<String> = serde_json::from_value(
        config
            .get("excluded_instances")
            .cloned()
            .unwrap_or_else(|| serde_json::json!([])),
    )?;
    let instances: Vec<String> = sqlx::query_scalar("SELECT id FROM instances")
        .fetch_all(&mut *conn)
        .await?;
    for id in instances {
        sqlx::query("UPDATE instance_sync_preferences SET enabled=? WHERE instance_id=? AND feature='game_options'").bind(!excluded.contains(&id)).bind(id).execute(&mut *conn).await?;
    }
    let disabled: Vec<String> = serde_json::from_value(
        config
            .get("disabled_options")
            .cloned()
            .unwrap_or_else(|| serde_json::json!([])),
    )?;
    let ids = crate::instance::migration_option_ids();
    for id in &ids {
        sqlx::query("INSERT OR REPLACE INTO synced_game_option_preferences(option_id,enabled,source,revision) VALUES(?,?,'user',0)").bind(id).bind(!disabled.iter().any(|disabled|disabled==id)).execute(&mut *conn).await?;
    }
    let mut seeded = false;
    if let Some(values) = config.get("values").and_then(|v| v.as_object()) {
        for (id, value) in values {
            if !ids.contains(&id.as_str()) {
                continue;
            }
            let canonical: crate::state::CanonicalValue =
                serde_json::from_value(value.clone())?;
            sqlx::query("INSERT OR REPLACE INTO synced_game_option_values(option_id,kind,canonical_type,canonical_value_json,value_codec,seeded,revision,origin,source_instance_id,updated_at) VALUES(?,'vanilla',?,?,'catalog',1,1,'source_seed',?,unixepoch())").bind(id).bind(canonical.type_name()).bind(value.to_string()).bind(config.get("source_instance").and_then(|v|v.as_str())).execute(&mut *conn).await?;
            seeded = true;
        }
    }
    if seeded {
        sqlx::query("INSERT OR REPLACE INTO synced_game_option_state(singleton,revision,catalog_revision) VALUES(1,1,1)").execute(conn).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn fixture(
        path: &Path,
        enabled: bool,
    ) -> crate::Result<SqliteConnection> {
        let mut conn = SqliteConnection::connect_with(
            &SqliteConnectOptions::new()
                .filename(path)
                .create_if_missing(true)
                .foreign_keys(false)
                .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal),
        )
        .await?;
        sqlx::raw_sql(include_str!("../../tests/fixtures/nyanrinth-1.1.0.sql"))
            .execute(&mut conn)
            .await?;
        sqlx::query("CREATE TABLE _sqlx_migrations(version BIGINT PRIMARY KEY, success BOOLEAN NOT NULL)").execute(&mut conn).await?;
        sqlx::query("INSERT INTO _sqlx_migrations VALUES(20250716135844,1)")
            .execute(&mut conn)
            .await?;
        sqlx::query("INSERT INTO minecraft_users(uuid,active,username,access_token,refresh_token,expires,account_type) VALUES('00000000-0000-0000-0000-000000000001',1,'NyanQA','test-access','test-refresh',2000000000,'elyby')").execute(&mut conn).await?;
        sqlx::raw_sql("INSERT INTO instances(id,path,applied_content_set_id,install_stage,launcher_feature_version,name,created,modified) VALUES('local:fixture','Тестовая сборка','set-a','installed','1','NyanFixture',1,1); INSERT INTO instance_content_sets(id,instance_id,name,source_kind,status,game_version,loader,created,modified) VALUES('set-a','local:fixture','Current','local','available','1.21.4','vanilla',1,1); INSERT INTO instance_groups(id,name) VALUES('group-a','Избранное'); INSERT INTO instance_launch_overrides(instance_id,overrides) VALUES('local:fixture','{\"memory\":{\"maximum\":6144,\"minimum\":512}}');").execute(&mut conn).await?;
        let config = serde_json::json!({"enabled":enabled,"excluded_instances":["local:fixture"],"disabled_options":["chat_scale"],"values":{"chat_scale":{"type":"decimal","value":"0.75"},"unknown_option":{"type":"text","value":"local"}}});
        sqlx::query(
            "INSERT INTO nyan_game_sync_config(id,payload) VALUES(1,?)",
        )
        .bind(config.to_string())
        .execute(&mut conn)
        .await?;
        Ok(conn)
    }

    #[tokio::test]
    async fn upgrade_preserves_wal_accounts_and_source_ledger()
    -> crate::Result<()> {
        for enabled in [false, true] {
            let dir = tempfile::tempdir()?;
            let source = dir.path().join("app.db");
            let mut source_conn = fixture(&source, enabled).await?;
            let selected = select_database(dir.path()).await?;
            assert_ne!(selected, source);
            let mut converted = read_only(&selected).await?;
            let user: (String, String, String) = sqlx::query_as(
				"SELECT username,access_token,account_type FROM minecraft_users",
			)
			.fetch_one(&mut converted)
			.await?;
            assert_eq!(
                user,
                ("NyanQA".into(), "test-access".into(), "elyby".into())
            );
            let retained:(String,String)=sqlx::query_as("SELECT i.path,c.game_version FROM instances i JOIN instance_content_sets c ON c.id=i.applied_content_set_id WHERE i.id='local:fixture'").fetch_one(&mut converted).await?;
            assert_eq!(retained, ("Тестовая сборка".into(), "1.21.4".into()));
            let excluded:bool=sqlx::query_scalar("SELECT enabled FROM instance_sync_preferences WHERE instance_id='local:fixture' AND feature='game_options'").fetch_one(&mut converted).await?;
            assert!(!excluded);
            let group: String = sqlx::query_scalar(
                "SELECT name FROM instance_groups WHERE id='group-a'",
            )
            .fetch_one(&mut converted)
            .await?;
            assert_eq!(group, "Избранное");

            let actual:bool=sqlx::query_scalar("SELECT globally_enabled FROM sync_feature_settings WHERE feature='game_options'").fetch_one(&mut converted).await?;
            assert_eq!(enabled, actual);
            let codec:String=sqlx::query_scalar("SELECT value_codec FROM synced_game_option_values WHERE option_id='chat_scale'").fetch_one(&mut converted).await?;
            assert_eq!(codec, "catalog");
            let unknown:i64=sqlx::query_scalar("SELECT count(*) FROM synced_game_option_values WHERE option_id='unknown_option'").fetch_one(&mut converted).await?;
            assert_eq!(unknown, 0);
            let old_ledger: i64 =
                sqlx::query_scalar("SELECT count(*) FROM _sqlx_migrations")
                    .fetch_one(&mut source_conn)
                    .await?;
            assert_eq!(old_ledger, 1);
            assert_eq!(select_database(dir.path()).await?, selected);
        }
        Ok(())
    }

    #[tokio::test]
    async fn incomplete_target_is_rejected() -> crate::Result<()> {
        let dir = tempfile::tempdir()?;
        let mut conn = SqliteConnection::connect_with(
            &SqliteConnectOptions::new()
                .filename(dir.path().join(CONVERTED))
                .create_if_missing(true),
        )
        .await?;
        sqlx::query("CREATE TABLE app_metadata(key TEXT, value TEXT)")
            .execute(&mut conn)
            .await?;
        assert!(select_database(dir.path()).await.is_err());
        Ok(())
    }
}
