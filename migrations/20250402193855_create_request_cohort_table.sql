-- Add request_cohort table
CREATE TABLE request_cohort (
	request_id INTEGER NOT NULL,
	cohort_id INTEGER NOT NULL,
	PRIMARY KEY (request_id, cohort_id),
	FOREIGN KEY (request_id) REFERENCES request(request_id) ON DELETE CASCADE,
	FOREIGN KEY (cohort_id) REFERENCES cohort(cohort_id) ON DELETE CASCADE
);
