//! `WebAssembly.instantiate` on the QuickJS global, backed by zwasm.

use std::ptr;

use rquickjs::function::Rest;
use rquickjs::{ArrayBuffer, Ctx, Exception, Function, Object, Value};

use crate::zwasm::{
    self, GuestFunc, HostFunc, MemState, WASM_EXTERN_FUNC, WASM_EXTERN_MEMORY, wasm_byte_vec_t,
    wasm_exporttype_vec_t, wasm_extern_vec_t, wasm_importtype_vec_t,
};

pub(crate) fn register<'js>(ctx: &Ctx<'js>) -> rquickjs::Result<()> {
    zwasm::ensure_engine();

    let memory_proto = Object::new(ctx.clone())?;
    let memory_ctor = Function::new(ctx.clone(), |ctx: Ctx<'_>, _args: Rest<Value<'_>>| {
        Err::<Value<'_>, _>(Exception::throw_type(
            &ctx,
            "WebAssembly.Memory constructor is not supported",
        ))
    })?;
    memory_ctor.set("prototype", memory_proto.clone())?;
    memory_proto.set("constructor", memory_ctor.clone())?;

    let wasm = Object::new(ctx.clone())?;
    wasm.set("Memory", memory_ctor)?;
    wasm.set(
        "instantiate",
        Function::new(
            ctx.clone(),
            |ctx: Ctx<'js>, args: Rest<Value<'js>>| instantiate(&ctx, args),
        )?,
    )?;
    ctx.globals().set("WebAssembly", wasm)?;
    Ok(())
}

fn instantiate<'js>(ctx: &Ctx<'js>, args: Rest<Value<'js>>) -> rquickjs::Result<Object<'js>> {
    let bytes = args
        .0
        .first()
        .ok_or_else(|| Exception::throw_type(ctx, "WebAssembly.instantiate requires bytes"))?;
    let bytes = buffer_source(ctx, bytes.clone())?;
    let import_object = args.0.get(1).cloned();

    let module = unsafe {
        let binary = wasm_byte_vec_t {
            size: bytes.len(),
            data: bytes.as_ptr() as *mut u8,
        };
        zwasm::wasm_module_new(zwasm::store(), &binary)
    };
    if module.is_null() {
        return Err(Exception::throw_message(
            ctx,
            "WebAssembly.instantiate: module rejected",
        ));
    }
    zwasm::retain(module.cast());

    let imports = link_imports(ctx, module, import_object)?;
    let mut trap = ptr::null_mut();
    let instance = unsafe {
        zwasm::wasm_instance_new(zwasm::store(), module, &imports.vec, &mut trap)
    };
    if instance.is_null() {
        let message = if trap.is_null() {
            "WebAssembly.instantiate failed".to_string()
        } else {
            format!(
                "WebAssembly.instantiate failed: {}",
                zwasm::trap_text(trap)
            )
        };
        return Err(Exception::throw_message(ctx, &message));
    }
    zwasm::retain(instance.cast());
    let instance_id = zwasm::last_instance_id();

    let wasm_ns: Object = ctx.globals().get("WebAssembly")?;
    let memory_ctor: Object = wasm_ns.get("Memory")?;
    let memory_proto: Object = memory_ctor.get("prototype")?;
    let exports = export_object(ctx, &memory_proto, module, instance, instance_id)?;
    apply_growths(ctx)?;

    let instance_obj = Object::new(ctx.clone())?;
    instance_obj.set("exports", exports)?;
    let module_obj = Object::new(ctx.clone())?;
    let out = Object::new(ctx.clone())?;
    out.set("instance", instance_obj)?;
    out.set("module", module_obj)?;
    Ok(out)
}

struct ImportLink {
    vec: wasm_extern_vec_t,
    // Owns the pointer buffer `vec.data` references for the duration of instantiate.
    #[allow(dead_code)]
    slots: Vec<*mut zwasm::wasm_extern_t>,
}

