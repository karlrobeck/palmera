use std::future::Future;
use std::pin::Pin;
use uuid::Uuid;

// New HandlerFn with Higher-Ranked Trait Bound (HRTB)
pub type HandlerFn<T> = Box<
    dyn for<'a> Fn(&'a mut T) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>>
        + Send
        + Sync
        + 'static,
>;

pub struct Handler<T> {
    func: HandlerFn<T>,
    id: Option<String>,
    priority: Option<i16>,
}

pub struct Hook<T> {
    handlers: Vec<Handler<T>>,
}

impl<T: Send> Hook<T> {
    // T must be Send if you want to use it across awaits
    pub fn new() -> Self {
        Self { handlers: vec![] }
    }

    pub fn bind(&mut self, handler: Handler<T>) -> String {
        let mut handler = handler;
        if handler.id.is_none() {
            handler.id = Some(generate_hook_id());
        }
        let id = handler.id.clone().unwrap();
        self.handlers.push(handler);
        self.handlers.sort_by_key(|h| h.priority.unwrap_or(0));
        id
    }

    // New bind_fn signature
    pub fn bind_fn<F>(&mut self, callback: F) -> String
    where
        F: for<'a> Fn(&'a mut T) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>>
            + Send
            + Sync
            + 'static,
    {
        let func: HandlerFn<T> = Box::new(move |value: &mut T| Box::pin(callback(value)));
        self.bind(Handler {
            func,
            id: None,
            priority: None,
        })
    }

    // Unchanged methods
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

    pub async fn trigger(&mut self, value: &mut T) -> Vec<anyhow::Error> {
        let mut errors = vec![];
        for handler in &self.handlers {
            if let Err(err) = (handler.func)(value).await {
                errors.push(err);
            }
        }
        errors
    }
}

fn generate_hook_id() -> String {
    Uuid::new_v4().to_string()
}

#[cfg(test)]
mod test {

    use crate::hook::{Handler, Hook};
    use anyhow::Result;

    async fn dummy_handler(val: &mut i32) -> Result<()> {
        *val += 1;
        Ok(())
    }

    #[tokio::test]
    async fn test_bind_and_trigger() {
        let mut hook = Hook::<i32>::new();
        let id = hook.bind_fn(|val| Box::pin(dummy_handler(val)));
        let mut value = 0;
        let errors = hook.trigger(&mut value).await;
        assert!(errors.is_empty());
        assert_eq!(value, 1);
        assert_eq!(hook.length(), 1);
        assert!(!id.is_empty());
    }

    #[tokio::test]
    async fn test_unbind() {
        let mut hook = Hook::<i32>::new();
        let id = hook.bind_fn(|val| Box::pin(dummy_handler(val)));
        assert_eq!(hook.length(), 1);
        hook.unbind(id.clone()).unwrap();
        assert_eq!(hook.length(), 0);
        // Unbinding again should error
        assert!(hook.unbind(id).is_err());
    }

    #[tokio::test]
    async fn test_priority_order() {
        let mut hook = Hook::<i32>::new();
        // Handler that adds 10, with higher priority (lower number)
        let handler1: Handler<i32> = Handler {
            func: Box::new(|val| {
                Box::pin(async move {
                    *val += 10;
                    Ok(())
                })
            }),
            id: None,
            priority: Some(-10),
        };
        // Handler that multiplies by 2, default priority
        let handler2: Handler<i32> = Handler {
            func: Box::new(|val| {
                Box::pin(async move {
                    *val *= 2;
                    Ok(())
                })
            }),
            id: None,
            priority: None,
        };
        hook.bind(handler2);
        hook.bind(handler1);
        let mut value = 1;
        let _ = hook.trigger(&mut value).await;
        // handler1 runs first: 1+10=11, then handler2: 11*2=22
        assert_eq!(value, 22);
    }

    #[tokio::test]
    async fn test_handler_error() {
        let mut hook = Hook::<i32>::new();
        // Handler that always errors
        let handler: Handler<i32> = Handler {
            func: Box::new(|_| Box::pin(async move { Err(anyhow::anyhow!("fail")) })),
            id: None,
            priority: None,
        };
        hook.bind(handler);
        let mut value = 0;
        let errors = hook.trigger(&mut value).await;
        assert_eq!(errors.len(), 1);
        assert_eq!(hook.length(), 1);
    }
}
