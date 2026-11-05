use axum::{
    body::Body,
    extract::{Request, State},
    http::{header, HeaderMap, HeaderValue, Method, StatusCode},
    response::{IntoResponse, Response},
};
use std::path::Path;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt, SeekFrom};
use tokio_util::io::ReaderStream;

use super::vfs::Vfs;

/// Determines precise MIME type for Homestuck assets, Flash, and WASM
pub fn get_mime_type(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_ascii_lowercase())
        .as_deref()
    {
        Some("wasm") => "application/wasm",
        Some("swf") => "application/x-shockwave-flash",
        Some("mp3") => "audio/mpeg",
        Some("gif") => "image/gif",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("mp4") => "video/mp4",
        Some("webm") => "video/webm",
        Some("ogg") | Some("oga") => "audio/ogg",
        Some("wav") => "audio/wav",
        Some("m4a") => "audio/mp4",
        Some("html") | Some("htm") => "text/html; charset=utf-8",
        Some("js") => "application/javascript",
        Some("json") => "application/json",
        Some("css") => "text/css",
        Some("xml") => "application/xml",
        Some("txt") => "text/plain; charset=utf-8",
        Some("pdf") => "application/pdf",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("ttf") => "font/ttf",
        Some("otf") => "font/otf",
        Some("ico") => "image/x-icon",
        _ => "application/octet-stream",
    }
}

pub fn add_cors_headers(headers: &mut HeaderMap) {
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        HeaderValue::from_static("*"),
    );
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_METHODS,
        HeaderValue::from_static("GET, POST, HEAD, OPTIONS"),
    );
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_HEADERS,
        HeaderValue::from_static("Content-Type, Range"),
    );
}

enum RangeSpec {
    Exact(u64, u64),
    From(u64),
    Suffix(u64),
}

fn parse_range_header(val: &str) -> Result<RangeSpec, ()> {
    if !val.starts_with("bytes=") {
        return Err(());
    }
    let range_val = val["bytes=".len()..].trim();
    if range_val.is_empty() || !range_val.contains('-') || range_val.matches('-').count() > 1 {
        return Err(());
    }

    let mut parts = range_val.split('-');
    let start_str = parts.next().unwrap_or("").trim();
    let end_str = parts.next().unwrap_or("").trim();

    if !start_str.is_empty() && !end_str.is_empty() {
        let start = start_str.parse::<u64>().map_err(|_| ())?;
        let end = end_str.parse::<u64>().map_err(|_| ())?;
        Ok(RangeSpec::Exact(start, end))
    } else if !start_str.is_empty() {
        let start = start_str.parse::<u64>().map_err(|_| ())?;
        Ok(RangeSpec::From(start))
    } else if !end_str.is_empty() {
        let suffix = end_str.parse::<u64>().map_err(|_| ())?;
        Ok(RangeSpec::Suffix(suffix))
    } else {
        Err(())
    }
}

