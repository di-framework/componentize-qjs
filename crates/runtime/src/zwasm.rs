//! wasm-c-api declarations for the zwasm interpreter archive.
//!
//! Layout matches the calls wasi-wit-zwasm already runs against this archive:
//! `wasm_func_new` copies the functype, `wasm_functype_new` takes ownership of
//! valtype vectors allocated by `wasm_valtype_vec_new`, and the host callback
//! is `fn(*const wasm_val_vec_t, *mut wasm_val_vec_t) -> *mut wasm_trap_t`.
//! `wasm_memory_grow` in this build returns `bool`. Guest `memory.grow` still
//! returns the previous page count.

#![allow(non_camel_case_types, non_snake_case, dead_code)]

use std::cell::RefCell;
use std::os::raw::c_void;
use std::ptr;

use rquickjs::function::Args;
use rquickjs::{CaughtError, Ctx, Function, IntoJs, Persistent, Value};

pub(crate) enum wasm_engine_t {}
pub(crate) enum wasm_store_t {}
pub(crate) enum wasm_module_t {}
pub(crate) enum wasm_instance_t {}
pub(crate) enum wasm_func_t {}
pub(crate) enum wasm_extern_t {}
pub(crate) enum wasm_trap_t {}
pub(crate) enum wasm_memory_t {}
pub(crate) enum wasm_functype_t {}
pub(crate) enum wasm_valtype_t {}
pub(crate) enum wasm_importtype_t {}
pub(crate) enum wasm_exporttype_t {}
pub(crate) enum wasm_externtype_t {}

#[repr(C)]
pub(crate) struct wasm_byte_vec_t {
    pub size: usize,
    pub data: *mut u8,
}

#[repr(C)]
pub(crate) struct wasm_extern_vec_t {
    pub size: usize,
    pub data: *mut *mut wasm_extern_t,
}

#[repr(C)]
pub(crate) struct wasm_valtype_vec_t {
    pub size: usize,
    pub data: *mut *mut wasm_valtype_t,
}

#[repr(C)]
pub(crate) struct wasm_importtype_vec_t {
    pub size: usize,
    pub data: *mut *mut wasm_importtype_t,
}

#[repr(C)]
pub(crate) struct wasm_exporttype_vec_t {
    pub size: usize,
    pub data: *mut *mut wasm_exporttype_t,
}