fn link_imports<'js>(
    ctx: &Ctx<'js>,
    module: *mut zwasm::wasm_module_t,
    import_object: Option<Value<'js>>,
) -> rquickjs::Result<ImportLink> {
    let mut import_types = wasm_importtype_vec_t {
        size: 0,
        data: ptr::null_mut(),
    };
    unsafe { zwasm::wasm_module_imports(module, &mut import_types) };
    let result = (|| {
        if import_types.size == 0 {
            return Ok(ImportLink {
                vec: wasm_extern_vec_t {
                    size: 0,
                    data: ptr::null_mut(),
                },
                slots: Vec::new(),
            });
        }
        let import_object = import_object.ok_or_else(|| {
            Exception::throw_type(ctx, "WebAssembly.instantiate: missing import object")
        })?;
        let import_object = import_object.into_object().ok_or_else(|| {
            Exception::throw_type(ctx, "WebAssembly.instantiate: import object must be an object")
        })?;
        let mut slots = Vec::with_capacity(import_types.size);
        for i in 0..import_types.size {
            let ty = unsafe { *import_types.data.add(i) };
            let module_name = zwasm::name_bytes(unsafe { zwasm::wasm_importtype_module(ty) });
            let import_name = zwasm::name_bytes(unsafe { zwasm::wasm_importtype_name(ty) });
            let extern_ty = unsafe { zwasm::wasm_importtype_type(ty) };
            let kind = unsafe { zwasm::wasm_externtype_kind(extern_ty) };
            if kind != WASM_EXTERN_FUNC {
                return Err(Exception::throw_type(
                    ctx,
                    &format!("unsupported import {module_name}.{import_name}"),
                ));
            }
            let module_obj: Object = import_object.get(&module_name).map_err(|_| {
                Exception::throw_type(
                    ctx,
                    &format!("missing import module {module_name}"),
                )
            })?;
            let value: Value = module_obj.get(&import_name).map_err(|_| {
                Exception::throw_type(
                    ctx,
                    &format!("missing import {module_name}.{import_name}"),
                )
            })?;
            let function = value.into_function().ok_or_else(|| {
                Exception::throw_type(
                    ctx,
                    &format!("import {module_name}.{import_name} is not a function"),
                )
            })?;
            let functype = unsafe { zwasm::wasm_externtype_as_functype(extern_ty as *mut _) };
            let params = zwasm::valtype_kinds(unsafe { zwasm::wasm_functype_params(functype) });
            let results = zwasm::valtype_kinds(unsafe { zwasm::wasm_functype_results(functype) });
            let slot = zwasm::push_host_func(HostFunc {
                js: rquickjs::Persistent::save(ctx, function),
                params: params.clone(),
                results: results.clone(),
            });
            let callback = zwasm::host_trampoline(slot).map_err(|err| Exception::throw_message(ctx, &err))?;
            let owned = zwasm::make_functype(&params, &results)
                .map_err(|err| Exception::throw_message(ctx, &err))?;
            let func = unsafe { zwasm::wasm_func_new(zwasm::store(), owned, callback) };
            unsafe { zwasm::wasm_functype_delete(owned) };
            if func.is_null() {
                return Err(Exception::throw_message(ctx, "wasm_func_new failed"));
            }
            zwasm::retain(func.cast());
            let ext = unsafe { zwasm::wasm_func_as_extern(func) };
            if ext.is_null() {
                return Err(Exception::throw_message(ctx, "wasm_func_as_extern failed"));
            }
            slots.push(ext);
        }
        let vec = wasm_extern_vec_t {
            size: slots.len(),
            data: slots.as_mut_ptr(),
        };
        Ok(ImportLink { vec, slots })
    })();
    unsafe {
        if !import_types.data.is_null() {
            zwasm::wasm_importtype_vec_delete(&mut import_types);
        }
    }
    result
}

