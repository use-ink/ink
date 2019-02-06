use core::panic::PanicInfo;
use core::alloc::Layout;

#[panic_handler]
#[no_mangle]
pub fn panic(_info: &PanicInfo) -> ! {
	unsafe { core::intrinsics::abort() }
}

#[alloc_error_handler]
pub extern fn oom(_: Layout) -> ! {
	unsafe { core::intrinsics::abort(); }
}

/// This is only required in non wasm32-unknown-unknown targets.
///
/// Since pdsl_core is targeted for wasm32-unknown-unknown we should
/// maybe remove this.
#[lang = "eh_personality"]
extern fn eh_personality() {}
