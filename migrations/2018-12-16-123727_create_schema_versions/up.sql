CREATE SEQUENCE schema_versions_id_seq;
CREATE TABLE schema_versions (
  id BIGINT PRIMARY KEY DEFAULT nextval('schema_versions_id_seq'::regclass),
  version INTEGER DEFAULT 1,
  subject_id BIGINT NOT NULL,
  schema_id BIGINT NOT NULL
);

CREATE UNIQUE INDEX index_schema_versions_on_subject_id_and_version ON schema_versions(subject_id, version);
