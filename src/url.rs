use mime_guess;
use url::Url;

pub fn is_url(s: &str) -> bool {
    Url::parse(s).is_ok()
}

pub fn is_image_url(s: &str) -> bool {
    if let Ok(url) = Url::parse(s) {
        if let Some(ext) = url
            .path_segments()
            .and_then(|segments| segments.last())
            .and_then(|name| name.split('.').last())
        {
            let mime = mime_guess::from_ext(ext).first_or_octet_stream();
            return mime.type_() == mime::IMAGE;
        }
    }
    false
}