#[repr(C)]
pub(crate) struct wasm_val_vec_t {
    pub size: usize,
    pub data: *mut wasm_val_t,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) union wasm_val_union {
    pub i32_: i32,
    pub i64_: i64,
    pub f32_: f32,
    pub f64_: f64,
    pub ref_: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct wasm_val_t {
    pub kind: u8,
    pub of: wasm_val_union,
}

pub(crate) const WASM_I32: u8 = 0;
pub(crate) const WASM_I64: u8 = 1;
pub(crate) const WASM_F32: u8 = 2;
pub(crate) const WASM_F64: u8 = 3;
pub(crate) const WASM_EXTERN_FUNC: u8 = 0;
pub(crate) const WASM_EXTERN_MEMORY: u8 = 3;
pub(crate) const PAGE: usize = 65536;

pub(crate) type HostCallback =
    unsafe extern "C" fn(*const wasm_val_vec_t, *mut wasm_val_vec_t) -> *mut wasm_trap_t;

unsafe extern "C" {
    pub fn wasm_engine_new() -> *mut wasm_engine_t;
    pub fn wasm_store_new(engine: *mut wasm_engine_t) -> *mut wasm_store_t;
    pub fn wasm_module_new(
        store: *mut wasm_store_t,
        binary: *const wasm_byte_vec_t,
    ) -> *mut wasm_module_t;
    pub fn wasm_instance_new(
        store: *mut wasm_store_t,
        module: *const wasm_module_t,
        imports: *const wasm_extern_vec_t,
        trap_out: *mut *mut wasm_trap_t,
    ) -> *mut wasm_instance_t;
    pub fn wasm_instance_exports(instance: *mut wasm_instance_t, out: *mut wasm_extern_vec_t);
    pub fn wasm_extern_kind(ext: *const wasm_extern_t) -> u8;
    pub fn wasm_extern_as_func(ext: *mut wasm_extern_t) -> *mut wasm_func_t;
    pub fn wasm_extern_as_memory(ext: *mut wasm_extern_t) -> *mut wasm_memory_t;
    pub fn wasm_func_call(
        func: *mut wasm_func_t,
        args: *const wasm_val_vec_t,
        results: *mut wasm_val_vec_t,
    ) -> *mut wasm_trap_t;
    pub fn wasm_func_copy(func: *const wasm_func_t) -> *mut wasm_func_t;
    pub fn wasm_func_type(func: *const wasm_func_t) -> *mut wasm_functype_t;
    pub fn wasm_memory_copy(memory: *const wasm_memory_t) -> *mut wasm_memory_t;
    pub fn wasm_memory_data(memory: *mut wasm_memory_t) -> *mut u8;
    pub fn wasm_memory_data_size(memory: *const wasm_memory_t) -> usize;
    pub fn wasm_trap_delete(trap: *mut wasm_trap_t);
    pub fn wasm_trap_message(trap: *const wasm_trap_t, out: *mut wasm_byte_vec_t);
    pub fn wasm_trap_new(
        store: *mut wasm_store_t,
        message: *const wasm_byte_vec_t,
    ) -> *mut wasm_trap_t;
    pub fn wasm_byte_vec_delete(vec: *mut wasm_byte_vec_t);
    pub fn wasm_extern_vec_delete(vec: *mut wasm_extern_vec_t);
    pub fn wasm_module_imports(module: *const wasm_module_t, out: *mut wasm_importtype_vec_t);
    pub fn wasm_module_exports(module: *const wasm_module_t, out: *mut wasm_exporttype_vec_t);
    pub fn wasm_importtype_module(ty: *const wasm_importtype_t) -> *const wasm_byte_vec_t;
    pub fn wasm_importtype_name(ty: *const wasm_importtype_t) -> *const wasm_byte_vec_t;
    pub fn wasm_importtype_type(ty: *const wasm_importtype_t) -> *const wasm_externtype_t;
    pub fn wasm_exporttype_name(ty: *const wasm_exporttype_t) -> *const wasm_byte_vec_t;
    pub fn wasm_exporttype_type(ty: *const wasm_exporttype_t) -> *const wasm_externtype_t;
    pub fn wasm_externtype_kind(ty: *const wasm_externtype_t) -> u8;
    pub fn wasm_externtype_as_functype(ty: *mut wasm_externtype_t) -> *mut wasm_functype_t;
    pub fn wasm_functype_params(ft: *const wasm_functype_t) -> *const wasm_valtype_vec_t;
    pub fn wasm_functype_results(ft: *const wasm_functype_t) -> *const wasm_valtype_vec_t;
    pub fn wasm_valtype_kind(vt: *const wasm_valtype_t) -> u8;
    pub fn wasm_valtype_new(kind: u8) -> *mut wasm_valtype_t;
    pub fn wasm_valtype_vec_new(
        out: *mut wasm_valtype_vec_t,
        size: usize,
        src: *const *mut wasm_valtype_t,
    );
    pub fn wasm_functype_new(
        params: *mut wasm_valtype_vec_t,
        results: *mut wasm_valtype_vec_t,
    ) -> *mut wasm_functype_t;
    pub fn wasm_functype_delete(ft: *mut wasm_functype_t);
    pub fn wasm_func_new(
        store: *mut wasm_store_t,
        ft: *const wasm_functype_t,
        callback: HostCallback,
    ) -> *mut wasm_func_t;
    pub fn wasm_func_as_extern(func: *mut wasm_func_t) -> *mut wasm_extern_t;
    pub fn wasm_importtype_vec_delete(vec: *mut wasm_importtype_vec_t);
    pub fn wasm_exporttype_vec_delete(vec: *mut wasm_exporttype_vec_t);
    pub fn zwasm_engine_set_memory_growth_hook(
        engine: *mut wasm_engine_t,
        hook: Option<unsafe extern "C" fn(*mut c_void, u64, u32, u64, u64)>,
        user_data: *mut c_void,
    );
    pub fn zwasm_engine_set_instantiate_hook(
        engine: *mut wasm_engine_t,
        hook: Option<unsafe extern "C" fn(*mut c_void, u64)>,
        user_data: *mut c_void,
    );
}

/// One host function kept until the store is dropped. This runtime never drops
/// the store: it lives with the component.
pub(crate) struct HostFunc {
    pub js: Persistent<Function<'static>>,
    pub params: Vec<u8>,
    pub results: Vec<u8>,
}

pub(crate) struct GuestFunc {
    pub func: *mut wasm_func_t,
    pub params: Vec<u8>,
    pub results: Vec<u8>,
}

pub(crate) struct MemState {
    pub instance_id: u64,
    pub memory_index: u32,
    pub memory: *mut wasm_memory_t,
    pub handed: Vec<Persistent<Value<'static>>>,
}

struct Growth {
    instance_id: u64,
    memory_index: u32,
}

struct EngineHost {
    store: *mut wasm_store_t,
    host_funcs: Vec<HostFunc>,
    guest_funcs: Vec<GuestFunc>,
    memories: Vec<MemState>,
    /// Modules, instances, and copied handles kept for the store lifetime.
    retained: Vec<*mut c_void>,
    growths: Vec<Growth>,
}

struct HostCell(RefCell<EngineHost>);

// SAFETY: the component is single-threaded, matching the rest of this runtime.
unsafe impl Sync for HostCell {}

static HOST: HostCell = HostCell(RefCell::new(EngineHost {
    store: ptr::null_mut(),
    host_funcs: Vec::new(),
    guest_funcs: Vec::new(),
    memories: Vec::new(),
    retained: Vec::new(),
    growths: Vec::new(),
}));

static mut LAST_INSTANCE_ID: u64 = 0;

unsafe extern "C" fn on_instantiate(_user: *mut c_void, instance_id: u64) {
    unsafe {
        LAST_INSTANCE_ID = instance_id;
    }
}

/// Records growth. Must not call back into the engine, the store, or the instance.
unsafe extern "C" fn on_memory_growth(
    _user: *mut c_void,
    instance_id: u64,
    memory_index: u32,
    _old_pages: u64,
    _new_pages: u64,
) {
    if let Ok(mut host) = HOST.0.try_borrow_mut() {
        host.growths.push(Growth {
            instance_id,
            memory_index,
        });
    }
}

#[allow(private_bounds)]
pub(crate) fn with_host<R>(f: impl FnOnce(&mut EngineHost) -> R) -> R {
    f(&mut HOST.0.borrow_mut())
}

pub(crate) fn ensure_engine() {
    let mut host = HOST.0.borrow_mut();
    if !host.store.is_null() {
        return;
    }
    unsafe {
        let engine = wasm_engine_new();
        assert!(!engine.is_null(), "wasm_engine_new failed");
        zwasm_engine_set_memory_growth_hook(engine, Some(on_memory_growth), ptr::null_mut());
        zwasm_engine_set_instantiate_hook(engine, Some(on_instantiate), ptr::null_mut());
        let store = wasm_store_new(engine);
        assert!(!store.is_null(), "wasm_store_new failed");
        host.store = store;
        host.retained.push(engine.cast());
    }
}

pub(crate) fn store() -> *mut wasm_store_t {
    ensure_engine();
    HOST.0.borrow().store
}

pub(crate) fn last_instance_id() -> u64 {
    unsafe { LAST_INSTANCE_ID }
}

pub(crate) fn retain(ptr: *mut c_void) {
    if !ptr.is_null() {
        HOST.0.borrow_mut().retained.push(ptr);
    }
}

pub(crate) fn push_host_func(func: HostFunc) -> usize {
    let mut host = HOST.0.borrow_mut();
    let id = host.host_funcs.len();
    host.host_funcs.push(func);
    id
}

pub(crate) fn push_guest_func(func: GuestFunc) -> usize {
    let mut host = HOST.0.borrow_mut();
    let id = host.guest_funcs.len();
    host.guest_funcs.push(func);
    id
}

pub(crate) fn push_memory(mem: MemState) -> usize {
    let mut host = HOST.0.borrow_mut();
    let id = host.memories.len();
    host.memories.push(mem);
    id
}

pub(crate) fn guest_func(id: usize) -> GuestFuncRef {
    let host = HOST.0.borrow();
    let func = &host.guest_funcs[id];
    GuestFuncRef {
        func: func.func,
        params: func.params.clone(),
        results: func.results.clone(),
    }
}

pub(crate) struct GuestFuncRef {
    pub func: *mut wasm_func_t,
    pub params: Vec<u8>,
    pub results: Vec<u8>,
}

pub(crate) fn name_bytes(name: *const wasm_byte_vec_t) -> String {
    unsafe {
        if name.is_null() {
            return String::new();
        }
        let name = &*name;
        if name.data.is_null() || name.size == 0 {
            return String::new();
        }
        let mut bytes = std::slice::from_raw_parts(name.data, name.size);
        if bytes.last() == Some(&0) {
            bytes = &bytes[..bytes.len() - 1];
        }
        String::from_utf8_lossy(bytes).into_owned()
    }
}

pub(crate) fn valtype_kinds(vec: *const wasm_valtype_vec_t) -> Vec<u8> {
    unsafe {
        if vec.is_null() {
            return Vec::new();
        }
        let vec = &*vec;
        let mut out = Vec::with_capacity(vec.size);
        for i in 0..vec.size {
            let vt = *vec.data.add(i);
            out.push(wasm_valtype_kind(vt));
        }
        out
    }
}

pub(crate) fn trap_text(trap: *mut wasm_trap_t) -> String {
    unsafe {
        if trap.is_null() {
            return "wasm trap".to_string();
        }
        let mut message = wasm_byte_vec_t {
            size: 0,
            data: ptr::null_mut(),
        };
        wasm_trap_message(trap, &mut message);
        let text = name_bytes(&message);
        if !message.data.is_null() {
            wasm_byte_vec_delete(&mut message);
        }
        wasm_trap_delete(trap);
        if text.is_empty() {
            "wasm trap".to_string()
        } else {
            text
        }
    }
}

pub(crate) fn make_functype(params: &[u8], results: &[u8]) -> Result<*mut wasm_functype_t, String> {
    unsafe {
        let mut param_tys = Vec::with_capacity(params.len());
        for &kind in params {
            let ty = wasm_valtype_new(kind);
            if ty.is_null() {
                return Err("wasm_valtype_new failed".into());
            }
            param_tys.push(ty);
        }
        let mut result_tys = Vec::with_capacity(results.len());
        for &kind in results {
            let ty = wasm_valtype_new(kind);
            if ty.is_null() {
                return Err("wasm_valtype_new failed".into());
            }
            result_tys.push(ty);
        }
        let mut param_vec = wasm_valtype_vec_t {
            size: 0,
            data: ptr::null_mut(),
        };
        let mut result_vec = wasm_valtype_vec_t {
            size: 0,
            data: ptr::null_mut(),
        };
        wasm_valtype_vec_new(&mut param_vec, param_tys.len(), param_tys.as_ptr());
        wasm_valtype_vec_new(&mut result_vec, result_tys.len(), result_tys.as_ptr());
        let ft = wasm_functype_new(&mut param_vec, &mut result_vec);
        if ft.is_null() {
            return Err("wasm_functype_new failed".into());
        }
        Ok(ft)
    }
}

fn js_to_val(value: &Value<'_>, kind: u8) -> Result<wasm_val_t, ()> {
    let number = value
        .as_float()
        .or_else(|| value.as_int().map(|n| n as f64));
    let Some(number) = number else {
        return Err(());
    };
    let mut slot = wasm_val_t {
        kind,
        of: wasm_val_union { i64_: 0 },
    };
    match kind {
        WASM_I32 => slot.of.i32_ = number as i32,
        WASM_I64 => slot.of.i64_ = number as i64,
        WASM_F32 => slot.of.f32_ = number as f32,
        WASM_F64 => slot.of.f64_ = number,
        _ => return Err(()),
    }
    Ok(slot)
}

fn val_to_js<'js>(ctx: &Ctx<'js>, val: &wasm_val_t) -> rquickjs::Result<Value<'js>> {
    let number = unsafe {
        match val.kind {
            WASM_I32 => val.of.i32_ as f64,
            WASM_I64 => val.of.i64_ as f64,
            WASM_F32 => val.of.f32_ as f64,
            WASM_F64 => val.of.f64_,
            _ => {
                return Err(rquickjs::Exception::throw_type(
                    ctx,
                    "unsupported wasm value type",
                ));
            }
        }
    };
    number.into_js(ctx)
}