pub async fn handle_asset_request(vfs: &Vfs, req: Request<Body>) -> Response {
    let method = req.method().clone();
    if method == Method::OPTIONS {
        let mut res = StatusCode::NO_CONTENT.into_response();
        add_cors_headers(res.headers_mut());
        return res;
    }
    if method != Method::GET && method != Method::HEAD {
        let mut res = StatusCode::METHOD_NOT_ALLOWED.into_response();
        add_cors_headers(res.headers_mut());
        return res;
    }

    let uri_path = req.uri().path();
    let resolved_path = match vfs.resolve(uri_path).await {
        Some(p) => p,
        None => {
            let mut res = (StatusCode::NOT_FOUND, "Asset Not Found").into_response();
            add_cors_headers(res.headers_mut());
            return res;
        }
    };

    let metadata = match tokio::fs::metadata(&resolved_path).await {
        Ok(m) if m.is_file() => m,
        _ => {
            let mut res = (StatusCode::NOT_FOUND, "Asset Not Found").into_response();
            add_cors_headers(res.headers_mut());
            return res;
        }
    };

    let file_size = metadata.len();
    let mime_type = get_mime_type(&resolved_path);

    // Check Range header
    if let Some(range_header) = req
        .headers()
        .get(header::RANGE)
        .and_then(|h| h.to_str().ok())
    {
        let range_spec = match parse_range_header(range_header) {
            Ok(spec) => spec,
            Err(_) => {
                let mut res =
                    (StatusCode::RANGE_NOT_SATISFIABLE, "Range Not Satisfiable").into_response();
                res.headers_mut().insert(
                    header::CONTENT_RANGE,
                    HeaderValue::from_str(&format!("bytes */{}", file_size)).unwrap(),
                );
                add_cors_headers(res.headers_mut());
                return res;
            }
        };

        let (start, end) = match range_spec {
            RangeSpec::Exact(s, e) => (s, e),
            RangeSpec::From(s) => (s, file_size.saturating_sub(1)),
            RangeSpec::Suffix(sfx) => {
                let start = file_size.saturating_sub(sfx);
                (start, file_size.saturating_sub(1))
            }
        };

        // Inverted range, beyond end, or invalid bounds
        if start > end || start >= file_size || end >= file_size {
            let mut res =
                (StatusCode::RANGE_NOT_SATISFIABLE, "Range Not Satisfiable").into_response();
            res.headers_mut().insert(
                header::CONTENT_RANGE,
                HeaderValue::from_str(&format!("bytes */{}", file_size)).unwrap(),
            );
            add_cors_headers(res.headers_mut());
            return res;
        }

        let length = end - start + 1;
        let mut res = if method == Method::HEAD {
            StatusCode::PARTIAL_CONTENT.into_response()
        } else {
            match File::open(&resolved_path).await {
                Ok(mut file) => {
                    if file.seek(SeekFrom::Start(start)).await.is_err() {
                        let mut err_res = StatusCode::INTERNAL_SERVER_ERROR.into_response();
                        add_cors_headers(err_res.headers_mut());
                        return err_res;
                    }
                    let limited = file.take(length);
                    let stream = ReaderStream::new(limited);
                    (StatusCode::PARTIAL_CONTENT, Body::from_stream(stream)).into_response()
                }
                Err(_) => {
                    let mut err_res = StatusCode::INTERNAL_SERVER_ERROR.into_response();
                    add_cors_headers(err_res.headers_mut());
                    return err_res;
                }
            }
        };

        let headers = res.headers_mut();
        headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(mime_type));
        headers.insert(
            header::CONTENT_LENGTH,
            HeaderValue::from_str(&length.to_string()).unwrap(),
        );
        headers.insert(
            header::CONTENT_RANGE,
            HeaderValue::from_str(&format!("bytes {}-{}/{}", start, end, file_size)).unwrap(),
        );
        headers.insert(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"));
        add_cors_headers(headers);

        return res;
    }

    // Full content response (200 OK)
    let mut res = if method == Method::HEAD {
        StatusCode::OK.into_response()
    } else {
        match File::open(&resolved_path).await {
            Ok(file) => {
                let stream = ReaderStream::new(file);
                (StatusCode::OK, Body::from_stream(stream)).into_response()
            }
            Err(_) => {
                let mut err_res = StatusCode::INTERNAL_SERVER_ERROR.into_response();
                add_cors_headers(err_res.headers_mut());
                return err_res;
            }
        }
    };

    let headers = res.headers_mut();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(mime_type));
    headers.insert(
        header::CONTENT_LENGTH,
        HeaderValue::from_str(&file_size.to_string()).unwrap(),
    );
    headers.insert(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"));
    add_cors_headers(headers);

    res
}

pub async fn asset_handler(State(vfs): State<Vfs>, req: Request<Body>) -> Response {
    handle_asset_request(&vfs, req).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mime_types() {
        assert_eq!(get_mime_type(Path::new("file.wasm")), "application/wasm");
        assert_eq!(
            get_mime_type(Path::new("movie.swf")),
            "application/x-shockwave-flash"
        );
        assert_eq!(get_mime_type(Path::new("song.mp3")), "audio/mpeg");
        assert_eq!(get_mime_type(Path::new("pic.gif")), "image/gif");
        assert_eq!(get_mime_type(Path::new("pic.PNG")), "image/png");
        assert_eq!(get_mime_type(Path::new("clip.mp4")), "video/mp4");
        assert_eq!(
            get_mime_type(Path::new("unknown.xyz")),
            "application/octet-stream"
        );
    }

    #[test]
    fn test_parse_range_header() {
        match parse_range_header("bytes=0-100").unwrap() {
            RangeSpec::Exact(s, e) => {
                assert_eq!(s, 0);
                assert_eq!(e, 100);
            }
            _ => panic!("Expected Exact"),
        }

        match parse_range_header("bytes=50-").unwrap() {
            RangeSpec::From(s) => assert_eq!(s, 50),
            _ => panic!("Expected From"),
        }

        match parse_range_header("bytes=-500").unwrap() {
            RangeSpec::Suffix(sfx) => assert_eq!(sfx, 500),
            _ => panic!("Expected Suffix"),
        }

        // Malformed ranges
        assert!(parse_range_header("items=0-10").is_err());
        assert!(parse_range_header("bytes=abc-def").is_err());
        assert!(parse_range_header("bytes=1-2-3").is_err());
        assert!(parse_range_header("bytes=-").is_err());
    }
}
