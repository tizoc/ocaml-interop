// Copyright (c) Viable Systems and TezEdge Contributors
// SPDX-License-Identifier: MIT

use ocaml_boxroot_sys::boxroot_teardown;
use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
    rc::Rc,
};

use crate::{memory::OCamlRef, value::OCaml};

/// OCaml runtime handle.
///
/// Should be initialized once at the beginning of the program
/// and the obtained handle passed around.
///
/// Once the handle is dropped, the OCaml runtime will be shutdown.
pub struct OCamlRuntime {
    _not_send_sync: PhantomData<Rc<()>>,
}

impl OCamlRuntime {
    /// Initializes the OCaml runtime and returns an OCaml runtime handle.
    ///
    /// Should not be called more than once.
    ///
    /// Once the handle is dropped, the OCaml runtime will be shutdown.
    pub fn init() -> Self {
        if !Self::init_persistent() {
            panic!("OCaml runtime already initialized");
        }
        Self {
            _not_send_sync: PhantomData,
        }
    }

    /// Initializes the OCaml runtime.
    ///
    /// After the first invocation, this method does nothing.
    ///
    /// Returns `true` if the OCaml runtime was initialized, `false` otherwise.
    pub fn init_persistent() -> bool {
        let mut initialized = false;
        #[cfg(not(feature = "no-caml-startup"))]
        {
            static INIT: std::sync::Once = std::sync::Once::new();

            INIT.call_once(|| {
                let arg0 = "ocaml\0".as_ptr() as *const ocaml_sys::Char;
                let c_args = [arg0, core::ptr::null()];
                unsafe {
                    ocaml_sys::caml_startup(c_args.as_ptr());
                    ocaml_boxroot_sys::boxroot_setup();
                };
                initialized = true;
            });

            initialized
        }
        #[cfg(feature = "no-caml-startup")]
        panic!("Rust code that is called from an OCaml program should not try to initialize the runtime.");
    }

    #[doc(hidden)]
    #[inline(always)]
    pub unsafe fn recover_handle_ptr_mut() -> *mut Self {
        static mut RUNTIME: OCamlRuntime = OCamlRuntime {
            _not_send_sync: PhantomData,
        };
        std::ptr::addr_of_mut!(RUNTIME)
    }

    #[doc(hidden)]
    #[inline(always)]
    pub unsafe fn recover_handle_mut() -> &'static mut Self {
        &mut *Self::recover_handle_ptr_mut()
    }

    #[doc(hidden)]
    #[inline(always)]
    unsafe fn recover_handle() -> &'static Self {
        Self::recover_handle_mut()
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
        f(&mut *lock)
    }
}

impl Drop for OCamlRuntime {
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
pub struct OCamlDomainLock {
    _not_send_sync: PhantomData<Rc<()>>,
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

    #[inline(always)]
    fn recover_handle<'a>(&self) -> &'a OCamlRuntime {
        unsafe { OCamlRuntime::recover_handle() }
    }

    #[inline(always)]
    fn recover_handle_mut<'a>(&mut self) -> &'a mut OCamlRuntime {
        unsafe { OCamlRuntime::recover_handle_mut() }
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
        self.recover_handle()
    }
}

impl DerefMut for OCamlDomainLock {
    fn deref_mut(&mut self) -> &mut OCamlRuntime {
        self.recover_handle_mut()
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
        OCAML_THREAD_REGISTRATION_GUARD.with(|_| {}); // create or access the guard
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
