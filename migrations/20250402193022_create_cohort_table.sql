-- Add cohort table
CREATE TABLE cohort (
	cohort_id INTEGER PRIMARY KEY AUTOINCREMENT,
	cohort_name TEXT COLLATE NOCASE NOT NULL UNIQUE
);

INSERT INTO cohort (cohort_name) VALUES ("1000Genomes");
INSERT INTO cohort (cohort_name) VALUES ("EGAS00001001245");
INSERT INTO cohort (cohort_name) VALUES ("LARGE_PD");
INSERT INTO cohort (cohort_name) VALUES ("Peruvian_Genome_Project");
INSERT INTO cohort (cohort_name) VALUES ("phs000101");
INSERT INTO cohort (cohort_name) VALUES ("phs000237");
INSERT INTO cohort (cohort_name) VALUES ("phs000306");
INSERT INTO cohort (cohort_name) VALUES ("phs000360");
INSERT INTO cohort (cohort_name) VALUES ("phs000388");
INSERT INTO cohort (cohort_name) VALUES ("phs000517");
INSERT INTO cohort (cohort_name) VALUES ("phs000678");
INSERT INTO cohort (cohort_name) VALUES ("phs000864");
INSERT INTO cohort (cohort_name) VALUES ("phs000925");
INSERT INTO cohort (cohort_name) VALUES ("phs000988");
INSERT INTO cohort (cohort_name) VALUES ("phs001033");
INSERT INTO cohort (cohort_name) VALUES ("phs001039");
INSERT INTO cohort (cohort_name) VALUES ("phs001057");
INSERT INTO cohort (cohort_name) VALUES ("phs001193");
INSERT INTO cohort (cohort_name) VALUES ("phs001215");
INSERT INTO cohort (cohort_name) VALUES ("phs001237");
INSERT INTO cohort (cohort_name) VALUES ("phs001385");
INSERT INTO cohort (cohort_name) VALUES ("phs001388");
INSERT INTO cohort (cohort_name) VALUES ("phs001395");
INSERT INTO cohort (cohort_name) VALUES ("phs001415");
INSERT INTO cohort (cohort_name) VALUES ("phs001416");
INSERT INTO cohort (cohort_name) VALUES ("phs001446");
INSERT INTO cohort (cohort_name) VALUES ("phs001468");
INSERT INTO cohort (cohort_name) VALUES ("phs001515");
INSERT INTO cohort (cohort_name) VALUES ("phs001584");
INSERT INTO cohort (cohort_name) VALUES ("phs001599");
INSERT INTO cohort (cohort_name) VALUES ("phs001603");
INSERT INTO cohort (cohort_name) VALUES ("phs001604");
INSERT INTO cohort (cohort_name) VALUES ("phs001644");
INSERT INTO cohort (cohort_name) VALUES ("phs001662");
INSERT INTO cohort (cohort_name) VALUES ("phs001726");
INSERT INTO cohort (cohort_name) VALUES ("phs001810");
INSERT INTO cohort (cohort_name) VALUES ("phs002025");
