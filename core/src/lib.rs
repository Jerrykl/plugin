use common::{Context, Function, InvocationError, PluginDeclaration, PluginRegistrar, Value};
use libloading::Library;
// use std::any::Any;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::io;
use std::sync::{Arc, RwLock};

pub struct FunctionProxy {
    function: Box<dyn Function>,
    _lib: Arc<Library>,
}

impl Function for FunctionProxy {
    fn call(
        &self,
        context: Arc<dyn common::Context>,
        args: &[Value],
    ) -> Result<Vec<Value>, InvocationError> {
        self.function.call(context, args)
    }

    fn help(&self) -> Option<&str> {
        self.function.help()
    }
}

pub struct PluginManager {
    functions: RwLock<HashMap<String, Arc<FunctionProxy>>>,
}

impl PluginManager {
    /// Create a new plugin manager.
    pub fn new() -> PluginManager {
        PluginManager {
            functions: Default::default(),
        }
    }

    /// Call a specific plugin function.
    pub fn call(
        &self,
        context: Arc<dyn Context>,
        function: &str,
        args: &[Value],
    ) -> Result<Vec<Value>, InvocationError> {
        self.functions
            .read()
            .unwrap()
            .get(function)
            .ok_or_else(|| format!("\"{}\" not found", function))?
            .call(context, args)
    }

    /// Load a plugin library and add all contained functions to the internal
    /// function table.
    ///
    /// # Safety
    ///
    /// A plugin library **must** be implemented using the
    /// [`plugins_core::plugin_declaration!()`] macro. Trying manually implement
    /// a plugin without going through that macro will result in undefined
    /// behaviour.
    pub unsafe fn load<P: AsRef<OsStr>>(&mut self, library_path: P) -> io::Result<()> {
        // load the library into memory
        let library = Arc::new(Library::new(library_path)?);

        // get a pointer to the plugin_declaration symbol.
        let decl = library
            .get::<*mut PluginDeclaration>(b"plugin_declaration\0")?
            .read();

        let mut registrar = PluginRegistrarImpl::new(Arc::clone(&library));

        (decl.register)(&mut registrar);

        // add all loaded plugins to the functions map
        self.functions.write().unwrap().extend(registrar.functions);
        // and make sure ExternalFunctions keeps a reference to the library
        // self.libraries.push(library);

        Ok(())
    }

    /// Unload all plugin libraries and all contained functions.
    pub fn unload(&self) {
        self.functions.write().unwrap().clear();
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

struct PluginRegistrarImpl {
    functions: HashMap<String, Arc<FunctionProxy>>,
    lib: Arc<Library>,
}

impl PluginRegistrarImpl {
    fn new(lib: Arc<Library>) -> PluginRegistrarImpl {
        PluginRegistrarImpl {
            lib,
            functions: HashMap::default(),
        }
    }
}

impl PluginRegistrar for PluginRegistrarImpl {
    fn register_function(&mut self, name: &str, function: Box<dyn Function>) {
        let proxy = FunctionProxy {
            function,
            _lib: Arc::clone(&self.lib),
        };
        self.functions.insert(name.to_string(), proxy.into());
    }
}

mod tests {
    use super::*;
    struct ContextImpl;

    impl Context for ContextImpl {
        fn interface1(&self) {}
    }

    #[test]
    fn test() {
        let context = Arc::new(ContextImpl);
        let plugin_library = "/Users/astolfo/Downloads/plugin/target/debug/librandom.dylib";
        let function = "random";
        let arguments = vec![Value(1.0), Value(2.0)];

        // create our functions table and load the plugin
        let mut plguin_manager = PluginManager::new();

        unsafe {
            plguin_manager
                .load(plugin_library)
                .expect("Function loading failed");
        }

        // then call the function
        let result = plguin_manager
            .call(context, function, &arguments)
            .expect("Invocation failed");

        // print out the result
        println!(
            "{}({}) = {}",
            function,
            arguments
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", "),
            result
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", "),
        );
    }
}
