use tls_parser::{TlsRecordType, parse_tls_plaintext};

#[derive(Debug)]
pub enum ProtocolType {
    RawHttp,
    Tls,
    Other,
}

const MAX_BUF_LENGTH: usize = 2048;
const H2_PREFACE: &[u8] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";

pub fn detect_protocol(buf: &[u8]) -> ProtocolType {
    let buf = &buf[..buf.len().min(MAX_BUF_LENGTH)];

    // h1
    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut req = httparse::Request::new(&mut headers);
    if req.parse(buf).is_ok() {
        return ProtocolType::RawHttp;
    }
    // h2
    if (buf.len() >= H2_PREFACE.len() && buf.starts_with(H2_PREFACE))
        || (buf.len() < H2_PREFACE.len() && H2_PREFACE.starts_with(buf))
    {
        return ProtocolType::RawHttp;
    }

    // tls
    match parse_tls_plaintext(buf) {
        Ok((_, ctx)) if ctx.hdr.record_type == TlsRecordType::Handshake => ProtocolType::Tls,
        Err(tls_parser::Err::Incomplete(_)) => ProtocolType::Tls,
        _ => ProtocolType::Other,
    }
}
