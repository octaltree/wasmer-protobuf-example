use prost::Message;

#[no_mangle]
pub unsafe fn alloc(len: usize) -> u64 {
    let (ptr, len) = alloc_impl(len);
    serialize_ptr(ptr, len)
}

#[no_mangle]
pub unsafe fn free(x: u64) {
    let (ptr, len) = deserialize_ptr(x);
    free_impl(ptr, len)
}

#[no_mangle]
pub fn score(x: u64) -> u64 {
    let candidates = {
        let (ptr, len) = unsafe { deserialize_ptr(x) };
        let vec: Vec<u8> = unsafe { to_bytes(ptr, len) };
        <score::Candidates as Message>::decode(&*vec).unwrap()
    };
    let scores = score::Scores {
        scores: candidates
            .candidates
            .into_iter()
            .map(|candidate| {
                // calc score of candidate with some other context
                candidate.value.chars().count() as i32
            })
            .collect()
    };
    {
        let mut buf = Vec::<u8>::with_capacity(scores.encoded_len());
        scores.encode(&mut buf).unwrap();
        let (ptr, len) = unsafe { from_bytes(buf) };
        unsafe { serialize_ptr(ptr, len) }
    }
}

unsafe fn serialize_ptr(offset: *mut u8, len: usize) -> u64 {
    std::mem::transmute((offset as u32, len))
}

unsafe fn deserialize_ptr(x: u64) -> (*mut u8, usize) {
    let (offset, len): (u32, u32) = std::mem::transmute(x);
    (offset as *mut u8, len as usize)
}

unsafe fn alloc_impl(len: usize) -> (*mut u8, usize) {
    let mut vec = Vec::<u8>::with_capacity(len);
    vec.set_len(len);
    from_bytes(vec)
}

unsafe fn free_impl(ptr: *mut u8, len: usize) {
    let s = std::slice::from_raw_parts_mut(ptr, len);
    Box::from_raw(s);
}

unsafe fn to_bytes(offset: *mut u8, len: usize) -> Vec<u8> { Vec::from_raw_parts(offset, len, len) }

unsafe fn from_bytes(vec: Vec<u8>) -> (*mut u8, usize) {
    let len = vec.len();
    let ptr = Box::into_raw(vec.into_boxed_slice()) as *mut u8;
    (ptr, len)
}

mod score {
    include!(concat!(env!("OUT_DIR"), "/score.rs"));
}
