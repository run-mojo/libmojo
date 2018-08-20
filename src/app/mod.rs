use actix::dev::{MessageResponse, ResponseChannel};
use actix::prelude::*;
//use actix::system::{SystemRunner};
use actix_redis::RedisSessionBackend;
use actix_web::{App, error, Error, Error as ActixWebError, Form, HttpRequest, HttpResponse, middleware, Responder, Result, server};
use actix_web::{AsyncResponder, Either};
use actix_web::middleware::session::{self, RequestSession};
use bytes;
use crate::jni::JNIEnv;
// These objects are what you should use as arguments to your native function.
// They carry extra lifetime information to prevent them escaping this context
// and getting used after being GC'd.
use crate::jni::objects::{GlobalRef, JClass, JObject, JString, JValue};
use crate::jni::signature::JavaType;
// This is just a pointer. We'll be returning it from our function.
// We can't return one of the objects with lifetime information because the
// lifetime checker won't let us.
use crate::jni::sys::{jbyteArray, jint, jlong, jstring};
use futures;
use futures::Future;
use futures::future::result;
use h2;
use http;
use std::mem;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;


pub struct NoHandler;

impl server::HttpHandler for NoHandler {
    type Task = Box<server::HttpHandlerTask>;

    fn handle(&self, req: server::Request) -> Result<<Self as server::HttpHandler>::Task, server::Request> {
        panic!("this handler should never be handled")
    }
}

pub struct AppState {
    inner: Option<Box<dyn GCHandle>>,
}

impl AppState {
//    pub fn new(managed: impl ManagedObject) -> AppState {
//        AppState {
//            managed,
//        }
//    }
}

pub struct Worker {
    system: actix::System,

}

/// 'ManagedObject' is a global reference held by Rust for an object
/// in a managed environment that is registered with it's GC.
pub trait GCHandle: 'static + Send {}


pub struct JavaGCHandle {
    inner: Option<GlobalRef>
}

impl GCHandle for JavaGCHandle {}


///
pub trait ManagedCallback: 'static + Send {}


///
pub struct MojoRegistry {}

struct AppBuilder {
    inner: Option<App<AppState>>,
}


/////////////////////////////////////////////////////////////////////////////
// JVM
/////////////////////////////////////////////////////////////////////////////

fn greet(req: &HttpRequest<AppState>) -> impl Responder {
    req.state();
//    println!("thread: {}", std::thread::current().name().unwrap());
    let to = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", to)
}

fn index(body: bytes::Bytes) -> Result<bytes::Bytes> {
    Ok(body)
}

fn is_error() -> bool {
    true
}

fn index2(req: &HttpRequest<AppState>) -> Result<Box<Future<Item=HttpResponse, Error=Error>>, Error> {
    if is_error() {
        Err(actix_web::error::ErrorBadRequest("bad request"))
    } else {
        Ok(Box::new(
            result(Ok(HttpResponse::Ok()
                .content_type("text/html")
                .body(format!("Hello!"))))))
    }
}

pub struct EitherHandler {

}

type EitherResult = Either<
    HttpResponse,
    Box<Future<
        Item=HttpResponse,
        Error=Error>>>;

//impl Responder for EitherHandler {
//    type Item = EitherResult;
//    type Error = Error;
//
//    fn respond_to(self, req: & HttpRequest<AppState>) -> Result<<Self as Responder>::Item, <Self as Responder>::Error> {
//        unimplemented!()
//    }
//}

fn either_handler(req: &HttpRequest<AppState>) -> EitherResult {
//    let j:AsyncResponder
    let (sender, receiver) = futures::oneshot::<HttpResponse>();

    sender.send(HttpResponse::Ok()
                .content_type("text/html")
                .body("Hello!"));

    if is_a_variant() {
        // <- choose variant A
        Either::A(HttpResponse::BadRequest().body("Bad data"))
    } else {
        Either::B(
            receiver
                .and_then(|r| {
                    result(Ok(HttpResponse::Ok()
                        .content_type("text/html")
                        .body("Hello!")))
                })
                .map_err(|err: futures::Canceled| err.into())
                .responder(),
        )
    }
}

