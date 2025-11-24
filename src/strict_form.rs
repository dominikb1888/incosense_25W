// strict_form.rs
// Production-ready StrictForm extractor for Axum using serde_urlencoded
// - rejects invalid percent-encoding
// - rejects invalid (non-UTF-8) decoded fields
// - enforces configurable body size limit and field limits
// - deserializes into T using serde_urlencoded after strict validation

use axum::{
    body::{Body, Bytes, to_bytes},
    extract::FromRequest,
    http::Request,
    response::IntoResponse,
};
use serde::de::DeserializeOwned;
use serde_urlencoded;
use std::collections::HashMap;

/// Configuration constants — adjust to your needs
const MAX_BODY_BYTES: usize = 16 * 1024; // 16 KiB
const MAX_FIELDS: usize = 256; // maximum number of form pairs

/// StrictForm wrapper — use in handlers as `StrictForm<T>`
pub struct StrictForm<T>(pub T);

#[derive(Debug)]
pub enum StrictFormRejection {
    ReadBody,
    PayloadTooLarge,
    InvalidPercentEncoding,
    InvalidUtf8,
    TooManyFields,
    InvalidFormStructure(String),
}

impl IntoResponse for StrictFormRejection {
    fn into_response(self) -> axum::response::Response {
        let code = match &self {
            StrictFormRejection::ReadBody => axum::http::StatusCode::BAD_REQUEST,
            StrictFormRejection::PayloadTooLarge => axum::http::StatusCode::PAYLOAD_TOO_LARGE,
            StrictFormRejection::InvalidPercentEncoding => axum::http::StatusCode::BAD_REQUEST,
            StrictFormRejection::InvalidUtf8 => axum::http::StatusCode::UNPROCESSABLE_ENTITY,
            StrictFormRejection::TooManyFields => axum::http::StatusCode::BAD_REQUEST,
            StrictFormRejection::InvalidFormStructure(_) => axum::http::StatusCode::BAD_REQUEST,
        };

        let body = match self {
            StrictFormRejection::InvalidFormStructure(s) => s,
            other => format!("{:?}", other),
        };

        (code, body).into_response()
    }
}

// Axum 0.8 compatible: FromRequest<AppState, Body>
impl<S, T> FromRequest<S, Body> for StrictForm<T>
where
    T: DeserializeOwned + Send + 'static,
{
    type Rejection = StrictFormRejection;

    fn from_request(
        req: Request<Body>,
        _state: &S,
    ) -> impl std::future::Future<
        Output = Result<Self, <Self as FromRequest<(), axum::body::Body>>::Rejection>,
    > + Send {
        Box::pin(async move {
            let whole: Bytes = to_bytes(req.into_body(), MAX_BODY_BYTES)
                .await
                .map_err(|_| StrictFormRejection::ReadBody)?;
            let whole = whole.to_vec();

            if percent_encoding_is_invalid(&whole) {
                return Err(StrictFormRejection::InvalidPercentEncoding);
            }

            let parsed = parse_raw_form(&whole);
            if parsed.len() > MAX_FIELDS {
                return Err(StrictFormRejection::TooManyFields);
            }

            // convert raw bytes to UTF-8 strings
            let mut form_map: HashMap<String, String> = HashMap::new();
            for (raw_k, raw_v) in parsed.into_iter() {
                // Reject NUL bytes in keys or values
                if raw_k.contains(&0) || raw_v.contains(&0) {
                    return Err(StrictFormRejection::InvalidUtf8);
                }

                let k = String::from_utf8(raw_k).map_err(|_| StrictFormRejection::InvalidUtf8)?;
                let v = String::from_utf8(raw_v).map_err(|_| StrictFormRejection::InvalidUtf8)?;
                form_map.insert(k, v);
            }

            // deserialize into T using serde_urlencoded
            let t: T = serde_urlencoded::from_str(
                &serde_urlencoded::to_string(&form_map)
                    .map_err(|e| StrictFormRejection::InvalidFormStructure(e.to_string()))?,
            )
            .map_err(|e| StrictFormRejection::InvalidFormStructure(e.to_string()))?;

            Ok(StrictForm(t))
        })
    }
}

fn percent_encoding_is_invalid(bytes: &[u8]) -> bool {
    let mut i = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'%' => {
                if i + 2 >= bytes.len() {
                    return true;
                }
                if !is_hex_digit(bytes[i + 1]) || !is_hex_digit(bytes[i + 2]) {
                    return true;
                }
                i += 3;
            }
            _ => i += 1,
        }
    }
    false
}

#[inline]
fn is_hex_digit(b: u8) -> bool {
    (b'0'..=b'9').contains(&b) || (b'a'..=b'f').contains(&b) || (b'A'..=b'F').contains(&b)
}

fn parse_raw_form(data: &[u8]) -> Vec<(Vec<u8>, Vec<u8>)> {
    let mut out = Vec::new();
    for pair in data.split(|&b| b == b'&') {
        if pair.is_empty() {
            continue;
        }
        let mut eq_idx = None;
        for (i, &b) in pair.iter().enumerate() {
            if b == b'=' {
                eq_idx = Some(i);
                break;
            }
        }
        if let Some(idx) = eq_idx {
            let (k, v) = pair.split_at(idx);
            let v = &v[1..];
            let key = percent_decode_bytes(k);
            let val = percent_decode_bytes(v);
            out.push((key, val));
        } else {
            let key = percent_decode_bytes(pair);
            out.push((key, Vec::new()));
        }
    }
    out
}

fn percent_decode_bytes(input: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(input.len());
    let mut i = 0;
    while i < input.len() {
        match input[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' => {
                if i + 2 < input.len() {
                    if let (Some(h), Some(l)) = (from_hex(input[i + 1]), from_hex(input[i + 2])) {
                        out.push(h * 16 + l);
                        i += 3;
                        continue;
                    }
                }
                out.push(b'%');
                i += 1;
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    out
}

#[inline]
fn from_hex(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}
