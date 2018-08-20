
use jemalloc_ctl;

// This is the interface to the JVM that we'll
// call the majority of our methods on.
use crate::jni::JNIEnv;
// These objects are what you should use as arguments to your native function.
// They carry extra lifetime information to prevent them escaping this context
// and getting used after being GC'd.
use crate::jni::objects::{GlobalRef, JClass, JObject, JString};
// This is just a pointer. We'll be returning it from our function.
// We can't return one of the objects with lifetime information because the
// lifetime checker won't let us.
use crate::jni::sys::{jbyteArray, jint, jlong, jstring};

/////////////////////////////////////////////////////////////////////////////
// JVM
/////////////////////////////////////////////////////////////////////////////

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_run_mojo_mem_JEMalloc_epoch(
    _env: JNIEnv,
    _class: JClass, ) {
    jemalloc_ctl::epoch().unwrap();
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_run_mojo_mem_JEMalloc_allocated(
    _env: JNIEnv,
    _class: JClass)
    -> jlong {
    jemalloc_ctl::stats::allocated().unwrap() as jlong
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_run_mojo_mem_JEMalloc_active(
    _env: JNIEnv,
    _class: JClass)
    -> jlong {
    jemalloc_ctl::stats::active().unwrap() as jlong
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_run_mojo_mem_JEMalloc_metadata(
    _env: JNIEnv,
    _class: JClass)
    -> jlong {
    jemalloc_ctl::stats::metadata().unwrap() as jlong
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_run_mojo_mem_JEMalloc_resident(
    _env: JNIEnv,
    _class: JClass)
    -> jlong {
    jemalloc_ctl::stats::resident().unwrap() as jlong
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_run_mojo_mem_JEMalloc_mapped(
    _env: JNIEnv,
    _class: JClass)
    -> jlong {
    jemalloc_ctl::stats::mapped().unwrap() as jlong
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_run_mojo_mem_JEMalloc_retained(
    _env: JNIEnv,
    _class: JClass)
    -> jlong {
    jemalloc_ctl::stats::retained().unwrap() as jlong
}


/////////////////////////////////////////////////////////////////////////////
// CoreCLR
/////////////////////////////////////////////////////////////////////////////