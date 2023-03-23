use common::{Function, export_plugin, PluginRegistrar, Value};

export_plugin!(register);

extern "Rust" fn register(registrar: &mut dyn PluginRegistrar) {
    registrar.register_function("random", Box::new(Add));
}

#[derive(Debug, Clone)]
pub struct Add;

impl Function for Add {
    fn call(
        &self,
        context: std::sync::Arc<dyn common::Context>,
        args: &[Value],
    ) -> Result<Vec<Value>, common::InvocationError> {
        context.interface1();
        let res = args.iter().map(|x| x.0).sum();
        Ok(vec![Value(res)])
    }
}

#[cfg(test)]
mod tests {
    use common::Context;

    use super::*;
    use std::sync::Arc;

    struct ContextImpl;

    impl Context for ContextImpl {
        fn interface1(&self) {}
    }

    #[test]
    fn it_works() {
        let random = Add;
        let context = ContextImpl;
        let args = vec![Value(1.0), Value(1.0)];
        let res = random.call(Arc::new(context), &args).unwrap();
        println!("result: {:?}", res);
    }
}
