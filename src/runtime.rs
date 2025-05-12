// Copyright (c) Viable Systems and TezEdge Contributors
// SPDX-License-Identifier: MIT

use ocaml_boxroot_sys::boxroot_teardown;
use std::{
    cell::UnsafeCell,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use crate::{memory::OCamlRef, value::OCaml};

thread_local! {
  static TLS_RUNTIME: UnsafeCell<OCamlRuntime> = const { UnsafeCell::new({
      OCamlRuntime { _not_send_sync: PhantomData }
  })};
}

/// RAII guard for the OCaml runtime.
pub struct OCamlRuntimeStartupGuard {
    _not_send_sync: PhantomData<*const ()>,
}

impl Deref for OCamlRuntimeStartupGuard {
    type Target = OCamlRuntime;

    fn deref(&self) -> &OCamlRuntime {
        unsafe { internal::recover_runtime_handle() }
    }
}

impl DerefMut for OCamlRuntimeStartupGuard {
    fn deref_mut(&mut self) -> &mut OCamlRuntime {
        unsafe { internal::recover_runtime_handle_mut() }
    }
}

/// Per-thread handle to the OCaml runtime.
///
/// The first call to `OCamlRuntime::init()` on the “main” thread
/// will perform `caml_startup` and initialize the runtime. The
/// returned `OCamlRuntimeStartupGuard`, once dropped, will
/// perform the OCaml runtime shutdown and release resources.
///
/// In normal use you don’t pass this handle around yourself—invoke
/// `OCamlRuntime::with_domain_lock(...)` (or use the provided FFI
/// export macros) to enter the OCaml domain and get a `&mut` to it.
pub struct OCamlRuntime {
    _not_send_sync: PhantomData<*const ()>,
}

impl OCamlRuntime {
    /// Initialize the OCaml runtime exactly once.
    ///
    /// Returns a `OCamlRuntimeStartupGuard` that will perform the
    /// OCaml runtime shutdown and release resources once dropped.
    ///
    /// Returns `Err(String)` if called more than once.
    pub fn init() -> Result<OCamlRuntimeStartupGuard, String> {
        #[cfg(not(feature = "no-caml-startup"))]
        {
            use std::sync::atomic::{AtomicBool, Ordering};

            static INIT_CALLED: AtomicBool = AtomicBool::new(false);

            if INIT_CALLED
                .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                .is_err()
            {
                return Err("OCaml runtime already initialized".to_string());
            }
            unsafe {
                let arg0 = c"ocaml".as_ptr() as *const ocaml_sys::Char;
                let args = [arg0, core::ptr::null()];
                ocaml_sys::caml_startup(args.as_ptr());
                ocaml_boxroot_sys::boxroot_setup();
                ocaml_sys::caml_enter_blocking_section();
            }

            Ok(OCamlRuntimeStartupGuard {
                _not_send_sync: PhantomData,
            })
        }
        #[cfg(feature = "no-caml-startup")]
        return Err(
            "Rust code called from OCaml should not try to initialize the runtime".to_string(),
        );
    }

    /// Release the OCaml runtime lock, call `f`, and re-acquire the OCaml runtime lock.
    pub fn releasing_runtime<T, F>(&mut self, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        OCamlBlockingSection::new().perform(f)
    }

    /// Returns the OCaml valued to which this GC tracked reference points to.
    pub fn get<'tmp, T>(&'tmp self, reference: OCamlRef<T>) -> OCaml<'tmp, T> {
        OCaml {
            _marker: PhantomData,
            raw: unsafe { reference.get_raw() },
        }
    }

    /// Run f with the OCaml lock held (enter / leave automatically).
    ///
    /// This is a blocking call that will wait until the OCaml runtime is available.
    pub fn with_domain_lock<F, T>(f: F) -> T
    where
        F: FnOnce(&mut Self) -> T,
    {
        let mut lock = OCamlDomainLock::new();
        f(&mut lock)
    }
}

impl Drop for OCamlRuntimeStartupGuard {
    fn drop(&mut self) {
        unsafe {
            boxroot_teardown();
            ocaml_sys::caml_shutdown();
        }
    }
}

struct OCamlBlockingSection;

impl OCamlBlockingSection {
    fn new() -> Self {
        Self
    }

    fn perform<T, F>(self, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        unsafe { ocaml_sys::caml_enter_blocking_section() };
        f()
    }
}

impl Drop for OCamlBlockingSection {
    fn drop(&mut self) {
        unsafe { ocaml_sys::caml_leave_blocking_section() };
    }
}

