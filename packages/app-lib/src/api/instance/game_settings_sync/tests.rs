use super::*;

#[test]
fn merges_keys_and_converts_legacy_controls_without_touching_local_options() {
	let source = GameOptionsDocument::parse(b"version:1343\r\nkey_key.forward:17\r\nmusic:0.25\r\nfov:0.0\r\nmod.example:keep\r\n").unwrap();
	let config = Config {
		values: read_values(&source, "1.12.2"),
		..Config::default()
	};
	let original = b"version:4189\r\nkey_key.forward:key.keyboard.s\r\nfov:1.0\r\nmod.example:local\r\n";
	let mut target = GameOptionsDocument::parse(original).unwrap();
	assert!(apply_values(&mut target, "1.21.4", &config).unwrap());
	assert_eq!(target.value("key_key.forward"), Some("key.keyboard.w"));
	assert_eq!(target.value("mod.example"), Some("local"));
	assert_eq!(target.value("version"), Some("4189"));
	assert!(
		String::from_utf8(target.serialize().unwrap())
			.unwrap()
			.contains("mod.example:local\r\n")
	);
}

#[test]
fn disabled_options_snapshots_and_launcher_options_stay_local() {
	let source =
		GameOptionsDocument::parse(b"fov:0.0\nfullscreen:true\n").unwrap();
	let config = Config {
		values: read_values(&source, "1.21.4"),
		disabled_options: vec!["fov".into()],
		..Config::default()
	};
	assert!(!config.values.contains_key("fullscreen"));
	let mut target =
		GameOptionsDocument::parse(b"fov:1.0\nfullscreen:false\n").unwrap();
	assert!(!apply_values(&mut target, "1.21.4", &config).unwrap());
	assert_eq!(target.value("fov"), Some("1.0"));
	assert!(read_values(&target, "25w14a").is_empty());
}

#[test]
fn backups_and_compare_before_replace_protect_concurrent_changes() {
	let directory = tempfile::tempdir().unwrap();
	let path = directory.path().join("options.txt");
	let backups = directory.path().join("backups");
	std::fs::write(&path, b"fov:0.0\n").unwrap();
	assert!(
		!replace_document(&path, b"fov:1.0\n", b"fov:0.5\n", &backups).unwrap()
	);
	assert!(
		replace_document(&path, b"fov:0.0\n", b"fov:0.5\n", &backups).unwrap()
	);
	assert_eq!(
		std::fs::read(
			backups.join(format!("{}.txt", sha1_bytes(b"fov:0.0\n")))
		)
		.unwrap(),
		b"fov:0.0\n"
	);
}

#[test]
fn utf16_and_duplicate_keys_roundtrip() {
	let mut bytes = vec![0xff, 0xfe];
	for unit in "fov:0.0\r\nfov:0.5\r\n# note\r\n".encode_utf16() {
		bytes.extend(unit.to_le_bytes());
	}
	let document = GameOptionsDocument::parse(&bytes).unwrap();
	assert_eq!(document.serialize().unwrap(), bytes);
	assert_eq!(document.value("fov"), Some("0.5"));
}

#[test]
fn backups_accept_real_windows_instance_ids() {
	let directory = tempfile::tempdir().unwrap();
	let file = directory.path().join("options.txt");
	std::fs::write(&file, b"fov:0.0\n").unwrap();
	let backup = backup_directory(
		directory.path(),
		"local:5442f671-60bc-4c09-9b80-e5269992aa59",
	);
	assert!(
		replace_document(&file, b"fov:0.0\n", b"fov:0.5\n", &backup).unwrap()
	);
	assert!(backup.is_dir());
	assert_eq!(std::fs::read(&file).unwrap(), b"fov:0.5\n");
}
