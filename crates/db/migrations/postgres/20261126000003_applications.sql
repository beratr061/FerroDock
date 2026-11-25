CREATE TABLE applications (
    id uuid PRIMARY KEY,
    environment_id uuid NOT NULL REFERENCES environments (id) ON DELETE CASCADE,
    name text NOT NULL,
    slug text NOT NULL,
    description text NOT NULL DEFAULT '',
    source_kind text NOT NULL CHECK (source_kind IN ('git', 'image')),
    git_url text,
    git_branch text,
    build_kind text CHECK (build_kind IN ('dockerfile', 'compose')),
    build_path text,
    build_context text,
    image text,
    deploy_key_public text,
    deploy_key_private_enc text,
    strategy text NOT NULL DEFAULT 'recreate' CHECK (strategy IN ('recreate', 'blue_green')),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (environment_id, slug),
    CHECK (
        (source_kind = 'git' AND git_url IS NOT NULL AND git_branch IS NOT NULL
            AND build_kind IS NOT NULL AND build_path IS NOT NULL
            AND build_context IS NOT NULL AND image IS NULL)
        OR (source_kind = 'image' AND image IS NOT NULL AND git_url IS NULL)
    ),
    CHECK ((deploy_key_public IS NULL) = (deploy_key_private_enc IS NULL))
);
