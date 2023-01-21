use anyhow::Result;
use wasmer_wasi::WasiState;
use wasmer::{Instance, Module, Store, Memory, AsStoreRef, MemoryView, FunctionEnvMut, WasmPtr, FunctionEnv, Function};

// Utils
pub fn read_string(view: &MemoryView, start: u32, len: u32) -> Result<String> {
    Ok(WasmPtr::<u8>::new(start).read_utf8_string(view, len)?)
}

// Environment
pub struct ExampleEnv {
    memory: Option<Memory>,
}

impl ExampleEnv {
    pub fn set_memory(&mut self, memory: Memory) {
        self.memory = Some(memory);
    }

    fn view<'a>(&'a self, store: &'a impl AsStoreRef) -> MemoryView<'a> {
        self.memory.as_ref().unwrap().view(store)
    }
}

fn http_get(ctx: FunctionEnvMut<ExampleEnv>, url: u32, url_len: u32) -> u32 {
    // Setup environment
    let env = ctx.data();
    let view = env.view(&ctx);

    // Read url from memory
    let address = read_string(&view, url, url_len).unwrap();
    let response = ureq::get(&address).call().unwrap().into_string().unwrap();

    // Write response as string [ptr, cap, len] to wasm memory and return pointer
    view.write(1024, &u32::to_le_bytes(1036)).unwrap();
    view.write(1028, &u32::to_le_bytes(response.len() as u32)).unwrap();
    view.write(1032, &u32::to_le_bytes(response.len() as u32)).unwrap();
    view.write(1036, response.as_bytes()).unwrap();
    1024
}

fn main() -> Result<()> {
    // Load module
    let module_bytes = include_bytes!("../demo.wasm");
    let mut store = Store::default();
    let module = Module::new(&store, &module_bytes)?;

    // Initialize wasi
    let wasi_env = WasiState::new("example").finalize(&mut store)?;
    let mut import_object = wasi_env.import_object(&mut store, &module)?;

    // Add host functions
    let function_env = FunctionEnv::new(&mut store, ExampleEnv { memory: None });
    import_object.define(
        "env",
        "http_get",
        Function::new_typed_with_env(&mut store, &function_env, http_get),
    );

    // Create instance
    let instance = Instance::new(&mut store, &module, &import_object)?;
    let memory = instance.exports.get_memory("memory")?;

    // Give reference to memory
    wasi_env.data_mut(&mut store).set_memory(memory.clone());
    function_env.as_mut(&mut store).set_memory(memory.clone());

    // Call function
    let wasm = instance.exports.get_function("main")?;
    wasm.call(&mut store, &[])?;

    Ok(())
}
