use actix::dev::{MessageResponse, ResponseChannel};
use actix::prelude::*;
//use actix::system::{SystemRunner};
use actix_redis::RedisSessionBackend;
use actix_web::{App, Body, dev::Handler, error, Error, Error as ActixWebError, Form, HttpRequest, HttpResponse, middleware, Responder, Result, server};
use actix_web::{AsyncResponder, Either};
use actix_web::middleware::session::{self, RequestSession};
use futures::Future;
use futures::future::result;
use crate::jni::JNIEnv;
// These objects are what you should use as arguments to your native function.
// They carry extra lifetime information to prevent them escaping this context
// and getting used after being GC'd.
use crate::jni::objects::{GlobalRef, JClass, JObject, JString, JValue};
use crate::jni::signature::JavaType;
// This is just a pointer. We'll be returning it from our function.
// We can't return one of the objects with lifetime information because the
// lifetime checker won't let us.
use crate::jni::sys::{jbyteArray, jint, jlong, jstring, jobject};
use crate::jni::{JavaVM};
use std::mem;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;


pub type GCHandle = GlobalRef;

pub type JavaCallback = Box<FnOnce()>;



////////////////////////////////////////////////////////////////////////////////
// HttpResponse Body
////////////////////////////////////////////////////////////////////////////////

pub struct JavaBody {
    inner: Body,
}


////////////////////////////////////////////////////////////////////////////////
// HttpHandler
////////////////////////////////////////////////////////////////////////////////

pub struct JavaHttpHandler {
//    arbiter: super::MojoArbiter,
    handler: Option<GlobalRef>
}

impl JavaHttpHandler {
    pub fn new() -> JavaHttpHandler {
        JavaHttpHandler {
            handler: None
        }
    }
}

impl<S> Handler<S> for JavaHttpHandler {
    type Result = Either<
        HttpResponse,
        Box<Future<
            Item=HttpResponse,
            Error=Error>>>;

    fn handle(&self, req: &HttpRequest<S>) -> <Self as Handler<S>>::Result {
        // Invoke Java handler.
//        self.handler.as_obj();
        // Invoke "run.mojo.http.HttpHandler.handle()"

        // Return response?
        if !is_async_result() {
            return Either::A(HttpResponse::BadRequest().body("Bad data"))
        }

        // Create oneshot channel.
        let (sender, receiver) =
            futures::oneshot::<HttpResponse>();

//        Arbiter::spawn(futures::lazy(move || {
//            tokio_timer::Delay::new(
//                std::time::Instant::now() +
//                    std::time::Duration::new(2, 0)
//            ).map(move |r| {
//                println!("Timeout!");
//                sender.send(HttpResponse::Ok()
//                    .content_type("text/html")
//                    .body("Hello!"));
//                ()
//            }).map_err(|e| ())
//        }));

        // Return oneshot receiver as the future.
        Either::B(
            // Convert into AsyncResponder.
            receiver
                .map_err(|err| err.into())
                .responder(),
        )
    }
}

fn is_async_result() -> bool {
    false
}


////////////////////////////////////////////////////////////////////////////////
// Futures
////////////////////////////////////////////////////////////////////////////////

pub struct JavaFuture {
    inner: GCHandle,
}

impl JavaFuture {
    pub fn complete(&mut self) {}

    pub fn exception(&mut self) {}
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_run_mojo_future_Futures_alloc(
    env: JNIEnv,
    _class: JClass,
    state: JObject,
) -> jlong {
    // Must attach the JVM to the current worker thread.
//    env
//        .get_java_vm()
//        .expect("expected get_java_vm() to return JavaVM handle")
//        .attach_current_thread()
//        .expect("could not attach JVM to current thread");
//
//    let state = env.new_global_ref(state).unwrap();
//
//    unsafe {
//        let app = Box::leak(Box::new(App::with_state(state)));
//        app as *mut _ as usize as jlong
//    }
    0
}



////////////////////////////////////////////////////////////////////////////////
// Actor
////////////////////////////////////////////////////////////////////////////////

// Use a generic Actor Type which handles invoking methods on a GCHandle.


enum ActorMessage {
    Java,
    NETCore,
    Deno,
    Node,
}


enum Messages {
    Ping,
    Pong,
}

enum Responses {
    GotPing,
    GotPong,
}

impl<A, M> MessageResponse<A, M> for Responses
    where
        A: Actor,
        M: Message<Result=Responses>,
{
    fn handle<R: ResponseChannel<M>>(self, _: &mut A::Context, tx: Option<R>) {
        if let Some(tx) = tx {
            tx.send(self);
        }
    }
}



//#[derive(actix::Message)]
pub struct Ping;

impl actix::Message for Ping {
    type Result = ();
}

impl actix::Handler<Ping> for Timer {
    type Result = ();

    fn handle(&mut self, msg: Ping, ctx: &mut Context<Timer>) {
        println!("ping! from {:?}", std::thread::current().id());
        ()
    }
}

pub struct Timer {
    pub dur: Duration,
//    sender: std::cell::RefCell<futures::sync::oneshot::Sender<HttpResponse>>,
}

impl Actor for Timer {
    type Context = Context<Self>;

    // stop system after `self.dur` seconds
    fn started(&mut self, ctx: &mut Context<Self>) {
        ctx.add_message_stream(
            futures::stream::once(Ok(Ping))
        );

        ctx.run_later(
            Duration::new(1, 0),
            |act: &mut Timer, ctx| {
                ctx.notify(Ping);
            });
    }
}


impl Message for Messages {
    type Result = Responses;
}

/// Define handler for `Messages` enum
impl actix::Handler<Messages> for Timer {
    type Result = Responses;

    fn handle(&mut self, msg: Messages, ctx: &mut Context<Self>) -> Self::Result {
        match msg {
            Messages::Ping => Responses::GotPing,
            Messages::Pong => Responses::GotPong,
        }
    }
}
