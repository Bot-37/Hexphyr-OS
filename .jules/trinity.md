## 2024-05-24 - Unsafe ABI pointer dereference in kernel entry
**Domain:** Security
**Learning:** The kernel's `_uefi_start` function accepted a raw pointer from the bootloader and dereferenced it. Although `is_null()` was checked, the function was exported as `pub extern "C"` but lacked the `unsafe` keyword, violating Rust's memory safety guarantees for external APIs.
**Action:** Always mark public functions accepting and dereferencing raw pointers as `unsafe`, and ensure caller contracts are documented or internal safety boundaries are strict.
