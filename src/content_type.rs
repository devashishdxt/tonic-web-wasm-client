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
        for ct in content_type.split(';') {
            match ct.trim() {
                GRPC_WEB_TEXT | GRPC_WEB_TEXT_PROTO => return Ok(Encoding::Base64),
                GRPC_WEB | GRPC_WEB_PROTO => return Ok(Encoding::None),
                _ => continue,
            }
        }
        Err(Error::InvalidContentType(content_type.to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoding_from_content_type() {
        let vals = [
            (GRPC_WEB, Encoding::None),
            (GRPC_WEB_PROTO, Encoding::None),
            (GRPC_WEB_TEXT, Encoding::Base64),
            (GRPC_WEB_TEXT_PROTO, Encoding::Base64),
            ("application/grpc-web+proto;charset=utf-8", Encoding::None),
            ("application/grpc-web+proto; charset=utf-8", Encoding::None),
            ("charset=utf-8; application/grpc-web+proto", Encoding::None),
        ];
        for (content_type, expected) in vals.iter() {
            assert_eq!(
                Encoding::from_content_type(content_type).ok(),
                Some(*expected)
            );
        }
    }
}
