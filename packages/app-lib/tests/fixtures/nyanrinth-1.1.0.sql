CREATE TABLE settings (
    id INTEGER NOT NULL CHECK (id = 0),

    max_concurrent_downloads INTEGER NOT NULL DEFAULT 10,
    max_concurrent_writes INTEGER NOT NULL DEFAULT 10,

    theme TEXT NOT NULL DEFAULT 'oled',
    default_page TEXT NOT NULL DEFAULT 'home',
    collapsed_navigation INTEGER NOT NULL DEFAULT TRUE,
    advanced_rendering INTEGER NOT NULL DEFAULT TRUE,
    native_decorations INTEGER NOT NULL DEFAULT FALSE,

    telemetry INTEGER NOT NULL DEFAULT FALSE,
    discord_rpc INTEGER NOT NULL DEFAULT TRUE,
    developer_mode INTEGER NOT NULL DEFAULT FALSE,

    onboarded INTEGER NOT NULL DEFAULT FALSE,

    -- array of strings
    extra_launch_args JSONB NOT NULL,
    -- array of (string, string)
    custom_env_vars JSONB NOT NULL,
    mc_memory_max INTEGER NOT NULL DEFAULT 2048,
    mc_force_fullscreen INTEGER NOT NULL DEFAULT FALSE,
    mc_game_resolution_x INTEGER NOT NULL DEFAULT 854,
    mc_game_resolution_y INTEGER NOT NULL DEFAULT 480,

    hide_on_process_start INTEGER NOT NULL DEFAULT FALSE,

    hook_pre_launch TEXT NULL,
    hook_wrapper TEXT NULL,
    hook_post_exit TEXT NULL,

    custom_dir TEXT NULL,
    prev_custom_dir TEXT NULL,
    migrated INTEGER NOT NULL DEFAULT FALSE,

    PRIMARY KEY (id)
);

INSERT INTO settings (id, extra_launch_args, custom_env_vars) VALUES (0, jsonb_array(), jsonb_array());

CREATE TABLE java_versions (
    major_version INTEGER NOT NULL,
    full_version TEXT NOT NULL,
    architecture TEXT NOT NULL,
    path TEXT NOT NULL,

    PRIMARY KEY (major_version)
);

CREATE TABLE minecraft_users (
    uuid TEXT NOT NULL,
    active INTEGER NOT NULL DEFAULT FALSE,
    username TEXT NOT NULL,
    access_token TEXT NOT NULL,
    refresh_token TEXT NOT NULL,
    expires INTEGER NOT NULL,

    PRIMARY KEY (uuid)
);
CREATE UNIQUE INDEX minecraft_users_active ON minecraft_users(active);

CREATE TABLE minecraft_device_tokens (
    id INTEGER NOT NULL CHECK (id = 0),

    uuid TEXT NOT NULL,
    private_key TEXT NOT NULL,
    x TEXT NOT NULL,
    y TEXT NOT NULL,
    issue_instant INTEGER NOT NULL,
    not_after INTEGER NOT NULL,
    token TEXT NOT NULL,
    display_claims JSONB NOT NULL,

    PRIMARY KEY (id)
);

CREATE TABLE modrinth_users (
    id TEXT NOT NULL,
    active INTEGER NOT NULL DEFAULT FALSE,
    session_id TEXT NOT NULL,
    expires INTEGER NOT NULL,

    PRIMARY KEY (id)
);
CREATE UNIQUE INDEX modrinth_users_active ON modrinth_users(active);

CREATE TABLE cache (
    id TEXT NOT NULL,
    data_type TEXT NOT NULL,
    alias TEXT NULL,

    data JSONB NULL,
    expires INTEGER NOT NULL,

    UNIQUE (data_type, alias),
    PRIMARY KEY (id, data_type)
);

CREATE TABLE profiles (
    path TEXT NOT NULL,
    install_stage TEXT NOT NULL,

    name TEXT NOT NULL,
    icon_path TEXT NULL,

    game_version TEXT NOT NULL,
    mod_loader TEXT NOT NULL,
    mod_loader_version TEXT NULL,

    -- array of strings
    groups JSONB NOT NULL,

    linked_project_id TEXT NULL,
    linked_version_id TEXT NULL,
    locked INTEGER NULL,

    created INTEGER NOT NULL,
    modified INTEGER NOT NULL,
    last_played INTEGER NULL,

    submitted_time_played INTEGER NOT NULL DEFAULT 0,
    recent_time_played INTEGER NOT NULL DEFAULT 0,

    override_java_path TEXT NULL,

    -- array of strings
    override_extra_launch_args JSONB NOT NULL,
    -- array of (string, string)
    override_custom_env_vars JSONB NOT NULL,

    override_mc_memory_max INTEGER NULL,
    override_mc_force_fullscreen INTEGER NULL,
    override_mc_game_resolution_x INTEGER NULL,
    override_mc_game_resolution_y INTEGER NULL,

    override_hook_pre_launch TEXT NULL,
    override_hook_wrapper TEXT NULL,
    override_hook_post_exit TEXT NULL,


    PRIMARY KEY (path)
);

CREATE TABLE processes (
    pid INTEGER NOT NULL,
    start_time INTEGER NOT NULL,
    name TEXT NOT NULL,
    executable TEXT NOT NULL,
    profile_path TEXT NOT NULL,
    post_exit_command TEXT NULL,

    UNIQUE (pid),
    PRIMARY KEY (pid),
    FOREIGN KEY (profile_path) REFERENCES profiles(path)
);
CREATE INDEX processes_profile_path ON processes(profile_path);

