#![feature(test)]
extern crate test;

use wasmer::{
    imports, wat2wasm, Array, Bytes, Instance, Memory, Module, NativeFunc, Pages, Store, WasmPtr
};
use wasmer_compiler_llvm::LLVM;
use wasmer_engine_jit::JIT;
// use wasmer_engine_native::Native;
use bytes::buf::{Buf, BufMut};
use core::cell::Cell;
use prost::{bytes::buf::UninitSlice, Message};
use tonic::{transport::Server, Request, Response, Status};

mod score {
    tonic::include_proto!("score");
}

const N: i32 = 300000;

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

// fn serialize_ptr(offset: WasmPtr<u8>, len: u32) -> u64 {
//    let offset = offset.offset();
//    unsafe { std::mem::transmute((offset, len)) }
//}

fn deserialize_ptr(x: u64) -> (WasmPtr<u8, Array>, u32) {
    let (offset, len): (u32, u32) = unsafe { std::mem::transmute(x) };
    (WasmPtr::<u8, Array>::new(offset), len)
}

fn write<M>(
    memory: &Memory,
    ptr: WasmPtr<u8, Array>,
    len: u32,
    msg: &M
) -> Result<(), prost::EncodeError>
where
    M: prost::Message + Sized
{
    let arr = unsafe { ptr.deref_mut(memory, 0, len).unwrap() };
    msg.encode(&mut Wrapper::new(arr))
}

fn read<M>(memory: &Memory, ptr: WasmPtr<u8, Array>, len: u32) -> Result<M, prost::DecodeError>
where
    M: prost::Message + Default
{
    let arr = unsafe { ptr.deref(memory, 0, len).unwrap() };
    <M as prost::Message>::decode(Wrapper::new(arr))
}

struct Wrapper<'a> {
    buf: &'a [Cell<u8>],
    cur: usize
}

impl<'a> Wrapper<'a> {
    fn new(buf: &'a [Cell<u8>]) -> Self { Self { buf, cur: 0 } }
}

unsafe impl<'a> BufMut for Wrapper<'a> {
    fn chunk_mut(&mut self) -> &mut UninitSlice {
        let cell = &self.buf[self.cur];
        let ptr = cell.as_ptr();
        let len = self.remaining_mut();
        unsafe { UninitSlice::from_raw_parts_mut(ptr, len) }
    }

    unsafe fn advance_mut(&mut self, cnt: usize) { self.cur += cnt; }

    fn remaining_mut(&self) -> usize { self.buf.len() - self.cur }
}

impl<'a> Buf for Wrapper<'a> {
    fn remaining(&self) -> usize { self.buf.len() - self.cur }

    fn chunk(&self) -> &[u8] {
        let cell = &self.buf[self.cur];
        let ptr = cell.as_ptr();
        let len = self.remaining_mut();
        unsafe { std::slice::from_raw_parts(ptr, len) }
    }

    fn advance(&mut self, cnt: usize) { self.cur += cnt; }
}

fn run(instance: &Instance) -> Result<(), Box<dyn std::error::Error>> {
    // let free = instance
    //    .exports
    //    .get_function("free")?
    //    .native::<(u32, u32), ()>()?;
    let alloc = instance
        .exports
        .get_function("alloc")?
        .native::<u32, u64>()?;
    let score = instance
        .exports
        .get_function("score")?
        .native::<u64, u64>()?;
    let memory = instance.exports.get_memory("memory")?;
    let candidates = score::Candidates {
        candidates: (1..=N)
            .map(|v| score::Candidate {
                value: v.to_string()
            })
            .collect()
    };
    let offsetlen = alloc.call(candidates.encoded_len() as u32)?;
    let (ptr, len) = deserialize_ptr(offsetlen);
    write(&memory, ptr, len, &candidates)?;
    let offsetlen = score.call(offsetlen)?;
    let (ptr, len) = deserialize_ptr(offsetlen);
    let _scores = read::<score::Scores>(&memory, ptr, len)?;
    // XXX: I have to `free` scores
    // assert_eq!(
    //    score::Scores {
    //        scores: vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 2]
    //    },
    //    scores
    //);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::test::Bencher;

    //#[test]
    // fn test_exported_function() -> Result<(), Box<dyn std::error::Error>> { main() }

    //#[bench]
    // fn bench_main(b: &mut Bencher) { b.iter(|| main()); }

    #[bench]
    fn bench_run(b: &mut Bencher) {
        let instance = initialize().unwrap();
        b.iter(|| run(&instance).unwrap());
    }

    #[bench]
    fn bench_native(b: &mut Bencher) {
        b.iter(|| {
            let candidates = score::Candidates {
                candidates: (1..=N)
                    .map(|v| score::Candidate {
                        value: v.to_string()
                    })
                    .collect()
            };
            let _scores = score::Scores {
                scores: candidates
                    .candidates
                    .into_iter()
                    .map(|c| c.value.chars().count() as i32)
                    .collect()
            };
        });
    }

    #[bench]
    fn bench_native2(b: &mut Bencher) {
        b.iter(|| {
            (1..=N)
                .map(|v| v.to_string())
                .map(|s| s.len() as i32)
                .for_each(|_| {});
        });
    }
}
