CREATE SEQUENCE configs_id_seq;
CREATE TABLE configs (
  id BIGINT PRIMARY KEY DEFAULT nextval('configs_id_seq'::regclass),
  compatibility CHARACTER VARYING,
  created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
  updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
  subject_id BIGINT
);

CREATE UNIQUE INDEX index_configs_on_subject_id ON configs(subject_id);

SELECT diesel_manage_updated_at('configs');
