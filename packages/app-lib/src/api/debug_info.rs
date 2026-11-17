use crate::util::io::IOError;
use crate::{ErrorKind, State};
use serde::Serialize;
use std::fs::File;
use std::io::{BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tokio_util::sync::CancellationToken;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DebugInfoExportStage {
    Preparing,
    Exporting,
    Finishing,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DebugInfoExportProgress {
    pub stage: DebugInfoExportStage,
    pub processed_bytes: u64,
    pub total_bytes: Option<u64>,
}

struct ExportFile {
    path: PathBuf,
    name: String,
    length: u64,
}

pub async fn export_debug_info(
    export_path: PathBuf,
    cancel: CancellationToken,
    on_progress: impl Fn(DebugInfoExportProgress) + Send + Sync + 'static,
) -> crate::Result<bool> {
    on_progress(DebugInfoExportProgress {
        stage: DebugInfoExportStage::Preparing,
        processed_bytes: 0,
        total_bytes: None,
    });
    let state = tokio::select! {
        _ = cancel.cancelled() => return Ok(false),
        state = State::get() => state?,
    };
    let directories = state.directories.clone();
    let database_files = tempfile::tempdir()?;
    let report = build_launch_report(&state).await?;
    tokio::fs::write(
        database_files.path().join("launch-diagnostics.json"),
        report,
    )
    .await?;
    let runtime = tokio::runtime::Handle::current();
    if cancel.is_cancelled() {
        return Ok(false);
    }

    tokio::task::spawn_blocking(move || -> crate::Result<bool> {
        let parent = export_path.parent().ok_or(ErrorKind::InputError(
            "export path has no parent directory".to_string(),
        ))?;
        let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
        let excluded_paths = [
            std::fs::canonicalize(temporary.path())?,
            std::fs::canonicalize(&export_path)
                .unwrap_or_else(|_| export_path.clone()),
        ];
        let mut files = Vec::new();
        collect_files(
            &mut files,
            database_files.path(),
            "diagnostics",
            &excluded_paths,
            &cancel,
        )?;
        if let Some(logs_dir) = directories.launcher_logs_dir() {
            collect_files(
                &mut files,
                &logs_dir,
                "launcher_logs",
                &excluded_paths,
                &cancel,
            )?;
        }
        let instances_dir = directories.instances_dir();
        match std::fs::read_dir(&instances_dir) {
            Ok(entries) => {
                for entry in entries {
                    if cancel.is_cancelled() {
                        return Ok(false);
                    }
                    let entry = entry?;
                    if !entry.file_type()?.is_dir() {
                        continue;
                    }
                    let instance_dir = entry.path();
                    let prefix = format!(
                        "instances/{}",
                        entry.file_name().to_string_lossy()
                    );
                    for name in
                        ["logs", "crash-reports", "authlib-injector.log"]
                    {
                        collect_files(
                            &mut files,
                            &instance_dir.join(name),
                            &format!("{prefix}/{name}"),
                            &excluded_paths,
                            &cancel,
                        )?;
                    }
                    for file in std::fs::read_dir(&instance_dir)? {
                        if cancel.is_cancelled() {
                            return Ok(false);
                        }
                        let file = file?;
                        let name = file.file_name();
                        let name = name.to_string_lossy();
                        if name.starts_with("hs_err_pid")
                            && name.ends_with(".log")
                        {
                            collect_files(
                                &mut files,
                                &file.path(),
                                &format!("{prefix}/{name}"),
                                &excluded_paths,
                                &cancel,
                            )?;
                        }
                    }
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(IOError::with_path(error, &instances_dir).into());
            }
        }
        if cancel.is_cancelled() {
            return Ok(false);
        }

        let total_bytes = files.iter().map(|file| file.length).sum();
        let mut progress = DebugInfoExportProgress {
            stage: DebugInfoExportStage::Exporting,
            processed_bytes: 0,
            total_bytes: Some(total_bytes),
        };
        on_progress(progress);
        let mut last_sent = Instant::now();
        let mut archive =
            ZipWriter::new(BufWriter::new(temporary.as_file_mut()));
        for file in files {
            if cancel.is_cancelled() {
                return Ok(false);
            }
            let input = match File::open(&file.path) {
                Ok(input) => input,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    progress.total_bytes =
                        progress.total_bytes.map(|total| total - file.length);
                    continue;
                }
                Err(error) => {
                    return Err(IOError::with_path(error, &file.path).into());
                }
            };
            archive
                .start_file(
                    file.name.strip_suffix(".gz").unwrap_or(&file.name),
                    file_options()
                        .large_file(file.length >= zip::ZIP64_BYTES_THR),
                )
                .map_err(std::io::Error::from)?;
            let text = read_export_log(
                input,
                file.length,
                file.path.extension().is_some_and(|ext| ext == "gz"),
            )?;
            let text = runtime.block_on(
                crate::install::diagnostics::censor_support_text(
                    text,
                    &state.pool,
                ),
            )?;
            archive.write_all(text.as_bytes())?;
            progress.processed_bytes += file.length;
            if last_sent.elapsed() >= Duration::from_millis(200) {
                on_progress(progress);
                last_sent = Instant::now();
            }
        }
        if cancel.is_cancelled() {
            return Ok(false);
        }
        progress.stage = DebugInfoExportStage::Finishing;
        on_progress(progress);
        archive
            .start_file("environment.json", file_options())
            .map_err(std::io::Error::from)?;
        let environment = serde_json::to_string_pretty(&serde_json::json!({
            "exported_at": chrono::Utc::now(),
            "app_version": env!("CARGO_PKG_VERSION"),
            "app_identifier": directories.app_identifier,
            "os": std::env::consts::OS,
            "os_version": sysinfo::System::long_os_version(),
            "kernel_version": sysinfo::System::kernel_version(),
            "architecture": std::env::consts::ARCH,
            "settings_dir": directories.settings_dir,
            "config_dir": directories.config_dir,
        }))?;
        let environment = runtime.block_on(
            crate::install::diagnostics::censor_support_text(
                environment,
                &state.pool,
            ),
        )?;
        archive.write_all(environment.as_bytes())?;
        archive.finish().map_err(std::io::Error::from)?.flush()?;
        temporary.as_file().sync_all()?;
        if cancel.is_cancelled() {
            return Ok(false);
        }
        temporary
            .persist(&export_path)
            .map_err(|error| IOError::with_path(error.error, &export_path))?;
        Ok(true)
    })
    .await?
}

fn file_options() -> SimpleFileOptions {
    SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o600)
}

fn collect_files(
    files: &mut Vec<ExportFile>,
    path: &Path,
    archive_name: &str,
    excluded_paths: &[PathBuf],
    cancel: &CancellationToken,
) -> crate::Result<()> {
    if cancel.is_cancelled() {
        return Ok(());
    }
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(());
        }
        Err(error) => return Err(IOError::with_path(error, path).into()),
    };
    if metadata.is_dir() {
        let entries = std::fs::read_dir(path)
            .map_err(|error| IOError::with_path(error, path))?;
        for entry in entries {
            if cancel.is_cancelled() {
                return Ok(());
            }
            let entry = entry?;
            collect_files(
                files,
                &entry.path(),
                &format!(
                    "{archive_name}/{}",
                    entry.file_name().to_string_lossy()
                ),
                excluded_paths,
                cancel,
            )?;
        }
    } else if metadata.is_file()
        && !std::fs::canonicalize(path)
            .is_ok_and(|path| excluded_paths.contains(&path))
    {
        files.push(ExportFile {
            path: path.to_path_buf(),
            name: archive_name.to_string(),
            length: metadata.len(),
        });
    }
    Ok(())
}

