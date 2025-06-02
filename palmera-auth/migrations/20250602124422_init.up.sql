-- Add up migration script here
create table auth_users (
  id uuid_text not null primary key,
  email email_text not null,
  password password_text not null,
  created datetime_text not null,
  updated datetime_text not null
)