fn export_object<'js>(
    ctx: &Ctx<'js>,
    memory_proto: &Object<'js>,
    module: *mut zwasm::wasm_module_t,
    instance: *mut zwasm::wasm_instance_t,
    instance_id: u64,
) -> rquickjs::Result<Object<'js>> {
    let mut export_types = wasm_exporttype_vec_t {
        size: 0,
        data: ptr::null_mut(),
    };
    unsafe { zwasm::wasm_module_exports(module, &mut export_types) };
    let mut exports = wasm_extern_vec_t {
        size: 0,
        data: ptr::null_mut(),
    };
    unsafe { zwasm::wasm_instance_exports(instance, &mut exports) };
    let result = (|| {
        let obj = Object::new(ctx.clone())?;
        let mut memory_index = 0u32;
        let count = export_types.size.min(exports.size);
        for i in 0..count {
            let ty = unsafe { *export_types.data.add(i) };
            let name = zwasm::name_bytes(unsafe { zwasm::wasm_exporttype_name(ty) });
            let extern_ty = unsafe { zwasm::wasm_exporttype_type(ty) };
            let kind = unsafe { zwasm::wasm_externtype_kind(extern_ty) };
            let ext = unsafe { *exports.data.add(i) };
            if kind == WASM_EXTERN_FUNC {
                let func = unsafe { zwasm::wasm_extern_as_func(ext) };
                let copied = unsafe { zwasm::wasm_func_copy(func) };
                if copied.is_null() {
                    return Err(Exception::throw_message(ctx, "wasm_func_copy failed"));
                }
                zwasm::retain(copied.cast());
                let ft = unsafe { zwasm::wasm_func_type(copied) };
                let params = zwasm::valtype_kinds(unsafe { zwasm::wasm_functype_params(ft) });
                let results = zwasm::valtype_kinds(unsafe { zwasm::wasm_functype_results(ft) });
                unsafe { zwasm::wasm_functype_delete(ft) };
                let id = zwasm::push_guest_func(GuestFunc {
                    func: copied,
                    params,
                    results,
                });
                let js = Function::new(ctx.clone(), move |ctx: Ctx<'js>, args: Rest<Value<'js>>| {
                    call_export(&ctx, id, &args.0)
                })?;
                obj.set(name, js)?;
            } else if kind == WASM_EXTERN_MEMORY {
                let memory = unsafe { zwasm::wasm_extern_as_memory(ext) };
                let copied = unsafe { zwasm::wasm_memory_copy(memory) };
                if copied.is_null() {
                    return Err(Exception::throw_message(ctx, "wasm_memory_copy failed"));
                }
                zwasm::retain(copied.cast());
                let id = zwasm::push_memory(MemState {
                    instance_id,
                    memory_index,
                    memory: copied,
                    handed: Vec::new(),
                });
                memory_index += 1;
                let mem_obj = Object::new(ctx.clone())?;
                unsafe {
                    rquickjs::qjs::JS_SetPrototype(
                        ctx.as_raw().as_ptr(),
                        mem_obj.as_value().as_raw(),
                        memory_proto.as_value().as_raw(),
                    );
                }
                mem_obj.prop(
                    "buffer",
                    rquickjs::object::Accessor::from(move |ctx: Ctx<'js>| -> rquickjs::Result<Value<'js>> {
                        current_buffer(&ctx, id)
                    })
                    .enumerable(),
                )?;
                obj.set(name, mem_obj)?;
            }
        }
        Ok(obj)
    })();
    unsafe {
        if !export_types.data.is_null() {
            zwasm::wasm_exporttype_vec_delete(&mut export_types);
        }
        if !exports.data.is_null() {
            zwasm::wasm_extern_vec_delete(&mut exports);
        }
    }
    result
}

fn call_export<'js>(ctx: &Ctx<'js>, id: usize, args: &[Value<'js>]) -> rquickjs::Result<Value<'js>> {
    let guest = zwasm::guest_func(id);
    let called = zwasm::call_guest(guest.func, &guest.params, &guest.results, args);
    apply_growths(ctx)?;
    match called {
        Ok(values) => zwasm::result_to_js(ctx, &values),
        Err(message) => Err(Exception::throw_message(ctx, &message)),
    }
}

fn current_buffer<'js>(ctx: &Ctx<'js>, id: usize) -> rquickjs::Result<Value<'js>> {
    apply_growths(ctx)?;
    if let Some(existing) = zwasm::last_buffer(id) {
        return existing.restore(ctx);
    }
    publish_buffer(ctx, id)
}

