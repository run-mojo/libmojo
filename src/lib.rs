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
use std::sync::{Arc, mpsc, Mutex};
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
        let mut arbiter = MojoArbiter::new(AppState {});

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


pub struct MojoApp {
    loops: Vec<std::sync::Arc<Addr<MojoArbiter>>>,
}

impl MojoApp {
    pub fn new() -> MojoApp {
        MojoApp {
            loops: Vec::with_capacity(128), // TODO: num_cpus
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

/// Single-threaded handle to a JavaVM. Detaches thread when dropped.
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


////////////////////////////////////////////////////////////////////////////////
// MojoActor
////////////////////////////////////////////////////////////////////////////////

///
pub enum MojoActorProxy {
    Nil,
    // Java uses "JNIEnv->NewGlobalRef()" to pin in the GC.
    // When an actor is stopped this pin is released allowing the GC to
    // collect as long as the app has no other references. Standard practice
    // is to never share references to the Actor object itself.
    // The number of actor proxies is usually low and mostly have 'static
    // lifetimes / live for the duration of the app. A common pattern is the
    // Action pattern with a "reset()" state instead of stopping and starting
    // a new actor.
    Jvm(jvm::GCHandle),
    // GCHandle is used to pin inside GC.
    // .NET has value types which allows for a very lightweight integration.
    Net(usize),
    // Go can use "unsafe.Pointer"
    Go(usize),
    // Swift can use raw pointers.
    Swift(usize),
}


/// General purpose
pub struct MojoActor {
    handle: MojoActorProxy
}

impl Actor for MojoActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut <Self as Actor>::Context) {
        match self.handle {
            MojoActorProxy::Jvm(ref mut handle) => {
                // TODO: call-> run.mojo.actor.MetalActor.started0()
                if let Some((env, obj)) = handle.as_obj_env() {
//env.get_method_id()
//                    env.call_method_unsafe(obj, )
                } else {
                    // TODO: Wtf? Let's crash the Actor and let the supervisor restart if necessary.
                }
            }
            _ => {}
        }
        // TODO: Call managed lifecycle.
        unimplemented!()
    }

    fn stopping(&mut self, ctx: &mut <Self as Actor>::Context) -> Running {
        unimplemented!()
    }

    fn stopped(&mut self, ctx: &mut <Self as Actor>::Context) {
        unimplemented!()
    }
}

////////////////////////////////////////////////////////////////////////////////
// MojoActorMessage
////////////////////////////////////////////////////////////////////////////////

///
pub enum MojoActorMessage {
    Nil,
    Jvm(jvm::GCHandle),
    Net(u64),
    Bytes(bytes::Bytes),
    BytesMut(bytes::BytesMut),
}

///
impl actix::Message for MojoActorMessage {
    type Result = MojoActorMessage;
}


////////////////////////////////////////////////////////////////////////////////
// MojoActorRequest
////////////////////////////////////////////////////////////////////////////////

/// Requests create an Async MessageResponse.
pub enum MojoActorRequest {
    Jvm(jvm::GCHandle),
//    Bytes(bytes::Bytes),
//    BytesMut(bytes::BytesMut),
}

impl MojoActorRequest {
    pub fn invoke(&self) {
        match *self {
            MojoActorRequest::Jvm(ref handle) => {}
            _ => {}
        }
    }
}


///
pub enum MojoActorResult {
    Nil,
    Jvm(jvm::GCHandle),
}

///
impl actix::Message for MojoActorRequest {
    type Result = Result<MojoActorResult, ()>;
}


///
impl actix::Handler<MojoActorRequest> for MojoActor {
    type Result = Response<MojoActorResult, ()>;

    fn handle(&mut self, msg: MojoActorRequest, ctx: &mut Context<Self>) -> Self::Result {
        match msg {
            MojoActorRequest::Jvm(handle) => {
                // Invoke.

                // Create oneshot channel.
                let (sender, receiver) =
                    futures::oneshot::<Self::Result>();

                // Listen for result.
                return Response::future(receiver
                    .map(|r| MojoActorResult::Nil)
                    .map_err(|e| ())
                );
            }
            _ => {}
        }
//        unimplemented!("");
        Response::reply(Ok(MojoActorResult::Nil))
    }
}


enum ResponseTypeItem<I, E> {
    Result(Result<I, E>),
    Fut(Box<Future<Item=I, Error=E>>),
}

/// Helper type for representing different type of message responses
pub struct Response<I, E> {
    item: ResponseTypeItem<I, E>,
}

impl<I, E> Response<I, E> {
    /// Create async response
    pub fn future<T>(fut: T) -> Self
        where
            T: Future<Item=I, Error=E> + 'static {
        Response {
            item: ResponseTypeItem::Fut(Box::new(fut)),
        }
    }

    /// Create response
    pub fn reply(val: Result<I, E>) -> Self {
        Response {
            item: ResponseTypeItem::Result(val),
        }
    }
}

impl<A, M, I: 'static, E: 'static> MessageResponse<A, M> for Response<I, E>
    where
        A: Actor,
        M: Message<Result=Result<I, E>>,
        A::Context: AsyncContext<A>,
{
    fn handle<R: ResponseChannel<M>>(self, _: &mut A::Context, tx: Option<R>) {
        match self.item {
            ResponseTypeItem::Fut(fut) => {
                Arbiter::spawn(fut.then(move |res| {
                    if let Some(tx) = tx {
                        tx.send(res);
                    }
                    Ok(())
                }));
            }
            ResponseTypeItem::Result(res) => {
                if let Some(tx) = tx {
                    tx.send(res);
                }
            }
        }
    }
}


//enum ActorResponseTypeItem<A, I, E> {
//    Result(Result<I, E>),
//    Fut(Box<ActorFuture<Item = I, Error = E, Actor = A>>),
//}
//
///// Helper type for representing different type of message responses
//pub struct ActorResponse<A, I, E> {
//    item: ActorResponseTypeItem<A, I, E>,
//}
//
//impl<A: Actor, I, E> ActorResponse<A, I, E> {
//    /// Create response
//    pub fn reply(val: Result<I, E>) -> Self {
//        ActorResponse {
//            item: ActorResponseTypeItem::Result(val),
//        }
//    }
//
//    /// Create async response
//    pub fn later<T>(fut: T) -> Self
//        where
//            T: ActorFuture<Item = I, Error = E, Actor = A> + 'static,
//    {
//        ActorResponse {
//            item: ActorResponseTypeItem::Fut(Box::new(fut)),
//        }
//    }
//}
//
//impl<A, M, I: 'static, E: 'static> MessageResponse<A, M> for ActorResponse<A, I, E>
//    where
//        A: Actor,
//        M: Message<Result = Result<I, E>>,
//        A::Context: AsyncContext<A>,
//{
//    fn handle<R: ResponseChannel<M>>(self, ctx: &mut A::Context, tx: Option<R>) {
//        match self.item {
//            ActorResponseTypeItem::Fut(fut) => {
//                ctx.spawn(fut.then(move |res, _, _| {
//                    if let Some(tx) = tx {
//                        tx.send(res)
//                    }
//                    actix::fut::ok(())
//                }));
//            }
//            ActorResponseTypeItem::Result(res) => {
//                if let Some(tx) = tx {
//                    tx.send(res);
//                }
//            }
//        }
//    }
//}
