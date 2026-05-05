use url::Url;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum UrlKind {
    Image,
    Other,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn kind_of(s: &str) -> Option<UrlKind> {
        infer_url(s).map(|(_, k)| k)
    }

    #[test]
    fn test_infer_url_kind() {
        assert_eq!(kind_of("https://example.com/image.png"), Some(UrlKind::Image));
        assert_eq!(
            kind_of("https://example.com/document.pdf"),
            Some(UrlKind::Other)
        );
        assert_eq!(kind_of("not a url"), None);
    }

    #[test]
    fn test_infer_url_kind_image_with_query() {
        // Query string should not affect image detection
        assert_eq!(
            kind_of("https://example.com/image.png?w=100"),
            Some(UrlKind::Image)
        );
    }

    #[test]
    fn test_infer_url_returns_parsed_url() {
        let (url, kind) = infer_url("https://example.com/photo.png").unwrap();
        assert_eq!(url.as_str(), "https://example.com/photo.png");
        assert_eq!(kind, UrlKind::Image);
    }
}
