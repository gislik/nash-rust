use thiserror::Error;

pub type Result<T> = std::result::Result<T, ProtocolError>;

#[derive(Debug, Error, Clone)]
pub struct ProtocolError(pub &'static str);

impl ProtocolError {
    // FIXME: this is a terrible hack. Added temporarily because so much code was already relying
    // upon &'static str creation of protocol errors, but migrate everything to String and allow
    // construction of error messages dynamically
    pub fn coerce_static_from_str(error_str: &str) -> Self {
        let coerce_static = Box::leak(error_str.to_string().into_boxed_str());
        ProtocolError(coerce_static)
    }
}

impl std::fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}