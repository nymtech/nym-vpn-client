CREATE TABLE blocked_domains(
    domain_name TEXT NOT NULL,
    source TEXT NOT NULL,
    updated_at INTEGER NOT NULL
);
CREATE UNIQUE INDEX domain_name_source_idx ON blocked_domains (domain_name, source);
CREATE INDEX source_updated_at_idx ON blocked_domains (source, updated_at);
CREATE INDEX domain_name_idx ON blocked_domains (domain_name);
