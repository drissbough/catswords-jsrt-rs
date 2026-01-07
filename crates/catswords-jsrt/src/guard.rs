use crate::runtime::Runtime;
use catswords_jsrt_sys as sys;

pub struct Guard<'rt> {
    pub(crate) prev: sys::JsContextRef,
    pub(crate) current: sys::JsContextRef,
    pub(crate) runtime: &'rt Runtime,
    pub(crate) _marker: std::marker::PhantomData<&'rt ()>,
}

impl<'rt> Guard<'rt> {
    pub fn context_raw(&self) -> sys::JsContextRef {
        self.current
    }

    pub fn runtime(&self) -> &'rt Runtime {
        self.runtime
    }
}

impl Drop for Guard<'_> {
    fn drop(&mut self) {
        unsafe {
            let _ = sys::JsSetCurrentContext(self.prev);
        }
    }
}
