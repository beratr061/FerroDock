CREATE TABLE application_mounts (
    application_id uuid NOT NULL REFERENCES applications (id) ON DELETE CASCADE,
    target text NOT NULL,
    kind text NOT NULL CHECK (kind IN ('volume', 'bind')),
    source text NOT NULL,
    read_only boolean NOT NULL DEFAULT false,
    PRIMARY KEY (application_id, target)
);

CREATE TABLE application_ports (
    application_id uuid NOT NULL REFERENCES applications (id) ON DELETE CASCADE,
    published_port integer NOT NULL CHECK (published_port BETWEEN 1 AND 65535),
    target_port integer NOT NULL CHECK (target_port BETWEEN 1 AND 65535),
    protocol text NOT NULL CHECK (protocol IN ('tcp', 'udp')),
    PRIMARY KEY (published_port, protocol)
);

CREATE INDEX application_ports_application ON application_ports (application_id);