/// In OCaml 5 each **domain** has its own minor-heap and GC state.  Entering
/// OCaml from C requires the thread to *leave* the “blocking section”, thereby
/// resuming normal allocation/GC activity for **this domain**.  This guard
/// performs that transition in `new()` and restores the blocking section in
/// `Drop`.
///
/// While the guard is alive the current OS thread is the **only** thread that
/// can run OCaml code *inside this domain*.  That exclusivity makes it sound
/// to hand out **one** mutable reference to the process-wide
/// [`OCamlRuntime`] but only for the guard’s lifetime.
///
/// # Safety invariant
///
/// *Each* live `OCamlDomainLock` owns the “entered” state of the current
/// domain.  Creating a second guard simultaneously (nesting) would yield two
/// overlapping `&mut OCamlRuntime` borrows (that is undefined behaviour in
/// Rust) and would also violate the enter/leave protocol required by the OCaml
/// C API.  Likewise, leaking a guard with `mem::forget` keeps the domain
/// permanently *entered* and the mutable reference alive beyond its intended
/// scope; both are unsound.
///
/// Consequently this type is **!Send + !Sync** and must remain on the thread
/// where it was constructed.
struct OCamlDomainLock {
    _not_send_sync: PhantomData<*const ()>,
}

impl OCamlDomainLock {
    #[inline(always)]
    fn new() -> Self {
        OCamlThreadRegistrationGuard::ensure();
        unsafe {
            ocaml_sys::caml_leave_blocking_section();
        };
        Self {
            _not_send_sync: PhantomData,
        }
    }
}

impl Drop for OCamlDomainLock {
    fn drop(&mut self) {
        unsafe {
            ocaml_sys::caml_enter_blocking_section();
        };
    }
}

impl Deref for OCamlDomainLock {
    type Target = OCamlRuntime;

    fn deref(&self) -> &OCamlRuntime {
        unsafe { internal::recover_runtime_handle() }
    }
}

impl DerefMut for OCamlDomainLock {
    fn deref_mut(&mut self) -> &mut OCamlRuntime {
        unsafe { internal::recover_runtime_handle_mut() }
    }
}

// Thread registration handling

extern "C" {
    pub fn caml_c_thread_register() -> isize;
    pub fn caml_c_thread_unregister() -> isize;
}

/// RAII guard for per-thread OCaml runtime registration.
///
/// This struct is instantiated once per thread (via the `thread_local!`
/// `OCAML_THREAD`) to ensure that the OCaml runtime is registered
/// before any FFI calls into OCaml. The `registered` field is set to `true`
/// **only** if the initial call to `caml_c_thread_register()` returns `1`
/// (indicating success). When the thread exits, the guard’s `Drop`
/// implementation will call `caml_c_thread_unregister()` exactly once
/// if and only if `registered` is `true`.
struct OCamlThreadRegistrationGuard {
    registered: bool,
}

thread_local! {
    static OCAML_THREAD_REGISTRATION_GUARD: OCamlThreadRegistrationGuard = {
        let ok = unsafe { caml_c_thread_register() } == 1;
        OCamlThreadRegistrationGuard { registered: ok }
    };
}

impl OCamlThreadRegistrationGuard {
    /// **Call this at the start of any function that may touch the OCaml runtime.**
    ///
    /// After the first invocation in the thread it’s just a cheap TLS lookup.
    #[inline(always)]
    pub fn ensure() {
        OCAML_THREAD_REGISTRATION_GUARD.with(|_| {});
    }
}

impl Drop for OCamlThreadRegistrationGuard {
    fn drop(&mut self) {
        if self.registered {
            unsafe {
                caml_c_thread_unregister();
            }
        }
    }
}

// For initializing from an OCaml-driven program

#[no_mangle]
extern "C" fn ocaml_interop_setup(_unit: crate::RawOCaml) -> crate::RawOCaml {
    ocaml_sys::UNIT
}

#[no_mangle]
extern "C" fn ocaml_interop_teardown(_unit: crate::RawOCaml) -> crate::RawOCaml {
    unsafe { boxroot_teardown() };
    ocaml_sys::UNIT
}

#[doc(hidden)]
pub mod internal {
    use super::{OCamlRuntime, TLS_RUNTIME};

    #[inline(always)]
    pub unsafe fn recover_runtime_handle_mut() -> &'static mut OCamlRuntime {
        TLS_RUNTIME.with(|cell| &mut *cell.get())
    }

    #[inline(always)]
    pub unsafe fn recover_runtime_handle() -> &'static OCamlRuntime {
        TLS_RUNTIME.with(|cell| &*cell.get())
    }
}