fn publish_buffer<'js>(ctx: &Ctx<'js>, id: usize) -> rquickjs::Result<Value<'js>> {
    let memory = zwasm::memory_ptr(id);
    let (ptr, len) = unsafe {
        (
            zwasm::wasm_memory_data(memory),
            zwasm::wasm_memory_data_size(memory),
        )
    };
    let raw = unsafe {
        rquickjs::qjs::JS_NewArrayBuffer(
            ctx.as_raw().as_ptr(),
            ptr,
            len as _,
            Some(noop_free),
            ptr::null_mut(),
            false,
        )
    };
    if raw >> 32 == rquickjs::qjs::JS_TAG_EXCEPTION as u64 {
        return Err(Exception::throw_message(ctx, "failed to publish wasm memory buffer"));
    }
    let value = unsafe { Value::from_raw(ctx.clone(), raw) };
    zwasm::remember_buffer(id, rquickjs::Persistent::save(ctx, value.clone()));
    Ok(value)
}

unsafe extern "C" fn noop_free(
    _rt: *mut rquickjs::qjs::JSRuntime,
    _opaque: *mut std::ffi::c_void,
    _ptr: *mut std::ffi::c_void,
) {
}

fn apply_growths(ctx: &Ctx<'_>) -> rquickjs::Result<()> {
    let events = zwasm::take_growths();
    for (instance_id, memory_index) in events {
        for id in zwasm::memories_matching(instance_id, memory_index) {
            for buffer in zwasm::take_handed(id) {
                let value = buffer.restore(ctx)?;
                unsafe {
                    rquickjs::qjs::JS_DetachArrayBuffer(ctx.as_raw().as_ptr(), value.as_raw());
                }
            }
            let _ = publish_buffer(ctx, id)?;
        }
    }
    Ok(())
}

fn buffer_source(ctx: &Ctx<'_>, value: Value<'_>) -> rquickjs::Result<Vec<u8>> {
    if let Some(buffer) = value
        .as_object()
        .cloned()
        .and_then(ArrayBuffer::from_object)
    {
        let raw = buffer.as_raw().ok_or_else(|| {
            Exception::throw_type(ctx, "WebAssembly.instantiate: ArrayBuffer is detached")
        })?;
        return Ok(unsafe { std::slice::from_raw_parts(raw.ptr.as_ptr(), raw.len).to_vec() });
    }
    let object = value.into_object().ok_or_else(|| {
        Exception::throw_type(ctx, "WebAssembly.instantiate: bytes must be a BufferSource")
    })?;
    if object.get::<_, Value>("buffer").is_ok() && object.get::<_, Value>("byteLength").is_ok() {
        let buffer: ArrayBuffer = object.get("buffer").map_err(|_| {
            Exception::throw_type(ctx, "WebAssembly.instantiate: bytes must be a BufferSource")
        })?;
        let offset: usize = object.get("byteOffset").unwrap_or(0);
        let length: usize = object.get("byteLength").map_err(|_| {
            Exception::throw_type(ctx, "WebAssembly.instantiate: bytes must be a BufferSource")
        })?;
        let raw = buffer.as_raw().ok_or_else(|| {
            Exception::throw_type(ctx, "WebAssembly.instantiate: ArrayBuffer is detached")
        })?;
        let end = offset.saturating_add(length);
        if end > raw.len {
            return Err(Exception::throw_type(
                ctx,
                "WebAssembly.instantiate: view exceeds its buffer",
            ));
        }
        return Ok(unsafe { std::slice::from_raw_parts(raw.ptr.as_ptr().add(offset), length).to_vec() });
    }
    Err(Exception::throw_type(
        ctx,
        "WebAssembly.instantiate: bytes must be a BufferSource",
    ))
}
