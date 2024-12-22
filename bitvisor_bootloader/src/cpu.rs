use core::arch::asm;

/// Halt cpu
/// stop the cpu
#[inline(always)]
pub fn halt_loop() -> ! {
    loop {
        unsafe { asm!("hlt")};
    }
}