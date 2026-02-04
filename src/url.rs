use mime_guess;
use url::Url;

#[derive(Debug, PartialEq, Eq)]
pub enum UrlKind {
    Image,
    Other,
}

pub fn infer_url_kind(s: &str) -> Option<UrlKind> {
    if let Ok(url) = Url::parse(s) {
        if let Some(ext) = url
            .path_segments()
            .and_then(|segments| segments.last())
            .and_then(|name| name.split('.').last())
        {
            let mime = mime_guess::from_ext(ext).first_or_octet_stream();
            if mime.type_() == mime::IMAGE {
                return Some(UrlKind::Image);
            } else {
                return Some(UrlKind::Other);
            }
        } else {
            return Some(UrlKind::Other);
        }
    }
    None
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

pub fn is_url(s: &str) -> bool {
    Url::parse(s).is_ok()
}