;
DROP INDEX modrinth_users_active;
DROP INDEX minecraft_users_active;
;
ALTER TABLE settings ADD COLUMN personalized_ads INTEGER NOT NULL DEFAULT TRUE;
;
ALTER TABLE settings ADD COLUMN toggle_sidebar INTEGER NOT NULL DEFAULT FALSE;
ALTER TABLE settings ADD COLUMN feature_flags JSONB NOT NULL default '{}';
;
ALTER TABLE profiles ADD COLUMN protocol_version INTEGER NULL

;
CREATE TABLE join_log (
    profile_path TEXT NOT NULL,
    host TEXT NOT NULL,
    port INTEGER NOT NULL,
    join_time INTEGER NOT NULL,

    PRIMARY KEY (profile_path, host, port),
    FOREIGN KEY (profile_path) REFERENCES profiles(path) ON DELETE CASCADE
);
CREATE INDEX join_log_profile_path ON join_log(profile_path);

;
CREATE TABLE default_minecraft_capes (
    minecraft_user_uuid TEXT NOT NULL,
    id TEXT NOT NULL,

    PRIMARY KEY (minecraft_user_uuid, id)
);

-- Emulate a ON UPDATE CASCADE foreign key constraint for the user UUID on the default_minecraft_capes table,
-- but allowing deletion of the user UUID in the minecraft_users table. This allows the application to temporarily
-- keep skin state around for logged-out users, allowing them to retain their skins under the right conditions
CREATE TRIGGER default_minecraft_capes_user_uuid_insert_check
    BEFORE INSERT ON default_minecraft_capes FOR EACH ROW
    BEGIN
        SELECT CASE WHEN NOT EXISTS (
            SELECT 1 FROM minecraft_users WHERE uuid = NEW.minecraft_user_uuid
        ) THEN RAISE(ABORT, 'Cannot add a default cape for an unknown Minecraft user UUID') END;
    END;

CREATE TRIGGER default_minecraft_capes_user_uuid_update_check
    BEFORE UPDATE ON default_minecraft_capes FOR EACH ROW
    BEGIN
        SELECT CASE WHEN NOT EXISTS (
            SELECT 1 FROM minecraft_users WHERE uuid = NEW.minecraft_user_uuid
        ) THEN RAISE(ABORT, 'Cannot change a default cape to refer to an unknown Minecraft user UUID') END;
    END;

CREATE TRIGGER default_minecraft_capes_user_uuid_update_cascade
    AFTER UPDATE OF uuid ON minecraft_users FOR EACH ROW
    BEGIN
        UPDATE default_minecraft_capes SET minecraft_user_uuid = NEW.uuid WHERE minecraft_user_uuid = OLD.uuid;
    END;

CREATE TABLE custom_minecraft_skins (
    minecraft_user_uuid TEXT NOT NULL,
    texture_key TEXT NOT NULL,
    variant TEXT NOT NULL CHECK (variant IN ('CLASSIC', 'SLIM', 'UNKNOWN')),
    cape_id TEXT,

    PRIMARY KEY (minecraft_user_uuid, texture_key, variant, cape_id),
    FOREIGN KEY (texture_key) REFERENCES custom_minecraft_skin_textures(texture_key)
        ON DELETE CASCADE ON UPDATE CASCADE
);

-- Similar partial foreign key emulation as above
CREATE TRIGGER custom_minecraft_skins_user_uuid_insert_check
    BEFORE INSERT ON custom_minecraft_skins FOR EACH ROW
    BEGIN
        SELECT CASE WHEN NOT EXISTS (
            SELECT 1 FROM minecraft_users WHERE uuid = NEW.minecraft_user_uuid
        ) THEN RAISE(ABORT, 'Cannot add a custom skin for an unknown Minecraft user UUID') END;
    END;

CREATE TRIGGER custom_minecraft_skins_user_uuid_update_check
    BEFORE UPDATE ON custom_minecraft_skins FOR EACH ROW
    BEGIN
        SELECT CASE WHEN NOT EXISTS (
            SELECT 1 FROM minecraft_users WHERE uuid = NEW.minecraft_user_uuid
        ) THEN RAISE(ABORT, 'Cannot change a custom skin to refer to an unknown Minecraft user UUID') END;
    END;

CREATE TRIGGER custom_minecraft_skins_user_uuid_update_cascade
    AFTER UPDATE OF uuid ON minecraft_users FOR EACH ROW
    BEGIN
        UPDATE custom_minecraft_skins SET minecraft_user_uuid = NEW.uuid WHERE minecraft_user_uuid = OLD.uuid;
    END;

CREATE TABLE custom_minecraft_skin_textures (
    texture_key TEXT NOT NULL,
    texture PNG BLOB NOT NULL,

    PRIMARY KEY (texture_key)
);

CREATE TRIGGER custom_minecraft_skin_texture_delete_cleanup
    AFTER DELETE ON custom_minecraft_skins FOR EACH ROW
    BEGIN
        DELETE FROM custom_minecraft_skin_textures WHERE texture_key NOT IN (
            SELECT texture_key FROM custom_minecraft_skins
        );
    END;

;
ALTER TABLE profiles ADD COLUMN launcher_feature_version TEXT NOT NULL DEFAULT 'none'

;
CREATE TABLE attached_world_data (
    profile_path TEXT NOT NULL,
    world_type TEXT CHECK ( world_type in ('singleplayer', 'server') ) NOT NULL,
    world_id TEXT NOT NULL,
    display_status TEXT NOT NULL DEFAULT 'normal',

    PRIMARY KEY (profile_path, world_type, world_id),
    FOREIGN KEY (profile_path) REFERENCES profiles(path) ON DELETE CASCADE
);
CREATE INDEX attached_world_data_profile_path ON attached_world_data(profile_path);

;
ALTER TABLE settings ADD COLUMN hide_nametag_skins_page INTEGER NOT NULL DEFAULT 0 CHECK (hide_nametag_skins_page IN (0, 1));

