use crate::mlvalues::tag;
use crate::mlvalues::{is_block, string_val, tag_val, RawOCaml};
use crate::value::caml_string_length;
use std::slice;

/// An OCaml exception value.
#[derive(Debug)]
pub struct OCamlException {
    raw: RawOCaml,
}

#[derive(Debug)]
pub enum OCamlError {
    Exception(OCamlException),
}

impl OCamlException {
    pub fn of(raw: RawOCaml) -> Self {
        OCamlException { raw }
    }

    pub fn message(&self) -> Option<String> {
        if is_block(self.raw) {
            unsafe {
                let message = *(self.raw as *const RawOCaml).add(1);

                if tag_val(message) == tag::STRING {
                    let error_message =
                        slice::from_raw_parts(string_val(message), caml_string_length(message))
                            .to_owned();
                    let error_message = String::from_utf8_unchecked(error_message);
                    Some(error_message)
                } else {
                    None
                }
            }
        } else {
            None
        }
    }
}
