use std::{any::Any, collections::BTreeMap};

use axum::Router;
use tokio::net::TcpListener;

use crate::{
    events::{BackupEvent, MailerEvent, ServeEvent, TerminateEvent},
    hook::Hook,
};

pub struct App {
    pub store: BTreeMap<String, Box<dyn Any + Send + Sync>>,
    router: Router,
    // core events
    pub on_serve: Hook<ServeEvent<'static>>,
    pub on_terminate: Hook<TerminateEvent>,
    pub on_backup: Hook<BackupEvent>,
    // mail events
    pub on_mail_send: Hook<MailerEvent>,
}

impl App {
    pub fn new() -> Self {
        Self {
            store: BTreeMap::new(),
            router: Router::new(),
            on_serve: Hook::new(),
            on_terminate: Hook::new(),
            on_backup: Hook::new(),
            on_mail_send: Hook::new(),
        }
    }

    pub async fn start(&mut self) -> anyhow::Result<()> {
        // SAFETY: We are extending the lifetime to 'static for the router reference,
        // which is valid because self lives for the duration of App.
        let router_ptr: *mut Router = &mut self.router;
        let router_static: &'static mut Router = unsafe { &mut *router_ptr };

        self.on_serve
            .trigger(&mut ServeEvent {
                router: router_static,
            })
            .await;

        let listener = TcpListener::bind("0.0.0.0:3000").await?;

        axum::serve(listener, self.router.clone().into_make_service()).await?;

        Ok(())
    }
}