/// Trampolines. `wasm_func_new` has no environment pointer, so each import
/// gets its own callback that closes over a slot kept for the store lifetime.
macro_rules! host_slots {
    ($($name:ident = $index:expr),* $(,)?) => {
        $(
            unsafe extern "C" fn $name(
                args: *const wasm_val_vec_t,
                results: *mut wasm_val_vec_t,
            ) -> *mut wasm_trap_t {
                dispatch_host($index, args, results)
            }
        )*
        const HOST_TRAMPOLINES: &[HostCallback] = &[$($name),*];
    };
}

host_slots! {
    host_slot_0 = 0, host_slot_1 = 1, host_slot_2 = 2, host_slot_3 = 3,
    host_slot_4 = 4, host_slot_5 = 5, host_slot_6 = 6, host_slot_7 = 7,
    host_slot_8 = 8, host_slot_9 = 9, host_slot_10 = 10, host_slot_11 = 11,
    host_slot_12 = 12, host_slot_13 = 13, host_slot_14 = 14, host_slot_15 = 15,
    host_slot_16 = 16, host_slot_17 = 17, host_slot_18 = 18, host_slot_19 = 19,
    host_slot_20 = 20, host_slot_21 = 21, host_slot_22 = 22, host_slot_23 = 23,
    host_slot_24 = 24, host_slot_25 = 25, host_slot_26 = 26, host_slot_27 = 27,
    host_slot_28 = 28, host_slot_29 = 29, host_slot_30 = 30, host_slot_31 = 31,
}

