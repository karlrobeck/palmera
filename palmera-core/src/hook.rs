pub struct Handler<T> {
    func: Box<dyn Fn(&mut T) -> Result<(), anyhow::Error>>,
    id: String,
    priority: i16,
}

pub struct Hook<T> {
    handlers: Vec<Handler<T>>,
}

impl<T> Hook<T> {
    pub fn new() -> Self {
        Self { handlers: vec![] }
    }

    pub fn bind<F>(&mut self, callback: F)
    where
        F: Fn(&mut T) -> Result<(), anyhow::Error> + Send + 'static,
    {
        self.handlers.push(Handler {
            func: Box::new(callback),
            id: "".into(),
            priority: -1,
        });
    }

    pub fn length(&self) -> usize {
        self.handlers.len()
    }

    pub fn trigger(&mut self, value: &mut T) {
        for handler in &self.handlers {
            if let Err(err) = (handler.func)(value) {
                println!("{err}");
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::hook::Hook;

    struct FooStruct {
        bar: String,
    }

    #[test]
    fn test() {
        let mut hook: Hook<FooStruct> = Hook::new();

        hook.bind(|v: &mut FooStruct| {
            v.bar = "hello".into();
            Ok(())
        });

        let mut foo = FooStruct {
            bar: "hello world".into(),
        };
        _ = hook.trigger(&mut foo);

        assert_eq!(foo.bar, "hello");
    }
}
