#![no_std]
#![no_main]

extern crate core;
extern crate alloc;

use polkavm_derive::polkavm_export;
use core::fmt::Write;

#[repr(u32)]
pub enum Foo {
    Success = 0,
    CalleeTrapped = 1,
    Unknown,
}

impl ::core::fmt::Debug for Foo {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::write_str(
            f,
            match self {
                Foo::Success => "Success",
                Foo::CalleeTrapped => "CalleeTrapped",
                Foo::Unknown => "Unknown",
            },
        )
    }
}

struct Writer;
impl core::fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        unsafe {
            crate::debug_message(s.as_ptr(), s.len() as u32);
        }
        Ok(())
    }
}

#[polkavm_derive::polkavm_import]
extern "C" {
    pub fn debug_message(str_ptr: *const u8, str_len: u32);
}

#[polkavm_export(abi = polkavm_derive::default_abi)]
pub fn call() { }

#[polkavm_export(abi = polkavm_derive::default_abi)]
pub fn deploy() {
    {
        // on heap
        let foo = alloc::format!("{:?}", Foo::Success);
        unsafe {
            crate::debug_message(foo.as_ptr(), foo.len() as u32);
        }

        // on stack
        let mut m = Writer {};
        let _ = write!(&mut m, "bug: _{:?}_",
               Foo::Success);
    };
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe {
        core::arch::asm!("unimp");
        core::hint::unreachable_unchecked();
    }
}

use talc::*;
static mut ARENA: [u8; 10000] = [0; 10000];

#[global_allocator]
static ALLOCATOR: Talck<spin::Mutex<()>, ClaimOnOom> = Talc::new(unsafe {
    ClaimOnOom::new(Span::from_array(core::ptr::addr_of!(ARENA).cast_mut()))
}).lock();