;
-- [AR] - SQL Migration
ALTER TABLE minecraft_users ADD COLUMN account_type varchar(32) NOT NULL DEFAULT 'unknown';

UPDATE minecraft_users SET account_type = 'microsoft' WHERE access_token != 'null';
UPDATE minecraft_users SET account_type = 'pirate' WHERE access_token == 'null';
;
ALTER TABLE settings
ADD COLUMN skipped_update TEXT NULL;
ALTER TABLE settings
ADD COLUMN pending_update_toast_for_version TEXT NULL;
ALTER TABLE settings
ADD COLUMN auto_download_updates INT NULL;

;
ALTER TABLE settings ADD COLUMN version INTEGER NOT NULL DEFAULT 1;
;
-- Add migration script here
ALTER TABLE settings ADD COLUMN locale TEXT NOT NULL DEFAULT 'en-US';

;
ALTER TABLE attached_world_data ADD COLUMN project_id TEXT;
ALTER TABLE attached_world_data ADD COLUMN content_kind TEXT;

;
DROP TABLE IF EXISTS default_minecraft_capes;

-- Keep only one saved skin per Minecraft account and texture.
-- variant and cape_id are settings on that saved skin, not part of the skin identity.
DELETE FROM custom_minecraft_skins
WHERE rowid NOT IN (
    SELECT MAX(rowid)
    FROM custom_minecraft_skins
    GROUP BY minecraft_user_uuid, texture_key
);

CREATE UNIQUE INDEX custom_minecraft_skins_one_per_texture
    ON custom_minecraft_skins (minecraft_user_uuid, texture_key);

;
DROP TRIGGER IF EXISTS default_minecraft_capes_user_uuid_insert_check;
DROP TRIGGER IF EXISTS default_minecraft_capes_user_uuid_update_check;
DROP TRIGGER IF EXISTS default_minecraft_capes_user_uuid_update_cascade;

;
ALTER TABLE profiles
ADD COLUMN preferred_update_channel TEXT NOT NULL DEFAULT 'release';

;
ALTER TABLE custom_minecraft_skins
ADD COLUMN display_order INTEGER NOT NULL DEFAULT 0;

UPDATE custom_minecraft_skins
SET display_order = (
	SELECT COUNT(*)
	FROM custom_minecraft_skins AS previous
	WHERE previous.minecraft_user_uuid = custom_minecraft_skins.minecraft_user_uuid
		AND previous.rowid <= custom_minecraft_skins.rowid
) - 1;

CREATE INDEX custom_minecraft_skins_user_display_order
	ON custom_minecraft_skins (minecraft_user_uuid, display_order);

;
CREATE TABLE instances (
	id TEXT NOT NULL,
	path TEXT NOT NULL,
	applied_content_set_id TEXT NULL,

	install_stage TEXT NOT NULL,
	launcher_feature_version TEXT NOT NULL,
	update_channel TEXT NOT NULL DEFAULT 'release',

	name TEXT NOT NULL,
	icon_path TEXT NULL,

	created INTEGER NOT NULL,
	modified INTEGER NOT NULL,
	last_played INTEGER NULL,

	submitted_time_played INTEGER NOT NULL DEFAULT 0,
	recent_time_played INTEGER NOT NULL DEFAULT 0,

	PRIMARY KEY (id),
	UNIQUE (path)
);

CREATE INDEX instances_path ON instances(path);
CREATE INDEX instances_applied_content_set_id
	ON instances(applied_content_set_id);

CREATE TABLE instance_links (
	instance_id TEXT NOT NULL,
	link_kind TEXT NOT NULL,

	modrinth_project_id TEXT NULL,
	modrinth_version_id TEXT NULL,

	server_project_id TEXT NULL,

	content_project_id TEXT NULL,
	content_version_id TEXT NULL,

	hosting_server_id TEXT NULL,
	hosting_instance_ids JSONB NULL,
	hosting_active_instance_id TEXT NULL,

	shared_instance_id TEXT NULL,

	PRIMARY KEY (instance_id),
	FOREIGN KEY (instance_id) REFERENCES instances(id) ON DELETE CASCADE
);

CREATE INDEX instance_links_link_kind ON instance_links(link_kind);
CREATE INDEX instance_links_modrinth_project_id
	ON instance_links(modrinth_project_id);
CREATE INDEX instance_links_server_project_id
	ON instance_links(server_project_id);
CREATE INDEX instance_links_hosting_server_id
	ON instance_links(hosting_server_id);
CREATE INDEX instance_links_hosting_active_instance_id
	ON instance_links(hosting_active_instance_id);
CREATE INDEX instance_links_shared_instance_id
	ON instance_links(shared_instance_id);

CREATE TABLE instance_groups (
	instance_id TEXT NOT NULL,
	group_name TEXT NOT NULL,

	PRIMARY KEY (instance_id, group_name),
	FOREIGN KEY (instance_id) REFERENCES instances(id) ON DELETE CASCADE
);

CREATE INDEX instance_groups_group_name ON instance_groups(group_name);

CREATE TABLE instance_launch_overrides (
	instance_id TEXT NOT NULL,
	overrides JSONB NOT NULL,

	PRIMARY KEY (instance_id),
	FOREIGN KEY (instance_id) REFERENCES instances(id) ON DELETE CASCADE
);

CREATE TABLE instance_content_sets (
	id TEXT NOT NULL,
	instance_id TEXT NOT NULL,

	name TEXT NOT NULL,
	source_kind TEXT NOT NULL,
	status TEXT NOT NULL,

	game_version TEXT NOT NULL,
	protocol_version INTEGER NULL,
	loader TEXT NOT NULL,
	loader_version TEXT NULL,

	created INTEGER NOT NULL,
	modified INTEGER NOT NULL,

	PRIMARY KEY (id),
	FOREIGN KEY (instance_id) REFERENCES instances(id) ON DELETE CASCADE
);

