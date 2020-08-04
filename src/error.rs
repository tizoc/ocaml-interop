use mlvalues::RawOCaml;

#[derive(Debug)]
pub enum CamlError {
    Exception(RawOCaml),
}

#[derive(Debug)]
pub enum Error {
    Caml(CamlError),
}