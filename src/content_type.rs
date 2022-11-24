use crate::Error;

const GRPC_WEB: &str = "application/grpc-web";
const GRPC_WEB_PROTO: &str = "application/grpc-web+proto";
const GRPC_WEB_TEXT: &str = "application/grpc-web-text";
const GRPC_WEB_TEXT_PROTO: &str = "application/grpc-web-text+proto";

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Encoding {
    Base64,
    None,
}

impl Encoding {
    pub fn from_content_type(content_type: &str) -> Result<Self, Error> {
        match content_type {
            GRPC_WEB_TEXT | GRPC_WEB_TEXT_PROTO => Ok(Encoding::Base64),
            GRPC_WEB | GRPC_WEB_PROTO => Ok(Encoding::None),
            _ => Err(Error::InvalidContentType(content_type.to_owned())),
        }
    }
}