CREATE INDEX instance_content_sets_instance_id
	ON instance_content_sets(instance_id);

CREATE TABLE instance_content_set_remote_refs (
	content_set_id TEXT NOT NULL,
	ref_type TEXT NOT NULL,
	ref_id TEXT NOT NULL,

	PRIMARY KEY (content_set_id, ref_type),
	FOREIGN KEY (content_set_id)
		REFERENCES instance_content_sets(id)
		ON DELETE CASCADE
);

CREATE INDEX instance_content_set_remote_refs_ref
	ON instance_content_set_remote_refs(ref_type, ref_id);

CREATE TABLE instance_content_set_sync_state (
	content_set_id TEXT NOT NULL,
	provider TEXT NOT NULL,

	applied_update_id TEXT NULL,
	latest_available_update_id TEXT NULL,
	checked_at INTEGER NULL,
	status TEXT NOT NULL,

	PRIMARY KEY (content_set_id),
	FOREIGN KEY (content_set_id)
		REFERENCES instance_content_sets(id)
		ON DELETE CASCADE
);

CREATE INDEX instance_content_set_sync_state_provider
	ON instance_content_set_sync_state(provider);

CREATE INDEX instance_content_set_sync_state_latest_available_update_id
	ON instance_content_set_sync_state(latest_available_update_id);

CREATE TABLE instance_files (
	id TEXT NOT NULL,
	instance_id TEXT NOT NULL,

	relative_path TEXT NOT NULL,
	file_name TEXT NOT NULL,
	enabled INTEGER NOT NULL,

	sha1 TEXT NOT NULL,
	size INTEGER NOT NULL,
	missing INTEGER NOT NULL DEFAULT 0,

	added_at INTEGER NOT NULL,
	modified_at INTEGER NOT NULL,

	PRIMARY KEY (id),
	UNIQUE (instance_id, relative_path),
	FOREIGN KEY (instance_id) REFERENCES instances(id) ON DELETE CASCADE
);

CREATE INDEX instance_files_instance_id ON instance_files(instance_id);
CREATE INDEX instance_files_sha1 ON instance_files(sha1);
CREATE INDEX instance_files_missing ON instance_files(missing);

CREATE TABLE instance_content_entries (
	id TEXT NOT NULL,
	instance_id TEXT NOT NULL,
	content_set_id TEXT NOT NULL,
	file_id TEXT NULL,

	project_type TEXT NOT NULL,
	project_id TEXT NULL,
	version_id TEXT NULL,

	source_kind TEXT NOT NULL,
	server_requirement TEXT NOT NULL,
	client_requirement TEXT NOT NULL,
	enabled INTEGER NOT NULL DEFAULT 1,

	added_at INTEGER NOT NULL,
	modified_at INTEGER NOT NULL,

	PRIMARY KEY (id),
	FOREIGN KEY (instance_id) REFERENCES instances(id) ON DELETE CASCADE,
	FOREIGN KEY (content_set_id)
		REFERENCES instance_content_sets(id)
		ON DELETE CASCADE,
	FOREIGN KEY (file_id) REFERENCES instance_files(id) ON DELETE SET NULL
);

CREATE INDEX instance_content_entries_instance_id
	ON instance_content_entries(instance_id);
CREATE INDEX instance_content_entries_content_set_id
	ON instance_content_entries(content_set_id);
CREATE INDEX instance_content_entries_file_id
	ON instance_content_entries(file_id);
CREATE INDEX instance_content_entries_project_id
	ON instance_content_entries(project_id);
CREATE INDEX instance_content_entries_version_id
	ON instance_content_entries(version_id);
CREATE INDEX instance_content_entries_source_kind
	ON instance_content_entries(source_kind);

CREATE TABLE instance_content_update_checks (
	content_entry_id TEXT NOT NULL,

	update_channel TEXT NOT NULL,
	update_version_id TEXT NULL,
	checked_at INTEGER NOT NULL,

	PRIMARY KEY (content_entry_id),
	FOREIGN KEY (content_entry_id)
		REFERENCES instance_content_entries(id)
		ON DELETE CASCADE
);

CREATE INDEX instance_content_update_checks_update_version_id
	ON instance_content_update_checks(update_version_id);

INSERT INTO instances (
	id,
	path,
	applied_content_set_id,
	install_stage,
	launcher_feature_version,
	update_channel,
	name,
	icon_path,
	created,
	modified,
	last_played,
	submitted_time_played,
	recent_time_played
)
SELECT
	'legacy:' || path,
	path,
	'legacy:' || path || ':default',
	install_stage,
	launcher_feature_version,
	preferred_update_channel,
	name,
	icon_path,
	created,
	modified,
	last_played,
	submitted_time_played,
	recent_time_played
FROM profiles;

INSERT INTO instance_content_sets (
	id,
	instance_id,
	name,
	source_kind,
	status,
	game_version,
	protocol_version,
	loader,
	loader_version,
	created,
	modified
)
SELECT
	'legacy:' || path || ':default',
	'legacy:' || path,
	'Default',
	CASE
		WHEN linked_project_id IS NOT NULL
			AND linked_version_id IS NOT NULL
			AND linked_version_id != ''
			THEN 'modrinth_modpack'
		WHEN linked_project_id IS NOT NULL
			AND (linked_version_id IS NULL OR linked_version_id = '')
			THEN 'server_project'
		ELSE 'local'
	END,
	'available',
	game_version,
	protocol_version,
	mod_loader,
	mod_loader_version,
	created,
	modified
FROM profiles;

