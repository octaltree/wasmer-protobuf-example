//#[no_mangle]
// pub fn score(a: String) -> String { a }

#[no_mangle]
pub unsafe fn alloc(len: usize) -> *mut u8 {
    let mut vec = Vec::<u8>::with_capacity(len);
    vec.set_len(len);
    Box::into_raw(vec.into_boxed_slice()) as *mut u8
}

#[no_mangle]
pub unsafe fn free(raw: *mut u8, len: usize) {
    let s = std::slice::from_raw_parts_mut(raw, len);
    Box::from_raw(s);
}

#[no_mangle]
pub fn foo(raw: *mut u8, len: usize) -> usize {
    let vec: Vec<u8> = unsafe { bytes(raw, len) };
    vec.into_iter().fold(0, |sum, i| sum + i) as usize
}

unsafe fn bytes(raw: *mut u8, len: usize) -> Vec<u8> {
    // let s = std::slice::from_raw_parts_mut(raw, len);
    // let b = Box::from_raw(s);
    // std::mem::transmute(b)
    Vec::from_raw_parts(raw, len, len)
}

// use tonic::{transport::Server, Request, Response, Status};

// mod score {
//    tonic::include_proto!("score");
//}

//#[tonic::async_trait]
// impl Greeter for MyGreeter {
//    async fn say_hello(
//        &self,
//        request: Request<HelloRequest>
//    ) -> Result<Response<HelloReply>, Status> {
//        println!("Got a request: {:?}", request);

//        let reply = hello_world::HelloReply {
//            message: format!("Hello {}!", request.into_inner().name).into()
//        };

//        Ok(Response::new(reply))
//    }
//}