async fn build_launch_report(state: &State) -> crate::Result<String> {
    let accounts = sqlx::query_scalar::<_, String>(
        "SELECT json_object('type',account_type,'active',active,'expires',expires) FROM minecraft_users",
    ).fetch_all(&state.pool).await?;
    let instances = sqlx::query_scalar::<_, String>(
        "SELECT json_object('id',i.id,'path',i.path,'name',i.name,'last_played',i.last_played,'game_version',c.game_version,'loader',c.loader,'loader_version',c.loader_version,'source_kind',c.source_kind) FROM instances i LEFT JOIN instance_content_sets c ON c.id=i.applied_content_set_id",
    ).fetch_all(&state.pool).await?;
    let mut mod_files = Vec::new();
    for instance in &instances {
        let metadata: serde_json::Value = serde_json::from_str(instance)?;
        if let Some(folder) = metadata["path"].as_str() {
            let path =
                state.directories.instances_dir().join(folder).join("mods");
            let mut names = Vec::new();
            if let Ok(mut entries) = tokio::fs::read_dir(path).await {
                while let Some(entry) = entries.next_entry().await? {
                    if entry.file_type().await?.is_file() {
                        names.push(
                            entry.file_name().to_string_lossy().into_owned(),
                        );
                    }
                }
            }
            names.sort();
            mod_files.push(
                serde_json::json!({"instance_id":metadata["id"],"files":names}),
            );
        }
    }
    let jvm_settings = sqlx::query_scalar::<_, String>(
        "SELECT json(extra_launch_args) FROM settings",
    )
    .fetch_all(&state.pool)
    .await?;
    let overrides = sqlx::query_scalar::<_, String>("SELECT json_object('instance_id',instance_id,'java_path',json_extract(overrides,'$.java_path'),'java_args',json_extract(overrides,'$.extra_launch_args')) FROM instance_launch_overrides").fetch_all(&state.pool).await?;
    let libraries =
        crate::nyanrinth::authentication::external_auth_library_states()
            .await?;
    let mut injector_health = Vec::new();
    for provider in crate::nyanrinth::authentication::external_auth_providers()
    {
        let health =
            match crate::nyanrinth::libraries::get_authlib_injector_library(
                provider.id,
                provider.display_name,
                provider.library(),
            )
            .await
            {
                Ok(path) => match tokio::fs::read(&path).await {
                    Ok(bytes) => {
                        use sha2::{Digest, Sha256};
                        serde_json::json!({"provider":provider.id,"file":path,"size":bytes.len(),"sha256":Sha256::digest(&bytes).iter().map(|byte| format!("{byte:02x}")).collect::<String>()})
                    }
                    Err(error) => {
                        serde_json::json!({"provider":provider.id,"error":error.to_string()})
                    }
                },
                Err(error) => {
                    serde_json::json!({"provider":provider.id,"error":error.to_string(),"bundled_fallback":true})
                }
            };
        injector_health.push(health);
    }
    let report = serde_json::to_string_pretty(&serde_json::json!({
        "accounts": accounts.iter().map(|v| serde_json::from_str::<serde_json::Value>(v)).collect::<Result<Vec<_>,_>>()?,
        "mods": mod_files,
        "instances": instances.iter().map(|v| serde_json::from_str::<serde_json::Value>(v)).collect::<Result<Vec<_>,_>>()?,
        "auth_libraries": libraries,
        "injector_health": injector_health,
        "global_java_args": jvm_settings,
        "launch_overrides": overrides,
        "note": "Logs are limited to 8 MiB per file; gzip logs are decompressed. Account database, passwords, cookies and environment variables are not included."
    }))?;
    crate::install::diagnostics::censor_support_text(report, &state.pool).await
}

