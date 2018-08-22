#![allow(dead_code)]
#![allow(non_camel_case_types)]


extern crate actix;
extern crate actix_redis;
extern crate actix_web;
extern crate bytes;
extern crate cesu8;
extern crate combine;
extern crate env_logger;
#[macro_use]
extern crate error_chain;
extern crate futures;
extern crate h2;
extern crate http;
extern crate jemalloc_ctl;
extern crate jemallocator;
extern crate jni_sys;
extern crate libc;
#[macro_use]
extern crate log;
//extern crate openssl;

use actix::dev::{MessageResponse, ResponseChannel};
use actix::prelude::*;
//use actix::system::{SystemRunner};
use actix_redis::RedisSessionBackend;
use actix_web::{App, Body, dev::Handler, error, Error, Error as ActixWebError, Form, HttpRequest, HttpResponse, middleware, Responder, Result, server};
use actix_web::{AsyncResponder, Either};
use actix_web::middleware::session::{self, RequestSession};
use crate::jni::JavaVM;
use crate::jni::JNIEnv;
// These objects are what you should use as arguments to your native function.
// They carry extra lifetime information to prevent them escaping this context
// and getting used after being GC'd.
use crate::jni::objects::{GlobalRef, JClass, JObject, JString, JValue};
use crate::jni::signature::JavaType;
// This is just a pointer. We'll be returning it from our function.
// We can't return one of the objects with lifetime information because the
// lifetime checker won't let us.
use crate::jni::sys::{jbyteArray, jint, jlong, jobject, jstring};
use futures::Future;
use futures::future::result;
use std::mem;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::Duration;

pub mod jni;
pub mod jvm;
pub mod alloc;
pub mod buf;
pub mod listpack;
pub mod actor;
pub mod app;

/////////////////////////////////////////////////////////////////////////////
// Set Global Allocator to JEMalloc
/////////////////////////////////////////////////////////////////////////////

#[global_allocator]
pub static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

pub fn run() {
    println!("Hello, Mojo!");

    // Create actor system.
    let system = actix::System::new("mojo");

    // Create MojoApp
    let mojo = Arc::new(Mutex::new(MojoApp::new()));
    let mojo_clone = mojo.clone();

    let http_server = server::new(move || {
        // Get instance of MojoApp.
        let mut mojo = mojo_clone.lock().unwrap();

        // Init HttpApplication.
        let mut app = App::with_state(AppState {});

        // Build route table.
        app = app.resource("/", |r| {
            r.h(jvm::JavaHttpHandler::new())
        });

        // Create arbiter.
        let mut arbiter = MojoArbiter::new(AppState{});

        mojo.add_arbiter(arbiter.start());

        // Release lock.
        drop(mojo);

        let addr = jvm::Timer { dur: Duration::new(1, 0) }.start();

        // Return app.
        app
    })
        .workers(2)
        .bind("localhost:8000").expect("bind failed")
        .start();

    println!("starting...");

//    let Some(mojo) = mojo.lock().unwrap();

//    let addr = Timer { dur: Duration::new(1, 0) }.start();
//
//    // Send Ping message.
//    // send() message returns Future object, that resolves to message result
//    let ping_future = addr.send(Messages::Ping);
//    let pong_future = addr.send(Messages::Pong);
//
//    // Spawn pong_future onto event loop
//    Arbiter::spawn(
//        pong_future
//            .map(|res| {
//                match res {
//                    Responses::GotPing => println!("Ping received"),
//                    Responses::GotPong => println!("Pong received"),
//                }
//            })
//            .map_err(|e| {
//                println!("Actor is probably died: {}", e);
//            }),
//    );
//
//    // Spawn ping_future onto event loop
//    Arbiter::spawn(
//        ping_future
//            .map(|res| {
//                match res {
//                    Responses::GotPing => println!("Ping received"),
//                    Responses::GotPong => println!("Pong received"),
//                }
//            })
//            .map_err(|e| {
//                println!("Actor is probably died: {}", e);
//            }),
//    );

    system.run();
}

pub enum GCHandleActorMessage {
    Jvm(jvm::GCHandle),
    Net(),
}

pub struct MojoApp {
    loops: Vec<std::sync::Arc<Addr<MojoArbiter>>>,
}

impl MojoApp {
    pub fn new() -> MojoApp {
        MojoApp {
            loops: Vec::with_capacity(128),
        }
    }

    pub fn add_arbiter(&mut self, arbiter: Addr<MojoArbiter>) {
        self.loops.push(std::sync::Arc::new(arbiter));
    }

    pub fn attach_jvm(&mut self, vm: JavaVM) {
        for arbiter in &self.loops {
//            arbiter.do_send(JvmAttach { vm: vm.clone() });
        }
    }
}

pub struct AppState {}

pub struct JVM {
    vm: crate::jni::JavaVM,
    thread: std::thread::Thread,
}

impl Drop for JVM {
    fn drop(&mut self) {
        // Detach from thread.
        if self.thread.id() == std::thread::current().id() {
            self.vm.detach_current_thread().unwrap();
        }
    }
}

pub trait GCObject {}

pub trait GCHandle {}

pub struct JvmActorHandle {}

/// Provides additional arbiter services for Mojo related things.
/// In charge of attaching to JVM or .NET Core.
pub struct MojoArbiter {
    state: AppState,
    system: System,
    arbiter: Addr<Arbiter>,
    thread: std::thread::Thread,
    jvm: Option<JVM>,
}

//impl Clone for MojoArbiter {
//    fn clone(&self) -> Self {
//        MojoArbiter {
//            state: AppState{},
//            system: self.system.clone(),
//        }
//    }
//}

impl MojoArbiter {
    pub fn new(state: AppState) -> MojoArbiter {
        MojoArbiter {
            state,
            system: actix::System::current(),
            arbiter: actix::Arbiter::current(),
            thread: std::thread::current(),
            jvm: None,
        }
    }

    pub fn init(&mut self) {
        // Attach thread.
    }

    pub fn attach_jvm(&mut self, vm: crate::jni::JavaVM) {
        self.jvm = Some(JVM {
            vm,
            thread: std::thread::current(),
        });
    }

    // TODO: Attach
}

impl Actor for MojoArbiter {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut <Self as Actor>::Context) {
        println!("mojo arbiter[ {:?} ] started", self.thread.id());
    }

    fn stopping(&mut self, ctx: &mut <Self as Actor>::Context) -> Running {
        println!("mojo arbiter[ {:?} ] stopping...", self.thread.id());
        Running::Stop
    }

    fn stopped(&mut self, ctx: &mut <Self as Actor>::Context) {
        println!("mojo arbiter[ {:?} ] stopped", self.thread.id());
    }
}

pub struct JvmAttach {
    vm: crate::jni::JavaVM
}

impl actix::Message for JvmAttach {
    type Result = ();
}

impl actix::Handler<JvmAttach> for MojoArbiter {
    type Result = ();

    fn handle(&mut self, msg: JvmAttach, ctx: &mut Self::Context) {
        self.attach_jvm(msg.vm);
    }
}