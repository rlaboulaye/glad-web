-- Add query table
CREATE TABLE query (
	query_id INTEGER PRIMARY KEY AUTOINCREMENT,
	user_id INTEGER NOT NULL,
	title TEXT NOT NULL CHECK(LENGTH(title) >= 4 AND LENGTH(title) <= 100),
	description TEXT DEFAULT '',
	file_path TEXT NOT NULL,
	self_described_latino INTEGER NOT NULL,
	n_controls INTEGER NOT NULL DEFAULT 100,
	created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
	user_visible_status TEXT NOT NULL CHECK(user_visible_status IN ('pending', 'processing', 'completed', 'failed')) DEFAULT 'pending',
	internal_status TEXT NOT NULL CHECK(internal_status IN ('pending', 'processing', 'retry_pending', 'failed_permanent', 'completed')) DEFAULT 'pending',
	status_updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
	retry_count INTEGER NOT NULL DEFAULT 0,
	last_error_message TEXT DEFAULT NULL,
	result_file_path TEXT DEFAULT NULL,
	FOREIGN KEY (user_id) REFERENCES user(user_id)
);