fn read_export_log(
    mut input: File,
    length: u64,
    gzip: bool,
) -> crate::Result<String> {
    const LIMIT: u64 = 8 * 1024 * 1024;
    let truncated = !gzip && length > LIMIT;
    if truncated {
        input.seek(SeekFrom::End(-(LIMIT as i64)))?;
    }
    let reader: Box<dyn Read> = if gzip {
        Box::new(flate2::read::GzDecoder::new(input))
    } else {
        Box::new(input)
    };
    let mut bytes = Vec::new();
    reader.take(LIMIT).read_to_end(&mut bytes)?;
    let text = String::from_utf8_lossy(&bytes).into_owned();
    Ok(if truncated {
        format!("[Earlier log content omitted; last 8 MiB follow]\n{text}")
    } else {
        text
    })
}

#[cfg(test)]
mod export_tests {
    use super::*;
    #[test]
    fn exports_decompressed_logs_and_latest_part_of_large_logs() {
        let mut gzip = tempfile::tempfile().unwrap();
        {
            let mut encoder = flate2::write::GzEncoder::new(
                &mut gzip,
                flate2::Compression::default(),
            );
            encoder.write_all(b"authlib-injector diagnostic").unwrap();
            encoder.finish().unwrap();
        }
        let length = gzip.metadata().unwrap().len();
        gzip.rewind().unwrap();
        assert_eq!(
            read_export_log(gzip, length, true).unwrap(),
            "authlib-injector diagnostic"
        );
        let mut large = tempfile::tempfile().unwrap();
        large.write_all(&vec![b'x'; 8 * 1024 * 1024 + 1]).unwrap();
        large.write_all(b"LATEST").unwrap();
        let length = large.metadata().unwrap().len();
        let text = read_export_log(large, length, false).unwrap();
        assert!(text.starts_with("[Earlier log content omitted"));
        assert!(text.ends_with("LATEST"));
    }
}