pub(crate) fn host_trampoline(index: usize) -> Result<HostCallback, String> {
    HOST_TRAMPOLINES
        .get(index)
        .copied()
        .ok_or_else(|| "too many function imports for one store".into())
}

fn dispatch_host(
    index: usize,
    args: *const wasm_val_vec_t,
    results: *mut wasm_val_vec_t,
) -> *mut wasm_trap_t {
    let (js, params, result_kinds) = {
        let host = HOST.0.borrow();
        let func = &host.host_funcs[index];
        (func.js.clone(), func.params.clone(), func.results.clone())
    };
    crate::with_ctx(|ctx| call_host_js(ctx, js, &params, &result_kinds, args, results))
}

fn call_host_js(
    ctx: &Ctx<'_>,
    js: Persistent<Function<'static>>,
    params: &[u8],
    result_kinds: &[u8],
    args: *const wasm_val_vec_t,
    results: *mut wasm_val_vec_t,
) -> *mut wasm_trap_t {
    let call = (|| {
        let func = js.restore(ctx).map_err(|_| ())?;
        let mut js_args: Vec<Value> = Vec::with_capacity(params.len());
        unsafe {
            if args.is_null()
                || (*args).size < params.len()
                || (params.len() > 0 && (*args).data.is_null())
            {
                return Err(());
            }
            for (i, &kind) in params.iter().enumerate() {
                let slot = &*(*args).data.add(i);
                js_args.push(val_to_js(ctx, &wasm_val_t { kind, of: slot.of }).map_err(|_| ())?);
            }
        }
        let mut call_args = Args::new(ctx.clone(), js_args.len());
        for arg in js_args {
            call_args.push_arg(arg).map_err(|_| ())?;
        }
        let returned = match CaughtError::catch(ctx, func.call_arg::<Value>(call_args)) {
            Ok(value) => value,
            Err(_) => return Err(()),
        };
        unsafe {
            if results.is_null() || (*results).size < result_kinds.len() {
                return Err(());
            }
            if result_kinds.is_empty() {
                return Ok(());
            }
            let values: Vec<Value> = if result_kinds.len() == 1 {
                vec![returned]
            } else {
                let array = rquickjs::Array::from_value(returned).map_err(|_| ())?;
                let mut items = Vec::with_capacity(result_kinds.len());
                for i in 0..result_kinds.len() {
                    items.push(array.get(i).map_err(|_| ())?);
                }
                items
            };
            for (i, &kind) in result_kinds.iter().enumerate() {
                let slot = js_to_val(&values[i], kind).map_err(|_| ())?;
                let dst = &mut *(*results).data.add(i);
                dst.kind = slot.kind;
                dst.of = slot.of;
            }
        }
        Ok(())
    })();
    match call {
        Ok(()) => ptr::null_mut(),
        Err(()) => new_trap("host function failed"),
    }
}

