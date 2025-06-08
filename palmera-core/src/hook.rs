use uuid::Uuid;

pub struct Handler<T> {
    func: Box<dyn Fn(&mut T) -> Result<(), anyhow::Error>>,
    id: Option<String>,
    priority: Option<i16>,
}

pub struct Hook<T> {
    handlers: Vec<Handler<T>>,
}

impl<T> Hook<T> {
    pub fn new() -> Self {
        Self { handlers: vec![] }
    }

    pub fn bind(&mut self, handler: Handler<T>) -> String {
        let mut handler = handler;

        if handler.id.is_none() {
            handler.id = Some(generate_hook_id());

            for existing in &self.handlers {
                if existing.id == handler.id {
                    handler.id = Some(generate_hook_id());
                }
            }
        }

        let id = handler.id.clone().unwrap();

        self.handlers.push(handler);

        self.handlers.sort_by_key(|h| h.priority.unwrap_or(0));

        return id;
    }

    pub fn bind_fn<F>(&mut self, callback: F) -> String
    where
        F: Fn(&mut T) -> Result<(), anyhow::Error> + Send + 'static,
    {
        self.bind(Handler {
            func: Box::new(callback),
            id: None,
            priority: None,
        })
    }

    pub fn unbind(&mut self, id: String) -> anyhow::Result<()> {
        let original_len = self.handlers.len();

        self.handlers
            .retain(|handler| handler.id.as_deref() != Some(&id));

        if self.handlers.len() == original_len {
            Err(anyhow::anyhow!("Handler with id {} not found", id))
        } else {
            Ok(())
        }
    }

    pub fn length(&self) -> usize {
        self.handlers.len()
    }

    pub fn trigger(&mut self, value: &mut T) -> Vec<anyhow::Error> {
        let mut errors = vec![];

        for handler in &self.handlers {
            if let Err(err) = (handler.func)(value) {
                errors.push(err);
            }
        }

        errors
    }
}

fn generate_hook_id() -> String {
    Uuid::new_v4().to_string()
}
