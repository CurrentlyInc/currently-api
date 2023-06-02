-- Create the organization table
create table organization
(
    organization_id uuid primary key default uuid_generate_v1mc(),
    name            text not null unique,
    created_at      timestamptz not null default now(),
    updated_at      timestamptz
);

-- Create the site table
create table site
(
    site_id         uuid primary key default uuid_generate_v1mc(),
    user_id         uuid references "user" (user_id) on delete cascade,
    organization_id uuid references organization (organization_id) on delete cascade,
    name            text not null,
    created_at      timestamptz not null default now(),
    updated_at      timestamptz
);

-- Create a unique constraint for user_id and organization_id
alter table site add constraint site_user_org_unique unique (user_id, organization_id);

-- Apply the `updated_at` trigger
SELECT trigger_updated_at('organization');
SELECT trigger_updated_at('site');
