CREATE SEQUENCE subjects_id_seq;
CREATE TABLE subjects (
  id BIGINT PRIMARY KEY DEFAULT nextval('subjects_id_seq'::regclass),
  name TEXT NOT NULL,
  created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
  updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL
);

CREATE UNIQUE INDEX index_subjects_on_name ON subjects(name);
SELECT diesel_manage_updated_at('subjects');
