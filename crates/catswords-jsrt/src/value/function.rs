use crate::error::{ok, Result};
use crate::guard::Guard;
use crate::runtime::Runtime;
use crate::value::Value;
use catswords_jsrt_sys as sys;
use std::ffi::c_void;

pub struct CallInfo {
    pub arguments: Vec<Value>,
}

type Callback = dyn Fn(&Guard<'_>, CallInfo) -> Result<Value> + Send + Sync + 'static;

// NEW: callback_state holds both runtime and callback.
struct CallbackState {
    runtime: *const Runtime,
    cb: Box<Callback>,
}

pub struct Function {
    v: Value,
}

impl Function {
    pub fn new(guard: &Guard<'_>, cb: Box<Callback>) -> Self {
        // Allocate callback state (thin pointer)
        let state = Box::new(CallbackState {
            runtime: guard.runtime() as *const Runtime,
            cb,
        });
        let state_ptr = Box::into_raw(state) as *mut c_void;

        // Runtime owns callback_state lifetime (freed after JsDisposeRuntime)
        guard.runtime().register_callback_state(state_ptr);

        let mut func: sys::JsValueRef = std::ptr::null_mut();
        unsafe {
            let _ = ok(sys::JsCreateFunction(Some(native_trampoline), state_ptr, &mut func));
        }

        Self { v: Value { raw: func } }
    }

    pub fn call(&self, _guard: &Guard<'_>, args: &[&Value]) -> Result<Value> {
        // ChakraCore requires argv[0] = thisArg. We'll use the function itself as thisArg.
        let mut argv: Vec<sys::JsValueRef> = Vec::with_capacity(args.len() + 1);
        argv.push(self.v.raw);
        for a in args {
            argv.push(a.raw);
        }

        let mut out: sys::JsValueRef = std::ptr::null_mut();
        unsafe { ok(sys::JsCallFunction(self.v.raw, argv.as_ptr(), argv.len() as u16, &mut out))?; }
        Ok(Value { raw: out })
    }

    pub fn into(self) -> Value {
        self.v
    }
}

impl Drop for Function {
    fn drop(&mut self) {
        // Do NOT free callback_state here.
        // Runtime will free all callback states after JsDisposeRuntime.
    }
}

unsafe extern "C" fn native_trampoline(
    _callee: sys::JsValueRef,
    _is_construct_call: bool,
    arguments: *const sys::JsValueRef,
    argument_count: u16,
    callback_state: *mut c_void,
) -> sys::JsValueRef {
    // Cast back to CallbackState
    let st = &*(callback_state as *const CallbackState);
    let cb: &Callback = &*st.cb;

    // Current context (assume host already set current context)
    let mut current: sys::JsContextRef = std::ptr::null_mut();
    let _ = sys::JsGetCurrentContext(&mut current);

    // Build Guard that includes runtime reference
    let guard = Guard {
        prev: current,
        current,
        runtime: &*st.runtime,
        _marker: std::marker::PhantomData,
    };

    // Copy args
    let mut argv: Vec<Value> = Vec::with_capacity(argument_count as usize);
    if !arguments.is_null() {
        let slice = std::slice::from_raw_parts(arguments, argument_count as usize);
        for &a in slice {
            argv.push(Value { raw: a });
        }
    }

    // ChakraCore passes thisArg at argv[0]. The user's closure expects only user args.
    let user_args = if argv.len() >= 2 { argv[1..].to_vec() } else { Vec::new() };
    let info = CallInfo { arguments: user_args };

    match cb(&guard, info) {
        Ok(v) => v.raw,
        Err(e) => {
            let msg = format!("{}", e);

            if let Ok(js_err) = Value::error_from_message(&guard, &msg) {
                let _ = sys::JsSetException(js_err.raw);
                return js_err.raw;
            }

            // Fallback to undefined if error creation fails
            let mut undef = std::ptr::null_mut();
            let _ = sys::JsGetUndefinedValue(&mut undef);
            let _ = sys::JsSetException(undef);
            undef
        }
    }
}

pub(crate) unsafe fn free_callback_state(p: *mut c_void) {
    // p was created from Box::into_raw(Box<CallbackState>) as *mut c_void
    let _state: Box<CallbackState> = Box::from_raw(p as *mut CallbackState);
    // drop happens here
}
