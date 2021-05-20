#![feature(test)]
extern crate test;

use wasmer::{
    imports, wat2wasm, Array, Bytes, Instance, Module, NativeFunc, Pages, Store, WasmPtr
};
use wasmer_compiler_llvm::LLVM;
use wasmer_engine_jit::JIT;
// use wasmer_engine_native::Native;
use tonic::{transport::Server, Request, Response, Status};

mod score {
    tonic::include_proto!("score");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let instance = initialize()?;
    run(&instance)?;
    Ok(())
}

fn initialize() -> Result<Instance, Box<dyn std::error::Error>> {
    // Let's declare the Wasm module.
    //
    // We are using the text representation of the module here but you can also load `.wasm`
    // files using the `include_bytes!` macro.
    //    let wasm_bytes = wat2wasm(
    //        br#"
    //(module
    //  (type $add_one_t (func (param i32) (result i32)))
    //  (func $add_one_f (type $add_one_t) (param $value i32) (result i32)
    //    local.get $value
    //    i32.const 1
    //    i32.add)
    //  (export "add_one" (func $add_one_f)))
    //"#
    //    )?;
    let wasm_bytes = include_bytes!("../guest/target/wasm32-unknown-unknown/release/guest.wasm");

    // Create a Store.
    // Note that we don't need to specify the engine/compiler if we want to use
    // the default provided by Wasmer.
    // You can use `Store::default()` for that.
    let store = Store::new(&JIT::new(LLVM::default()).engine());

    println!("Compiling module...");
    // Let's compile the Wasm module.
    let module = Module::new(&store, wasm_bytes)?;

    // Create an empty import object.
    let import_object = imports! {};

    println!("Instantiating module...");
    // Let's instantiate the Wasm module.
    let instance = Instance::new(&module, &import_object)?;
    Ok(instance)
}

fn run(instance: &Instance) -> Result<(), Box<dyn std::error::Error>> {
    let alloc = instance
        .exports
        .get_function("alloc")?
        .native::<u32, WasmPtr<u8, Array>>()?;
    let free = instance
        .exports
        .get_function("free")?
        .native::<(u32, u32), ()>()?;
    let foo = instance
        .exports
        .get_function("foo")?
        .native::<(u32, u32), u32>()?;
    let memory = instance.exports.get_memory("memory")?;
    let ptr = alloc.call(4)?;
    let arr = unsafe { ptr.deref_mut(memory, 0, 4).unwrap() };
    for i in 0..4 {
        arr[i].set(i as u8);
    }
    let x = foo.call(ptr.offset(), 4)?;
    assert_eq!(x, 6);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::test::Bencher;

    #[test]
    fn test_exported_function() -> Result<(), Box<dyn std::error::Error>> { main() }

    #[bench]
    fn bench_main(b: &mut Bencher) { b.iter(|| main()); }

    #[bench]
    fn bench_run(b: &mut Bencher) {
        let instance = initialize().unwrap();
        b.iter(|| run(&instance).unwrap());
    }
}
