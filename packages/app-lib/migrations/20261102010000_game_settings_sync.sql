CREATE TABLE nyan_game_sync_config (
	id INTEGER PRIMARY KEY CHECK (id = 1),
	payload TEXT NOT NULL
);
CREATE TABLE nyan_game_sync_instances (
	instance_id TEXT PRIMARY KEY REFERENCES instances(id) ON DELETE CASCADE,
	payload TEXT NOT NULL
);