INSERT INTO instance_links (
	instance_id,
	link_kind,
	modrinth_project_id,
	modrinth_version_id,
	server_project_id,
	content_project_id,
	content_version_id,
	hosting_server_id,
	hosting_instance_ids,
	hosting_active_instance_id,
	shared_instance_id
)
SELECT
	'legacy:' || path,
	CASE
		WHEN linked_project_id IS NULL
			THEN 'unmanaged'
		WHEN linked_version_id IS NULL OR linked_version_id = ''
			THEN 'server_project'
		ELSE 'modrinth_modpack'
	END,
	CASE
		WHEN linked_version_id IS NOT NULL AND linked_version_id != ''
			THEN linked_project_id
		ELSE NULL
	END,
	CASE
		WHEN linked_version_id IS NOT NULL AND linked_version_id != ''
			THEN linked_version_id
		ELSE NULL
	END,
	CASE
		WHEN linked_project_id IS NOT NULL
			AND (linked_version_id IS NULL OR linked_version_id = '')
			THEN linked_project_id
		ELSE NULL
	END,
	NULL,
	NULL,
	NULL,
	NULL,
	NULL,
	NULL
FROM profiles;

INSERT OR IGNORE INTO instance_groups (instance_id, group_name)
SELECT
	'legacy:' || profiles.path,
	json_each.value
FROM profiles, json_each(profiles.groups);

INSERT INTO instance_launch_overrides (
	instance_id,
	overrides
)
SELECT
	'legacy:' || path,
	jsonb(json_object(
		'java_path', override_java_path,
		'extra_launch_args', json(CASE
			WHEN json_type(override_extra_launch_args) = 'null' THEN 'null'
			ELSE json(override_extra_launch_args)
		END),
		'custom_env_vars', json(CASE
			WHEN json_type(override_custom_env_vars) = 'null' THEN 'null'
			ELSE json(override_custom_env_vars)
		END),
		'memory', json(CASE
			WHEN override_mc_memory_max IS NULL THEN 'null'
			ELSE json_object('maximum', override_mc_memory_max)
		END),
		'force_fullscreen', json(CASE
			WHEN override_mc_force_fullscreen IS NULL THEN 'null'
			WHEN override_mc_force_fullscreen = 0 THEN 'false'
			ELSE 'true'
		END),
		'game_resolution', json(CASE
			WHEN override_mc_game_resolution_x IS NULL
				OR override_mc_game_resolution_y IS NULL
				THEN 'null'
			ELSE json_array(
				override_mc_game_resolution_x,
				override_mc_game_resolution_y
			)
		END),
		'hooks', json_object(
			'pre_launch', COALESCE(override_hook_pre_launch, ''),
			'wrapper', COALESCE(override_hook_wrapper, ''),
			'post_exit', COALESCE(override_hook_post_exit, '')
		)
	))
FROM profiles;

;
CREATE TEMP TABLE finish_instance_profile_migration_validation (
	error_count INTEGER NOT NULL CHECK (error_count = 0)
);

INSERT INTO finish_instance_profile_migration_validation (error_count)
SELECT COUNT(*)
FROM profiles
LEFT JOIN instances ON instances.path = profiles.path
WHERE instances.id IS NULL;

INSERT INTO finish_instance_profile_migration_validation (error_count)
SELECT COUNT(*)
FROM instances
LEFT JOIN instance_content_sets
	ON instance_content_sets.id = instances.applied_content_set_id
	AND instance_content_sets.instance_id = instances.id
WHERE instances.applied_content_set_id IS NULL
	OR instance_content_sets.id IS NULL;

DROP TABLE finish_instance_profile_migration_validation;

DROP INDEX processes_profile_path;
ALTER TABLE processes RENAME TO processes_old;

CREATE TABLE processes (
	pid INTEGER NOT NULL,
	start_time INTEGER NOT NULL,
	name TEXT NOT NULL,
	executable TEXT NOT NULL,
	instance_id TEXT NOT NULL,
	post_exit_command TEXT NULL,

	UNIQUE (pid),
	PRIMARY KEY (pid),
	FOREIGN KEY (instance_id) REFERENCES instances(id) ON DELETE CASCADE
);

INSERT INTO processes (
	pid,
	start_time,
	name,
	executable,
	instance_id,
	post_exit_command
)
SELECT
	processes_old.pid,
	processes_old.start_time,
	processes_old.name,
	processes_old.executable,
	instances.id,
	processes_old.post_exit_command
FROM processes_old
JOIN instances ON instances.path = processes_old.profile_path;

DROP TABLE processes_old;

CREATE INDEX processes_instance_id ON processes(instance_id);

DROP INDEX join_log_profile_path;
ALTER TABLE join_log RENAME TO join_log_old;

CREATE TABLE join_log (
	instance_id TEXT NOT NULL,
	host TEXT NOT NULL,
	port INTEGER NOT NULL,
	join_time INTEGER NOT NULL,

	PRIMARY KEY (instance_id, host, port),
	FOREIGN KEY (instance_id) REFERENCES instances(id) ON DELETE CASCADE
);

INSERT INTO join_log (
	instance_id,
	host,
	port,
	join_time
)
SELECT
	instances.id,
	join_log_old.host,
	join_log_old.port,
	join_log_old.join_time
FROM join_log_old
JOIN instances ON instances.path = join_log_old.profile_path;

DROP TABLE join_log_old;

CREATE INDEX join_log_instance_id ON join_log(instance_id);

DROP INDEX attached_world_data_profile_path;
ALTER TABLE attached_world_data RENAME TO attached_world_data_old;

