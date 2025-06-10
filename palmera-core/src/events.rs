use axum::Router;
use lettre::SmtpTransport;

use crate::base::App;

// app events data

pub struct TerminateEvent {
    is_restart: bool,
}

pub struct BackupEvent {
    name: String,
    exclude: Vec<String>,
}

pub struct BootstrapEvent<'a> {
    pub router: &'a mut Router,
}

pub struct ServeEvent<'a> {
    pub router: &'a mut Router,
}

// mailer event

pub struct MailerEvent {
    mailer: SmtpTransport,
}
