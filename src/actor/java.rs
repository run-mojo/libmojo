use actix::dev::{MessageResponse, ResponseChannel};
use actix::prelude::*;
use actix_redis::RedisSessionBackend;
use actix_web::{App, HttpRequest, HttpResponse, middleware, Responder, Result, server};
use actix_web::middleware::session::{self, RequestSession};
use futures::Future;

use crate::jni::JNIEnv;
// These objects are what you should use as arguments to your native function.
// They carry extra lifetime information to prevent them escaping this context
// and getting used after being GC'd.
use crate::jni::objects::{GlobalRef, JClass, JObject, JString};
// This is just a pointer. We'll be returning it from our function.
// We can't return one of the objects with lifetime information because the
// lifetime checker won't let us.
use crate::jni::sys::{jbyteArray, jint, jlong, jstring};

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

//pub struct JavaActor(GlobalRef);
pub struct JavaActor(Option<JavaBox>);

pub struct JavaAct(Option<GlobalRef>);

pub struct JavaBox(GlobalRef);


pub type JavaRecipient = Recipient<JavaBox>;

impl Actor for JavaActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut <Self as Actor>::Context) {
        println!("JavaActor started");


//        self.0 = Some(ctx.run_interval(Duration::new(2, 0), |act, ctx| {
//            println!("interval");
//            ctx.cancel_future(act.0.unwrap());
//            act.0 = None;
//        }));

        // TODO: Invoke java.
    }

    fn stopping(&mut self, ctx: &mut <Self as Actor>::Context) -> Running {
        // TODO: Invoke java.

        Running::Stop
    }


    fn stopped(&mut self, ctx: &mut <Self as Actor>::Context) {
        println!("JavaActor stopped");

        // TODO: Invoke java.
    }
}

impl Message for JavaBox {
    type Result = Result<bool, std::io::Error>;
}

impl Handler<JavaBox> for JavaActor {
    type Result = Result<bool, std::io::Error>;

    fn handle(
        &mut self,
        msg: JavaBox,
        ctx: &mut <Self as Actor>::Context,
    ) -> <Self as Handler<JavaBox>>::Result {

        // TODO: Invoke java.

        unimplemented!()
    }
}