CREATE TABLE attached_world_data (
	instance_id TEXT NOT NULL,
	world_type TEXT CHECK ( world_type in ('singleplayer', 'server') ) NOT NULL,
	world_id TEXT NOT NULL,
	display_status TEXT NOT NULL DEFAULT 'normal',
	project_id TEXT NULL,
	content_kind TEXT NULL,

	PRIMARY KEY (instance_id, world_type, world_id),
	FOREIGN KEY (instance_id) REFERENCES instances(id) ON DELETE CASCADE
);

INSERT INTO attached_world_data (
	instance_id,
	world_type,
	world_id,
	display_status,
	project_id,
	content_kind
)
SELECT
	instances.id,
	attached_world_data_old.world_type,
	attached_world_data_old.world_id,
	attached_world_data_old.display_status,
	attached_world_data_old.project_id,
	attached_world_data_old.content_kind
FROM attached_world_data_old
JOIN instances ON instances.path = attached_world_data_old.profile_path;

DROP TABLE attached_world_data_old;

CREATE INDEX attached_world_data_instance_id
	ON attached_world_data(instance_id);

DROP TABLE profiles;

;
-- [AR] - SQL Migration
UPDATE minecraft_users SET account_type = 'offline' WHERE account_type == 'pirate';

;
CREATE TABLE app_metadata (
	key TEXT NOT NULL,
	value TEXT NOT NULL,
	updated_at INTEGER NOT NULL DEFAULT (unixepoch()),

	PRIMARY KEY (key)
);

;
CREATE TABLE install_jobs (
	id TEXT NOT NULL PRIMARY KEY,
	instance_id TEXT NULL,
	kind TEXT NOT NULL,
	
	status TEXT NOT NULL,
	state JSONB NOT NULL,

	created INTEGER NOT NULL,
	modified INTEGER NOT NULL,
	finished INTEGER NULL,

	dismissed INTEGER NOT NULL DEFAULT 0,
	FOREIGN KEY (instance_id) REFERENCES instances(id) ON DELETE SET NULL
);

CREATE INDEX install_jobs_instance_id ON install_jobs(instance_id);
CREATE INDEX install_jobs_status ON install_jobs(status);
CREATE INDEX install_jobs_finished ON install_jobs(finished);

;
ALTER TABLE instance_links ADD COLUMN imported_name TEXT NULL;
ALTER TABLE instance_links ADD COLUMN imported_version_number TEXT NULL;
ALTER TABLE instance_links ADD COLUMN imported_filename TEXT NULL;

UPDATE instance_links
SET
	link_kind = 'imported_modpack',
	imported_name = (
		SELECT instances.name
		FROM instances
		WHERE instances.id = instance_links.instance_id
	)
WHERE
	link_kind = 'unmanaged'
	AND EXISTS (
		SELECT 1
		FROM instance_content_entries
		WHERE
			instance_content_entries.instance_id = instance_links.instance_id
			AND instance_content_entries.source_kind = 'imported_modpack'
	);

;
UPDATE instances
SET
	submitted_time_played = CASE
		WHEN submitted_time_played < 0 THEN 0
		ELSE submitted_time_played
	END,
	recent_time_played = CASE
		WHEN recent_time_played < 0 THEN 0
		ELSE recent_time_played
	END
WHERE submitted_time_played < 0
	OR recent_time_played < 0;

;
ALTER TABLE instance_links
	ADD COLUMN shared_instance_role TEXT NULL;

;
ALTER TABLE instance_links
	ADD COLUMN shared_instance_manager_id TEXT NULL;

;
ALTER TABLE instance_links
	ADD COLUMN shared_instance_linked_user_id TEXT NULL;

;
ALTER TABLE instance_links
	ADD COLUMN shared_instance_server_manager_name TEXT NULL;

ALTER TABLE instance_links
	ADD COLUMN shared_instance_server_manager_icon_url TEXT NULL;

;
ALTER TABLE settings DROP COLUMN onboarded;

;
CREATE TABLE onboarding_checklist (
	id INTEGER NOT NULL CHECK (id = 0),
	has_created_instance INTEGER NOT NULL DEFAULT FALSE CHECK (has_created_instance IN (FALSE, TRUE)),
	has_logged_into_minecraft INTEGER NOT NULL DEFAULT FALSE CHECK (
		has_logged_into_minecraft IN (FALSE, TRUE)
	),
	has_logged_into_modrinth INTEGER NOT NULL DEFAULT FALSE CHECK (
		has_logged_into_modrinth IN (FALSE, TRUE)
	),
	show_checklist INTEGER NOT NULL DEFAULT TRUE CHECK (show_checklist IN (FALSE, TRUE)),
	PRIMARY KEY (id)
);

INSERT INTO onboarding_checklist
	(
		id,
		has_created_instance,
		has_logged_into_minecraft,
		has_logged_into_modrinth,
		show_checklist
	)
SELECT
	0,
	EXISTS (SELECT 1 FROM instances),
	EXISTS (SELECT 1 FROM minecraft_users),
	EXISTS (SELECT 1 FROM modrinth_users),
	NOT (
		EXISTS (SELECT 1 FROM instances)
		AND EXISTS (SELECT 1 FROM minecraft_users)
	);

;
-- Older app versions could save the same content entry more than once. Remove the older copies before preventing future duplicates.
WITH ranked_entries AS (
	SELECT
		id,
		ROW_NUMBER() OVER (
			PARTITION BY content_set_id, file_id
			ORDER BY modified_at DESC, added_at DESC, id DESC
		) AS duplicate_rank
	FROM instance_content_entries
	WHERE file_id IS NOT NULL
)
DELETE FROM instance_content_entries
WHERE id IN (
	SELECT id
	FROM ranked_entries
	WHERE duplicate_rank > 1
);

