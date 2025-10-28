// Raw FFI bindings to libnfqws.a

use std::ffi::c_char;

#[link(name = "nfqws", kind = "static")]
unsafe extern "C" {
    #[link_name = "main"]
    pub fn nfqws_main(argc: libc::c_int, argv: *const *const c_char) -> libc::c_int;
}
