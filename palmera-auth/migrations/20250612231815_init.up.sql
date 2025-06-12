-- Add up migration script here
create schema auth;

create table auth.users (
  id uuid not null primary key default gen_random_uuid(),
  email text unique not null,
  password text not null,
  created timestamptz not null default now(),
  updated timestamptz not null default now()
);