fn is_a_variant() -> bool { true }

//fn build_either_fn() -> Fn()

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_run_mojo_app_Native_start(
    env: JNIEnv,
    _class: JClass,
    bind: JString,
    system_name: JString,
    workers: jint,
    backlog: jint,
    hostname: JString,
    system_exit: jint,
    disable_signals: jint,
    shutdown_timeout: jint,
    no_http2: jint,
    app_factory: JObject,
) -> jlong {
    let (tx, rx) = mpsc::channel();

    // We need to obtain global reference to the `callback` object before sending
    // it to the thread, to prevent it from being collected by the GC.
    let app_factory = env.new_global_ref(app_factory).unwrap();
    let app_factory_ptr = unsafe {
        &app_factory as *const GlobalRef as usize
    };

    let env_ptr = unsafe {
        env.internal as usize
    };


    let system_name: String = match env.get_string(system_name) {
        Ok(v) => v.into(),
        Err(_) => "mojo".into()
    };

    // Set bind address.
    let bind: String = match env.get_string(bind) {
        Ok(v) => v.into(),
        Err(_) => "localhost:8000".into()
    };

    // Set bind address.
    let hostname: Option<String> = match env.get_string(hostname) {
        Ok(v) => Some(v.into()),
        Err(_) => None
    };

    // `JNIEnv` cannot be sent across thread boundaries. To be able to use JNI
    // functions in other threads, we must first obtain the `JavaVM` interface
    // which, unlike `JNIEnv` is `Send`.
    let jvm_ptr = env.get_java_vm().unwrap().as_ptr() as usize;

    thread::spawn(move || {
        let jvm = unsafe {
            let jvm_ptr = jvm_ptr as *mut crate::jni::sys::JavaVM;
            crate::jni::JavaVM::from_raw(
                jvm_ptr
            ).unwrap()
        };

        let attach_guard = jvm.attach_current_thread().unwrap();

        // Create a new SystemRunner.
        let runner = actix::System::new(system_name);

        // Shim the AppFactory.
        let mut svr = server::new(move || {
            unsafe {
                // Attach thread to JVM.
                let jvm = unsafe {
                    let jvm_ptr = jvm_ptr as *mut crate::jni::sys::JavaVM;
                    crate::jni::JavaVM::from_raw(
                        jvm_ptr
                    ).unwrap()
                };
                let attach_guard = jvm.attach_current_thread().unwrap();
                let app_factory = unsafe { &*(app_factory_ptr as *const GlobalRef) };


                let mut app = Box::new(AppBuilder { inner: None });
                let app_ptr = unsafe { &mut app as *mut _ as usize as jlong };

                let state = match attach_guard.get_method_id(
                    "run/mojo/app/WorkerBuilder",
                    "init",
                    "(J)Ljava/lang/Object;",
                ) {
                    Ok(found) => {
                        println!("Found it!");
                        match attach_guard.call_method_unsafe(
                            app_factory.as_obj(),
                            found,
                            JavaType::Object("java/lang/Object".into()),
                            &[JValue::Long(app_ptr)],
//                            &[JValue::Long(1)]
                        ) {
                            Ok(res) => match res {
                                JValue::Object(obj) => {
                                    AppState {
                                        inner: Some(Box::new(JavaGCHandle {
                                            inner: Some(attach_guard.new_global_ref(obj).unwrap())
                                        }))
                                    }
                                }
                                _ => {
                                    AppState {
                                        inner: Some(Box::new(JavaGCHandle {
                                            inner: None
                                        }))
                                    }
                                }
                            },
                            Err(_) => panic!("Doh!")
                        }
                    }
                    Err(_) => {
                        panic!("Could not find method 'init'");
                    }
                };

                app.inner = Some(App::with_state(state));

//                let mut app: App<()> = std::mem::zeroed();

//                let state = match attach_guard
//                    .call_method(
//                        app_factory.as_obj(),
//                        "hi",
//                        "(J)Ljava/lang/Object;",
////                        &[JValue::Long(unsafe { &app as *const _ as usize as jlong })],
//                        &[JValue::Long(2)],
//                    ) {
//                    Ok(state) => match state {
//                        JValue::Object(obj) => match attach_guard.new_global_ref(obj) {
//                            Ok(global_ref) => Some(global_ref),
//                            Err(_) => None
//                        },
//                        _ => None
//                    },
//                    Err(_) => None
//                };

//                app = App::with_state(JavaAppState {
//                    inner: state,
//                });

//                app = App::with_state(());

//                match env.call_method(app_factory.as_obj(), "build", "()V", &[]) {
//                    Ok(_) => println!("WorkerBuilder.build() success"),
//                    Err(_) => println!("WorkerBuilder.build() error")
//                }

                let app: App<AppState> = app.inner.unwrap();

                app
                    .resource("/", |r| {
                        r
                            .method(http::Method::GET)
                            .with_config(index, |cfg| {
                                cfg.limit(4096);
                            })
//                        r.f(greet)
                    })
                    .resource("/g", |r| {
                        r
                            .method(http::Method::GET)
                            .f(either_handler)
                    })

//                App::with_state(state)
//                    .resource("/", |r| {
//                        r.f(greet)
//                    })
//                    .resource("/", |r| r.f(greet))
//                    .resource("/{name}", |r| r.f(greet))
            }
        });

        if workers > 0 {
            svr = svr.workers(workers as usize);
        }
        if backlog > 0 {
            svr = svr.backlog(backlog);
        }
        if system_exit > 0 {
            svr = svr.system_exit();
        }
        if disable_signals > 0 {
            svr = svr.disable_signals();
        }
        if shutdown_timeout > 0 {
            svr = svr.shutdown_timeout(shutdown_timeout as u16);
        }
        if no_http2 > 0 {
            svr = svr.no_http2();
        }

        svr = svr
            .bind(bind)
            .expect("failed to bind to port");

        if hostname.is_some() {
            svr = svr.server_hostname(hostname.unwrap());
        }

        let addr = svr.start();

        let _ = tx.send(addr);
        let _ = runner.run();
    });

    // Return Addr<HttpServer> handle.
    unsafe { (Box::leak(Box::new(rx.recv().unwrap().clone()))) as *const _ as usize as jlong }
}


