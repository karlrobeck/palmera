use std::future::Future;
use std::pin::Pin;
use uuid::Uuid;

// New HandlerFn with Higher-Ranked Trait Bound (HRTB)
pub type HandlerFn<T> = Box<
    dyn Fn(&T) -> Pin<Box<dyn Future<Output = anyhow::Result<T>> + Send>> + Send + Sync + 'static,
>;

pub struct Handler<T> {
    func: HandlerFn<T>,
    id: Option<String>,
    priority: Option<i16>,
}

pub struct Hook<T> {
    handlers: Vec<Handler<T>>,
}

impl<T: Send + 'static> Hook<T> {
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
        F: Fn(&T) -> Pin<Box<dyn Future<Output = anyhow::Result<T>> + Send>>
            + Send
            + Sync
            + 'static,
    {
        let func: HandlerFn<T> = Box::new(move |value: &T| Box::pin(callback(value)));
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

    pub fn listen(&self) {
        todo!("starts a tokio channel and return trigger")
    }

    pub async fn trigger(&mut self, value: &T) -> Vec<anyhow::Result<T>> {
        let mut errors = vec![];
        for handler in &self.handlers {
            errors.push((handler.func)(value).await);
        }
        errors
    }
}

fn generate_hook_id() -> String {
    Uuid::new_v4().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::future;
    use std::sync::{Arc, Mutex};

    #[tokio::test]
    async fn test_bind_and_trigger() {
        let counter = Arc::new(Mutex::new(0));
        let counter_clone = counter.clone();
        let mut hook = Hook::new();
        hook.bind_fn(move |_val: &i32| {
            let counter = counter_clone.clone();
            Box::pin(async move {
                let mut num = counter.lock().unwrap();
                *num += 1;
                Ok(*num)
            })
        });
        let results = hook.trigger(&10).await;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].as_ref().unwrap(), &1);
    }

    #[tokio::test]
    async fn test_unbind() {
        let mut hook = Hook::new();
        let id = hook.bind_fn(|_val: &i32| Box::pin(future::ready(Ok(1))));
        assert_eq!(hook.length(), 1);
        hook.unbind(id).unwrap();
        assert_eq!(hook.length(), 0);
    }

    #[tokio::test]
    async fn test_priority_order() {
        let mut hook = Hook::new();
        let order_ref = Arc::new(Mutex::new(Vec::<i32>::new()));
        let order1 = order_ref.clone();
        let order2 = order_ref.clone();
        let order3 = order_ref.clone();
        // Handler with priority 2
        let handler1 = Handler {
            func: Box::new(move |_| {
                let order = order1.clone();
                Box::pin(async move {
                    order.lock().unwrap().push(2);
                    Ok(2)
                })
            }),
            id: None,
            priority: Some(2),
        };
        // Handler with priority 1
        let handler2 = Handler {
            func: Box::new(move |_| {
                let order = order2.clone();
                Box::pin(async move {
                    order.lock().unwrap().push(1);
                    Ok(1)
                })
            }),
            id: None,
            priority: Some(1),
        };
        // Handler with priority 3
        let handler3 = Handler {
            func: Box::new(move |_| {
                let order = order3.clone();
                Box::pin(async move {
                    order.lock().unwrap().push(3);
                    Ok(3)
                })
            }),
            id: None,
            priority: Some(3),
        };
        hook.bind(handler1);
        hook.bind(handler2);
        hook.bind(handler3);
        let _ = hook.trigger(&0).await;
        let order = order_ref.lock().unwrap().clone();
        assert_eq!(order, vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn test_unbind_nonexistent() {
        let mut hook = Hook::<i32>::new();
        let result = hook.unbind("nonexistent".to_string());
        assert!(result.is_err());
    }
}