pub(crate) fn new_trap(message: &str) -> *mut wasm_trap_t {
    let store = store();
    let mut bytes = message.as_bytes().to_vec();
    bytes.push(0);
    let vec = wasm_byte_vec_t {
        size: bytes.len(),
        data: bytes.as_mut_ptr(),
    };
    unsafe { wasm_trap_new(store, &vec) }
}

/// Publish fresh buffers for memories that grew. Runs after the hook has
/// returned, so reading `wasm_memory_data` is outside the hook.
pub(crate) fn take_growths() -> Vec<(u64, u32)> {
    HOST.0
        .borrow_mut()
        .growths
        .drain(..)
        .map(|g| (g.instance_id, g.memory_index))
        .collect()
}

pub(crate) fn memory_ptr(id: usize) -> *mut wasm_memory_t {
    HOST.0.borrow().memories[id].memory
}

pub(crate) fn last_buffer(id: usize) -> Option<Persistent<Value<'static>>> {
    HOST.0.borrow().memories[id].handed.last().cloned()
}

pub(crate) fn take_handed(id: usize) -> Vec<Persistent<Value<'static>>> {
    std::mem::take(&mut HOST.0.borrow_mut().memories[id].handed)
}

pub(crate) fn remember_buffer(id: usize, buffer: Persistent<Value<'static>>) {
    HOST.0.borrow_mut().memories[id].handed.push(buffer);
}