#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_run_mojo_app_Native_stop(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
    graceful: jint,
    wait: jint,
) {
    // Must attach the JVM to the current worker thread.
    env
        .get_java_vm()
        .expect("expected get_java_vm() to return JavaVM handle")
        .attach_current_thread()
        .expect("could not attach JVM to current thread");

    unsafe {
//        let addr: *mut Addr<server::HttpServer<DummyHandler>> = mem::transmute(handle);
        // Convert back into a Box and drop when finished.
        let addr: Box<Addr<server::HttpServer<NoHandler>>> =
            Box::from_raw(mem::transmute(handle));
        let r = (&*addr).send(server::StopServer { graceful: graceful > 0 });
        if wait > 0 {
            r.wait();
        }
    }
}


#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_run_mojo_app_Native_appNew(
    env: JNIEnv,
    _class: JClass,
    state: JObject,
) -> jlong {
    // Must attach the JVM to the current worker thread.
    env
        .get_java_vm()
        .expect("expected get_java_vm() to return JavaVM handle")
        .attach_current_thread()
        .expect("could not attach JVM to current thread");

    let state = env.new_global_ref(state).unwrap();

    unsafe {
        let app = Box::leak(Box::new(App::with_state(state)));
        app as *mut _ as usize as jlong
    }
}


struct AppResource {
    path: String,
}

/////////////////////////////////////////////////////////////////////////////
// CoreCLR
/////////////////////////////////////////////////////////////////////////////