//! The game is a browser wasm module, so the cluster cannot run it directly.
//! This wasi:http component serves the static web build instead, so it runs
//! under the runwasi wasmtime shim like any other wasm app.

include!(concat!(env!("OUT_DIR"), "/files.rs"));

const CSP: &str = "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; \
style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; media-src 'self' data: blob:; \
frame-ancestors 'none'; base-uri 'none'; form-action 'none'";

fn content_type(path: &str) -> &'static str {
    match path.rsplit('.').next() {
        Some("html") => "text/html; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("wasm") => "application/wasm",
        Some("png") => "image/png",
        Some("ttf") => "font/ttf",
        Some("ron" | "txt") => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

/// (status, content type, body) for a GET of `path`.
fn route(path: &str) -> (u16, &'static str, &'static [u8]) {
    let path = if path == "/" { "/index.html" } else { path };
    if path == "/healthz" {
        return (200, "text/plain; charset=utf-8", b"ok");
    }
    match FILES.binary_search_by(|(p, _)| p.cmp(&path)) {
        Ok(i) => (200, content_type(path), FILES[i].1),
        Err(_) => (404, "text/plain; charset=utf-8", b"not found"),
    }
}

#[cfg(target_arch = "wasm32")]
#[wstd::http_server]
async fn main(
    request: wstd::http::Request<wstd::http::body::Body>,
) -> Result<wstd::http::Response<wstd::http::body::Body>, wstd::http::Error> {
    use wstd::http::{Method, Response};

    let head = request.method() == Method::HEAD;
    let reply = Response::builder()
        .header("content-security-policy", CSP)
        .header("x-content-type-options", "nosniff")
        .header("referrer-policy", "no-referrer");
    if !head && request.method() != Method::GET {
        return Ok(reply
            .status(405)
            .header("allow", "GET, HEAD")
            .body(().into())?);
    }
    let (status, kind, body) = route(request.uri().path());
    // Assets keep their names between builds, so revalidate every time and
    // let the ETag turn repeat visits into 304s.
    let reply = reply
        .header("cache-control", "no-cache")
        .header("etag", ETAG);
    let fresh = request
        .headers()
        .get("if-none-match")
        .is_some_and(|v| v.as_bytes() == ETAG.as_bytes());
    if status == 200 && fresh {
        return Ok(reply.status(304).body(().into())?);
    }
    let body = if head { &[][..] } else { body };
    Ok(reply
        .status(status)
        .header("content-type", kind)
        .body(body.into())?)
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let _ = (CSP, route);
    eprintln!("antree-server is a wasi:http component: build it with ./build.sh and run it with `wasmtime serve -Scli antree-server.wasm`");
    std::process::exit(1);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serves_the_page_assets_and_health() {
        assert_eq!(route("/").1, "text/html; charset=utf-8");
        assert_eq!(route("/antree.wasm").1, "application/wasm");
        assert_eq!(route("/assets/data/strings.ron").0, 200);
        assert_eq!(
            route("/healthz"),
            (200, "text/plain; charset=utf-8", &b"ok"[..])
        );
        assert_eq!(route("/nope").0, 404);
        assert_eq!(route("/../Cargo.toml").0, 404);
    }

    #[test]
    fn files_are_sorted_for_binary_search() {
        assert!(FILES.windows(2).all(|w| w[0].0 < w[1].0));
    }
}
