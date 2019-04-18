CREATE SEQUENCE schemas_id_seq;
CREATE TABLE schemas (
  id BIGINT PRIMARY KEY DEFAULT nextval('schemas_id_seq'::regclass),
  fingerprint CHARACTER VARYING NOT NULL,
  json TEXT NOT NULL,
  created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
  updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
  fingerprint2 CHARACTER VARYING
);

CREATE UNIQUE INDEX index_schemas_on_fingerprint ON schemas(fingerprint);
CREATE UNIQUE INDEX index_schemas_on_fingerprint2 ON schemas(fingerprint2);

SELECT diesel_manage_updated_at('schemas');
