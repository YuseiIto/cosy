use url::Url;

#[derive(Debug, PartialEq, Eq)]
pub enum UrlKind {
    Image,
    Other,
}

pub fn infer_url_kind(s: &str) -> Option<UrlKind> {
    Some(infer_url(s)?.1)
}

/// Parses `s` as a URL and classifies it. Returns the parsed [`Url`] together
/// with the inferred [`UrlKind`] so callers do not have to re-parse.
pub fn infer_url(s: &str) -> Option<(Url, UrlKind)> {
    if !s.contains("://") {
        return None;
    }
    let url = Url::parse(s).ok()?;
    let kind = if let Some(ext) = url
        .path_segments()
        .and_then(|mut segments| segments.next_back())
        .and_then(|name| name.split('.').next_back())
    {
        let mime = mime_guess::from_ext(ext).first_or_octet_stream();
        if mime.type_() == mime::IMAGE {
            UrlKind::Image
        } else {
            UrlKind::Other
        }
    } else {
        UrlKind::Other
    };
    Some((url, kind))
}

#[test]
fn test_infer_url_kind() {
    assert_eq!(
        infer_url_kind("https://example.com/image.png"),
        Some(UrlKind::Image)
    );
    assert_eq!(
        infer_url_kind("https://example.com/document.pdf"),
        Some(UrlKind::Other)
    );
    assert_eq!(infer_url_kind("not a url"), None);
}

#[test]
fn test_infer_url_kind_image_with_query() {
    // Query string should not affect image detection
    assert_eq!(
        infer_url_kind("https://example.com/image.png?w=100"),
        Some(UrlKind::Image)
    );
}
