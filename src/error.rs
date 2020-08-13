use mlvalues::RawOCaml;

#[derive(Debug)]
pub enum OCamlError {
    Exception(RawOCaml),
}