WITH ranked_entries AS (
	SELECT
		id,
		ROW_NUMBER() OVER (
			PARTITION BY content_set_id, project_id, version_id
			ORDER BY modified_at DESC, added_at DESC, id DESC
		) AS duplicate_rank
	FROM instance_content_entries
	WHERE file_id IS NULL
		AND project_id IS NOT NULL
		AND version_id IS NOT NULL
)
DELETE FROM instance_content_entries
WHERE id IN (
	SELECT id
	FROM ranked_entries
	WHERE duplicate_rank > 1
);

CREATE UNIQUE INDEX instance_content_entries_content_set_file_unique
	ON instance_content_entries(content_set_id, file_id)
	WHERE file_id IS NOT NULL;

CREATE UNIQUE INDEX instance_content_entries_content_set_project_version_unique
	ON instance_content_entries(content_set_id, project_id, version_id)
	WHERE file_id IS NULL
		AND project_id IS NOT NULL
		AND version_id IS NOT NULL;

;
CREATE TABLE instance_quarantines (
	instance_id TEXT PRIMARY KEY NOT NULL,
	FOREIGN KEY (instance_id) REFERENCES instances(id) ON DELETE CASCADE
);

;
ALTER TABLE instance_groups RENAME TO legacy_instance_group_memberships;

CREATE TABLE instance_groups (
	id TEXT NOT NULL,
	name TEXT NOT NULL,
	display_order INTEGER NOT NULL DEFAULT 0,

	PRIMARY KEY (id)
);

CREATE TABLE instance_group_memberships (
	instance_id TEXT NOT NULL,
	group_id TEXT NOT NULL,

	PRIMARY KEY (instance_id, group_id),
	FOREIGN KEY (instance_id) REFERENCES instances(id) ON DELETE CASCADE,
	FOREIGN KEY (group_id) REFERENCES instance_groups(id) ON DELETE CASCADE
);

INSERT INTO instance_groups (id, name, display_order)
SELECT
	lower(hex(randomblob(16))),
	group_name,
	ROW_NUMBER() OVER (ORDER BY group_name) - 1
FROM (
	SELECT DISTINCT group_name
	FROM legacy_instance_group_memberships
);

INSERT INTO instance_group_memberships (instance_id, group_id)
SELECT
	legacy.instance_id,
	groups.id
FROM legacy_instance_group_memberships legacy
INNER JOIN instance_groups groups ON groups.name = legacy.group_name;

UPDATE install_jobs
SET state = json_remove(
	json_set(
		state,
		'$.rollback.instance.group_ids',
		json(COALESCE((
			SELECT json_group_array(groups.id)
			FROM json_each(state, '$.rollback.instance.groups') legacy_groups
			INNER JOIN instance_groups groups
				ON groups.name = legacy_groups.value
		), '[]'))
	),
	'$.rollback.instance.groups'
)
WHERE json_type(state, '$.rollback.instance.groups') = 'array';

DROP TABLE legacy_instance_group_memberships;

CREATE INDEX instance_groups_name ON instance_groups(name);
CREATE INDEX instance_groups_display_order ON instance_groups(display_order);
CREATE INDEX instance_group_memberships_group_id
	ON instance_group_memberships(group_id);

INSERT OR IGNORE INTO instance_groups (id, name, display_order)
VALUES ('group:favorites', 'Favorites', -1);

UPDATE instance_groups
SET name = 'Favorites', display_order = -1
WHERE id = 'group:favorites';

;
CREATE TABLE external_auth_libraries (
	provider_id TEXT NOT NULL PRIMARY KEY,
	asset_name TEXT NOT NULL
);

;
CREATE TABLE instance_icon_configs (
	instance_id TEXT NOT NULL,
	background TEXT NOT NULL,
	symbol TEXT NOT NULL,

	PRIMARY KEY (instance_id),
	FOREIGN KEY (instance_id) REFERENCES instances(id) ON DELETE CASCADE
);

CREATE TABLE recent_instance_icon_configs (
	background TEXT NOT NULL,
	symbol TEXT NOT NULL,
	used_at INTEGER NOT NULL,

	PRIMARY KEY (background, symbol)
);

CREATE INDEX recent_instance_icon_configs_used_at
	ON recent_instance_icon_configs(used_at DESC);

;
CREATE TABLE instance_content_locks (
	file_id TEXT NOT NULL,

	PRIMARY KEY (file_id),
	FOREIGN KEY (file_id) REFERENCES instance_files(id) ON DELETE CASCADE
);

;
ALTER TABLE settings
ADD COLUMN sync_theme_across_devices INTEGER NOT NULL DEFAULT TRUE;

ALTER TABLE settings
ADD COLUMN sync_behavior_across_devices INTEGER NOT NULL DEFAULT TRUE;

;
UPDATE settings SET locale = 'ru-RU', discord_rpc = FALSE, telemetry = FALSE;

;
UPDATE settings SET developer_mode = TRUE;

;
CREATE TABLE nyan_game_sync_config (
	id INTEGER PRIMARY KEY CHECK (id = 1),
	payload TEXT NOT NULL
);
CREATE TABLE nyan_game_sync_instances (
	instance_id TEXT PRIMARY KEY REFERENCES instances(id) ON DELETE CASCADE,
	payload TEXT NOT NULL
);

;
CREATE TABLE nyan_storage_migration (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    synced_options_version INTEGER NOT NULL DEFAULT 0,
    content_store_version INTEGER NOT NULL DEFAULT 0,
    completed_at INTEGER
);

INSERT INTO nyan_storage_migration (id)
VALUES (1);

;
CREATE TABLE sync_feature_settings (
    feature TEXT PRIMARY KEY NOT NULL,
    globally_enabled INTEGER NOT NULL CHECK (globally_enabled IN (0, 1)),
    new_instance_default INTEGER NOT NULL CHECK (new_instance_default IN (0, 1))
);

