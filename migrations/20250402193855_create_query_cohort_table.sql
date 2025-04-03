-- Add query_cohort table
CREATE TABLE query_cohort (
	query_id INTEGER NOT NULL,
	cohort_id INTEGER NOT NULL,
	PRIMARY KEY (query_id, cohort_id),
	FOREIGN KEY (query_id) REFERENCES query(query_id) ON DELETE CASCADE,
	FOREIGN KEY (cohort_id) REFERENCES cohort(cohort_id) ON DELETE CASCADE
);
