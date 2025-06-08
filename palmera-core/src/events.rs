#[derive(Debug, Clone)]
pub enum Event {
    OnBootstrap(String),
    OnServe,
}

pub struct EventHandler {
    handlers: Vec<Box<dyn Fn(&Event) -> Result<(), anyhow::Error>>>,
}

impl EventHandler {
    pub fn new() -> Self {
        Self { handlers: vec![] }
    }

    pub fn bind<F>(&mut self, callback: F)
    where
        F: Fn(&Event) -> Result<(), anyhow::Error> + 'static,
    {
        self.handlers.push(Box::new(callback));
    }

    pub fn trigger(&mut self, event: Event) {
        for handler in &self.handlers {
            if let Err(err) = handler(&event) {
                println!("{err}");
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::events::{Event, EventHandler};

    #[test]
    fn test() {
        let mut dispatcher = EventHandler::new();
        dispatcher.bind(foo_callback);
        dispatcher.bind(foo_callback2);
        dispatcher.trigger(Event::OnBootstrap("i just got triggered!!".into()));
    }

    fn foo_callback(event: &Event) -> Result<(), anyhow::Error> {
        if let Event::OnBootstrap(value) = event {
            println!("{value}");
        }
        Ok(())
    }

    fn foo_callback2(event: &Event) -> Result<(), anyhow::Error> {
        if let Event::OnBootstrap(value) = event {
            println!("{value}");
        }
        Ok(())
    }
}