INSERT INTO sync_feature_settings (feature, globally_enabled, new_instance_default)
VALUES
    ('game_options', 0, 1),
    ('command_history', 0, 0),
    ('multiplayer_servers', 0, 0),
    ('creative_hotbars', 0, 0),
    ('screenshots', 0, 0),
    ('resource_packs', 0, 0),
    ('data_packs', 0, 0);

CREATE TABLE instance_sync_preferences (
    instance_id TEXT NOT NULL,
    feature TEXT NOT NULL,
    enabled INTEGER NOT NULL CHECK (enabled IN (0, 1)),
    PRIMARY KEY (instance_id, feature),
    FOREIGN KEY (instance_id) REFERENCES instances (id) ON DELETE CASCADE,
    FOREIGN KEY (feature) REFERENCES sync_feature_settings (feature) ON DELETE CASCADE
);

CREATE INDEX instance_sync_preferences_feature_enabled
    ON instance_sync_preferences (feature, enabled);

INSERT INTO instance_sync_preferences (instance_id, feature, enabled)
SELECT i.id, f.feature, 0
FROM instances AS i
CROSS JOIN sync_feature_settings AS f;

CREATE TABLE synced_game_option_state (
    singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
    revision INTEGER NOT NULL CHECK (revision >= 0),
    catalog_revision INTEGER NOT NULL CHECK (catalog_revision >= 1)
);

INSERT INTO synced_game_option_state (singleton, revision, catalog_revision)
VALUES (1, 0, 1);

CREATE TABLE synced_game_option_values (
    option_id TEXT PRIMARY KEY NOT NULL,
    kind TEXT NOT NULL CHECK (kind IN ('vanilla', 'external')),
    raw_key TEXT,
    canonical_type TEXT NOT NULL,
    canonical_value_json TEXT,
    value_codec TEXT NOT NULL,
    seeded INTEGER NOT NULL CHECK (seeded IN (0, 1)),
    revision INTEGER NOT NULL CHECK (revision >= 0),
    origin TEXT NOT NULL CHECK (origin IN ('app_editor', 'instance', 'source_seed')),
    source_game_version TEXT,
    source_instance_id TEXT,
    updated_at INTEGER NOT NULL
);

CREATE TABLE synced_game_option_preferences (
    option_id TEXT PRIMARY KEY NOT NULL,
    enabled INTEGER NOT NULL CHECK (enabled IN (0, 1)),
    source TEXT NOT NULL CHECK (source IN ('catalog_default', 'discovery_default', 'user')),
    revision INTEGER NOT NULL CHECK (revision >= 0)
);

CREATE TABLE instance_game_option_pack_bases (
    instance_id TEXT PRIMARY KEY NOT NULL,
    pack_version_id TEXT,
    source TEXT NOT NULL CHECK (source IN ('client_overrides', 'overrides', 'none')),
    sha1 TEXT,
    encoding TEXT,
    document BLOB,
    FOREIGN KEY (instance_id) REFERENCES instances (id) ON DELETE CASCADE
);

CREATE TABLE instance_game_option_update_state (
    instance_id TEXT PRIMARY KEY NOT NULL,
    had_file INTEGER NOT NULL CHECK (had_file IN (0, 1)),
    sha1 TEXT,
    document BLOB,
    FOREIGN KEY (instance_id) REFERENCES instances (id) ON DELETE CASCADE
);

CREATE TABLE nyan_synced_options_bridge (
    singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
    source_instance_id TEXT,
    excluded_instances_json TEXT NOT NULL DEFAULT '[]',
    migrated INTEGER NOT NULL DEFAULT 0 CHECK (migrated IN (0, 1)),
    migrated_at INTEGER
);

INSERT INTO nyan_synced_options_bridge (singleton)
VALUES (1);

;
CREATE TABLE store_blobs (
	sha512 TEXT PRIMARY KEY NOT NULL,
	size INTEGER NOT NULL CHECK (size >= 0),
	status TEXT NOT NULL DEFAULT 'ready' CHECK (status IN ('ready', 'quarantined', 'deleting')),
	modified_as INTEGER NOT NULL,
	created_at INTEGER NOT NULL DEFAULT (unixepoch()),
	last_used_at INTEGER NOT NULL DEFAULT (unixepoch()),
	verified_at INTEGER NOT NULL DEFAULT (unixepoch()),
	sources TEXT NOT NULL DEFAULT '[]'
);

CREATE TABLE store_instance_files (
	file_id TEXT PRIMARY KEY NOT NULL REFERENCES instance_files(id) ON DELETE CASCADE,
	blob_sha512 TEXT NOT NULL REFERENCES store_blobs(sha512) ON DELETE RESTRICT,
	materialization_kind TEXT NOT NULL CHECK (materialization_kind IN ('reflink', 'hardlink', 'copy'))
);

CREATE INDEX store_instance_files_blob ON store_instance_files(blob_sha512);

CREATE TABLE store_retained_refs (
	owner_kind TEXT NOT NULL,
	owner_id TEXT NOT NULL,
	blob_sha512 TEXT NOT NULL REFERENCES store_blobs(sha512) ON DELETE RESTRICT,
	PRIMARY KEY (owner_kind, owner_id, blob_sha512)
);

CREATE TABLE store_operations (
	id TEXT PRIMARY KEY NOT NULL,
	instance_id TEXT NOT NULL,
	payload TEXT NOT NULL,
	created_at INTEGER NOT NULL DEFAULT (unixepoch())
);

CREATE INDEX store_operations_instance ON store_operations(instance_id);

INSERT INTO app_metadata (key, value, updated_at)
VALUES ('store_cache_limit_bytes', '5368709120', unixepoch())
ON CONFLICT(key) DO NOTHING;
