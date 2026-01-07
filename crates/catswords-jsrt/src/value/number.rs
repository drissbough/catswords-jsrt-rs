use crate::error::ok;
use crate::guard::Guard;
use crate::value::Value;
use catswords_jsrt_sys as sys;

pub struct Number {
    v: Value,
}

impl Number {
    pub fn new(_guard: &Guard<'_>, n: i32) -> Self {
        let mut out: sys::JsValueRef = std::ptr::null_mut();
        unsafe {
            // If this fails, it'll be a null pointer; for a minimal sample we keep it simple.
            let _ = ok(sys::JsIntToNumber(n, &mut out));
        }
        Self { v: Value { raw: out } }
    }

    pub fn into(self) -> Value {
        self.v
    }
}
