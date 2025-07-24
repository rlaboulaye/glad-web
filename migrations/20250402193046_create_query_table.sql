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
	status TEXT NOT NULL CHECK(status IN ('pending', 'processing', 'completed', 'errored')) DEFAULT 'pending',
	status_updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
	FOREIGN KEY (user_id) REFERENCES user(user_id)
);
