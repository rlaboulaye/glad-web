-- Add request table
CREATE TABLE request (
	request_id INTEGER PRIMARY KEY AUTOINCREMENT,
	user_id INTEGER NOT NULL,
	file_path TEXT NOT NULL,
	created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
	status TEXT NOT NULL CHECK(status IN ('pending', 'processing', 'completed', 'errored')) DEFAULT 'pending',
	status_updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
	FOREIGN KEY (user_id) REFERENCES user(user_id)
);
