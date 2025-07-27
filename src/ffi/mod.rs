pub mod disk_arbitration;

// https://codebrowser.dev/glibc/glibc/mach/mach/error.h.html
pub(in crate::ffi) const fn err_system(x: u32) -> u32 {
    (x & 0x3f) << 26
}

pub(in crate::ffi) const fn err_sub(x: u32) -> u32 {
    (x & 0xfff) << 14
}
