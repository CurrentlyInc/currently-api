-- Create the sub_module table
create table slave_module
(
    slave_module_id      uuid primary key default uuid_generate_v1mc(),
    master_module_id  uuid references master_module (master_module_id) on delete cascade,
    name              text not null,
    created_at        timestamptz not null default now(),
    updated_at        timestamptz
);

SELECT trigger_updated_at('slave_module');
