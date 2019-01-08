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

#[lang = "eh_personality"]
extern fn eh_personality() {}
