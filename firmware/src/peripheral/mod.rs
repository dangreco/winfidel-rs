pub mod led;

/// # Safety
/// Each peripheral field must be read exactly once across the program.
pub unsafe fn take<T>(field: &T) -> T {
    unsafe { core::ptr::read(field) }
}
