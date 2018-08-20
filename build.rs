extern crate gcc;

fn main() {
    gcc::Build::new()
        .file("src/c/endianconv.c")
        .file("src/c/zmalloc.c")
        .file("src/c/listpack.c")
        .file("src/c/rax.c")
        .file("src/c/rax_ext.c")
        .file("src/c/sds.c")
        .file("src/c/sds_ext.c")
        .file("src/c/siphash.c")
        .file("src/c/sha1.c")
        .file("src/c/dict.c")
        .file("src/c/util.c")
        .file("src/c/object.c")
//        .file("src/c/stream.c")
        .include("src/c/")
        .compile("libcmojo.a");
}