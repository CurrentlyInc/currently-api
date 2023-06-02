-- Create the master_module table
create table master_module
(
    master_module_id uuid primary key default uuid_generate_v1mc(),
    site_id          uuid references site (site_id) on delete cascade,
    name             text not null,
    created_at       timestamptz not null default now(),
    updated_at       timestamptz
);

SELECT trigger_updated_at('master_module');
