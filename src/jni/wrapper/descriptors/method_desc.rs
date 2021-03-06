use crate::jni::wrapper::errors::*;

use crate::jni::wrapper::descriptors::Desc;

use crate::jni::wrapper::objects::JClass;
use crate::jni::wrapper::objects::JMethodID;
use crate::jni::wrapper::objects::JStaticMethodID;

use crate::jni::wrapper::strings::JNIString;

use crate::jni::wrapper::JNIEnv;

impl<'a, 'c, T, U, V> Desc<'a, JMethodID<'a>> for (T, U, V)
where
    T: Desc<'a, JClass<'c>>,
    U: Into<JNIString>,
    V: Into<JNIString>,
{
    fn lookup(self, env: &JNIEnv<'a>) -> Result<JMethodID<'a>> {
        env.get_method_id(self.0, self.1, self.2)
    }
}

impl<'a, 'c, T, Signature> Desc<'a, JMethodID<'a>> for (T, Signature)
where
    T: Desc<'a, JClass<'c>>,
    Signature: Into<JNIString>,
{
    fn lookup(self, env: &JNIEnv<'a>) -> Result<JMethodID<'a>> {
        (self.0, "<init>", self.1).lookup(env)
    }
}

impl<'a, 'c, T, U, V> Desc<'a, JStaticMethodID<'a>> for (T, U, V)
where
    T: Desc<'a, JClass<'c>>,
    U: Into<JNIString>,
    V: Into<JNIString>,
{
    fn lookup(self, env: &JNIEnv<'a>) -> Result<JStaticMethodID<'a>> {
        env.get_static_method_id(self.0, self.1, self.2)
    }
}
