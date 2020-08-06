/// ```compile_fail
/// # use znfe::*;
/// # ocaml! { pub fn ocaml_function(arg1: String) -> String; }
/// let s = ocaml_frame!(gc, {
/// #   let arg1 = ocaml_alloc!(("test".to_owned()).to_ocaml(gc));
///     let result = ocaml_function(gc, arg1).unwrap();
/// #    znfe::unit()
/// });
/// ```
pub struct FailsWithoutOCamlCallMacro;

/// ```compile_fail
/// # use znfe::*;
/// # ocaml! { pub fn ocaml_function(arg1: String) -> String; }
/// let s = ocaml_frame!(gc, {
///     let arg1 = ("test".to_owned()).to_ocaml(gc);
/// #    znfe::unit()
/// });
/// ```
pub struct FailsWithoutOCamlAllocMacro;

/// ```compile_fail
/// # use znfe::*;
/// # ocaml! { pub fn ocaml_function(arg1: String) -> String; }
/// ocaml_frame!(gc, {
///     let arg1 = ocaml_alloc!(("test".to_owned()).to_ocaml(gc));
///     let result = ocaml_call!(ocaml_function(gc, arg1)).unwrap();
///     let another_result = ocaml_call!(ocaml_function(gc, arg1)).unwrap();
/// #   znfe::unit()
/// });
/// ```
pub struct LivenessFailureCheck;