pub(crate) fn memories_matching(instance_id: u64, memory_index: u32) -> Vec<usize> {
    HOST.0
        .borrow()
        .memories
        .iter()
        .enumerate()
        .filter(|(_, mem)| mem.instance_id == instance_id && mem.memory_index == memory_index)
        .map(|(i, _)| i)
        .collect()
}

pub(crate) fn call_guest(
    func: *mut wasm_func_t,
    params: &[u8],
    results: &[u8],
    args: &[Value<'_>],
) -> Result<Vec<wasm_val_t>, String> {
    if args.len() != params.len() {
        return Err("wasm function argument count mismatch".into());
    }
    let mut arg_slots = Vec::with_capacity(params.len());
    for (value, &kind) in args.iter().zip(params) {
        arg_slots
            .push(js_to_val(value, kind).map_err(|_| "wasm argument is not a number".to_string())?);
    }
    let mut result_slots = vec![
        wasm_val_t {
            kind: 0,
            of: wasm_val_union { i64_: 0 },
        };
        results.len()
    ];
    let arg_vec = wasm_val_vec_t {
        size: arg_slots.len(),
        data: if arg_slots.is_empty() {
            ptr::null_mut()
        } else {
            arg_slots.as_mut_ptr()
        },
    };
    let mut result_vec = wasm_val_vec_t {
        size: result_slots.len(),
        data: if result_slots.is_empty() {
            ptr::null_mut()
        } else {
            result_slots.as_mut_ptr()
        },
    };
    unsafe {
        let trap = wasm_func_call(func, &arg_vec, &mut result_vec);
        if !trap.is_null() {
            return Err(trap_text(trap));
        }
    }
    for (slot, &kind) in result_slots.iter_mut().zip(results) {
        slot.kind = kind;
    }
    Ok(result_slots)
}

pub(crate) fn result_to_js<'js>(
    ctx: &Ctx<'js>,
    values: &[wasm_val_t],
) -> rquickjs::Result<Value<'js>> {
    if values.is_empty() {
        return Ok(Value::new_undefined(ctx.clone()));
    }
    if values.len() == 1 {
        return val_to_js(ctx, &values[0]);
    }
    let array = rquickjs::Array::new(ctx.clone())?;
    for (i, value) in values.iter().enumerate() {
        array.set(i, val_to_js(ctx, value)?)?;
    }
    Ok(array.into_value())
}
