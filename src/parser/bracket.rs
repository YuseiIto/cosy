use super::bracket_content::take_bracket_content;
use crate::CosyParserExtension;
use crate::ast::Link;
use crate::ast::Node;
use crate::ast::{Latitude, Longitude};
use crate::tokens::{ICON_SUFFIX, LBRACKET, MATH_BRACKET_PREFIX, RBRACKET};
use crate::url::{UrlKind, infer_url};
use winnow::Result as PResult;
use winnow::ascii::{dec_uint, float};
use winnow::combinator::{alt, delimited, eof, opt, preceded, terminated};
use winnow::error::ContextError;
use winnow::prelude::*;
use winnow::token::take_till;

use super::node::parse_nodes;

pub fn parse_bracket<'s, 'i, E>(
    extension: &'s E,
) -> impl Parser<&'i str, Node<E::Output>, ContextError> + 's
where
    E: CosyParserExtension,
{
    move |input: &mut &'i str| {
        let content: &str =
            delimited(LBRACKET, take_bracket_content, RBRACKET).parse_next(input)?;

        let mut inner_input = content;
        alt((
            parse_math,
            parse_icon,
            parse_project_link,
            parse_coordinate,
            parse_links_and_pages(extension),
        ))
        .parse_next(&mut inner_input)
    }
}

fn parse_coordinate<T>(input: &mut &str) -> PResult<Node<T>> {
    terminated(
        (
            alt((
                preceded('N', float).map(Latitude::North),
                preceded('S', float).map(Latitude::South),
            )),
            preceded(
                ',',
                alt((
                    preceded('E', float).map(Longitude::East),
                    preceded('W', float).map(Longitude::West),
                )),
            ),
            opt(preceded(",Z", dec_uint)),
        ),
        eof,
    )
    .map(|(latitude, longitude, zoom)| Node::Coordinate {
        latitude,
        longitude,
        zoom,
    })
    .parse_next(input)
}

fn parse_math<T>(input: &mut &str) -> PResult<Node<T>> {
    preceded(MATH_BRACKET_PREFIX, winnow::token::rest)
        .map(|s: &str| Node::Math(s.trim().to_string()))
        .parse_next(input)
}

fn parse_icon<T>(input: &mut &str) -> PResult<Node<T>> {
    // [name.icon] or [name.icon*N] (N > 0)
    let content = *input;

    let (icon_part, count) = content
        .rsplit_once('*')
        .filter(|(before, _)| before.ends_with(ICON_SUFFIX))
        .and_then(|(before, after)| {
            let n = after.parse::<usize>().ok()?;
            (n > 0).then_some((before, n))
        })
        .unwrap_or((content, 1));

    let name = icon_part
        .strip_suffix(ICON_SUFFIX)
        .filter(|name| !name.is_empty())
        .ok_or_else(ContextError::new)?;

    *input = "";
    Ok(Node::Icon {
        name: name.to_string(),
        count,
    })
}

fn parse_project_link<T>(input: &mut &str) -> PResult<Node<T>> {
    // [/project], [/project/], or [/project/page]
    preceded(
        '/',
        (take_till(1.., '/'), opt(preceded('/', winnow::token::rest))),
    )
    .map(|(project, page): (&str, Option<&str>)| {
        let project = project.to_string();
        match page {
            Some(p) if !p.is_empty() => Node::Link(Link::ProjectPage {
                project,
                page: p.to_string(),
            }),
            _ => Node::Link(Link::Project(project)),
        }
    })
    .parse_next(input)
}

enum BracketToken<'a> {
    Image(::url::Url),
    Link(::url::Url),
    Plain(&'a str),
}

impl<'a> BracketToken<'a> {
    fn of(s: &'a str) -> Self {
        match infer_url(s) {
            Some((url, UrlKind::Image)) => Self::Image(url),
            Some((url, UrlKind::Other)) => Self::Link(url),
            None => Self::Plain(s),
        }
    }
}

fn parse_links_and_pages<'s, 'i, E>(
    extension: &'s E,
) -> impl FnMut(&mut &'i str) -> PResult<Node<E::Output>> + 's
where
    E: CosyParserExtension,
{
    move |input: &mut &'i str| {
        let content = *input;
        *input = ""; // Always consumes the whole bracket content.

        // Single-token content.
        let Some((first_token, rest)) = content.split_once(' ') else {
            return Ok(match BracketToken::of(content) {
                BracketToken::Image(url) => Node::Image(url),
                BracketToken::Link(url) => Node::Link(Link::Url(url)),
                BracketToken::Plain(s) => Node::Link(Link::Page(s.to_string())),
            });
        };
        // `split_once` succeeded, so `rsplit_once` must too.
        let (start, last_token) = content.rsplit_once(' ').expect("content contains ' '");

        let with_label = |href: ::url::Url, text: &str| -> PResult<Node<E::Output>> {
            let mut label_input = text;
            let label = parse_nodes(&mut label_input, extension)?;
            Ok(Node::Link(Link::WithLabel { href, label }))
        };

        // Only the first and last tokens can be URLs; classify each end and
        // dispatch on the pair.
        match (BracketToken::of(first_token), BracketToken::of(last_token)) {
            // LinkedImage — both ends are URLs and at least one is an image.
            // (Only reachable for 2-token inputs, since 3+ tokens means the
            // "label side" string contains spaces and can't be a URL.)
            (BracketToken::Image(href), BracketToken::Image(src)) => {
                Ok(Node::LinkedImage { src, href })
            }
            (BracketToken::Image(src), BracketToken::Link(href)) => {
                Ok(Node::LinkedImage { src, href })
            }
            (BracketToken::Link(href), BracketToken::Image(src)) => {
                Ok(Node::LinkedImage { src, href })
            }

            // URL anchor on the left → label is everything that follows.
            (BracketToken::Link(href), _) => with_label(href, rest),

            // URL anchor on the right → label is everything that precedes.
            (_, BracketToken::Link(href) | BracketToken::Image(href)) => with_label(href, start),

            // (Image, Plain) and (Plain, Plain) — no usable URL pair.
            _ => Ok(Node::Link(Link::Page(content.to_string()))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::parse_bracket;
    use crate::ast::{Latitude, Link, Longitude, Node};
    use winnow::Parser;

    fn parse(input: &str) -> Node<()> {
        let mut s = input;
        parse_bracket(&()).parse_next(&mut s).unwrap()
    }

    #[test]
    fn test_math_inline() {
        let node = parse("[$ y=a^2 + b^2]");
        assert_eq!(node, Node::Math("y=a^2 + b^2".to_string()));
    }

    #[test]
    fn test_icon_simple() {
        let node = parse("[user.icon]");
        assert_eq!(
            node,
            Node::Icon {
                name: "user".to_string(),
                count: 1
            }
        );
    }

    #[test]
    fn test_icon_repeat() {
        let node = parse("[user.icon*3]");
        assert_eq!(
            node,
            Node::Icon {
                name: "user".to_string(),
                count: 3
            }
        );
    }

    #[test]
    fn test_icon_repeat_one() {
        let node = parse("[user.icon*1]");
        assert_eq!(
            node,
            Node::Icon {
                name: "user".to_string(),
                count: 1
            }
        );
    }

    #[test]
    fn test_icon_invalid_count_zero() {
        let node = parse("[user.icon*0]");
        assert_eq!(node, Node::Link(Link::Page("user.icon*0".to_string())));
    }

    #[test]
    fn test_icon_invalid_count_str() {
        let node = parse("[user.icon*abc]");
        assert_eq!(node, Node::Link(Link::Page("user.icon*abc".to_string())));
    }

    #[test]
    fn test_project_page_basic() {
        let node = parse("[/project/page]");
        assert_eq!(
            node,
            Node::Link(Link::ProjectPage {
                project: "project".to_string(),
                page: "page".to_string(),
            })
        );
    }

    #[test]
    fn test_project_page_with_spaces() {
        let node = parse("[/project/page with spaces]");
        assert_eq!(
            node,
            Node::Link(Link::ProjectPage {
                project: "project".to_string(),
                page: "page with spaces".to_string(),
            })
        );
    }

    #[test]
    fn test_project_only_link() {
        // [/project] → Link::Project
        let node = parse("[/project]");
        assert_eq!(node, Node::Link(Link::Project("project".to_string())));
    }

    #[test]
    fn test_project_empty_page_is_project_link() {
        // [/project/] has empty page — treated as project link
        let node = parse("[/project/]");
        assert_eq!(node, Node::Link(Link::Project("project".to_string())));
    }

    // LinkedImage tests
    const IMAGE_URL: &str = "https://example.com/photo.png";
    const IMAGE_URL_2: &str = "https://example.com/other.jpg";
    const LINK_URL: &str = "https://example.com/page";

    #[test]
    fn test_linked_image_img_then_link() {
        // [image_url link_url] → LinkedImage { src: image_url, href: link_url }
        let node = parse(&format!("[{IMAGE_URL} {LINK_URL}]"));
        assert_eq!(
            node,
            Node::LinkedImage {
                src: ::url::Url::parse(IMAGE_URL).unwrap(),
                href: ::url::Url::parse(LINK_URL).unwrap(),
            }
        );
    }

    #[test]
    fn test_linked_image_link_then_img() {
        // [link_url image_url] → LinkedImage { src: image_url, href: link_url }
        let node = parse(&format!("[{LINK_URL} {IMAGE_URL}]"));
        assert_eq!(
            node,
            Node::LinkedImage {
                src: ::url::Url::parse(IMAGE_URL).unwrap(),
                href: ::url::Url::parse(LINK_URL).unwrap(),
            }
        );
    }

    #[test]
    fn test_linked_image_two_images() {
        // [img1 img2] → LinkedImage { src: img2, href: img1 }
        let node = parse(&format!("[{IMAGE_URL} {IMAGE_URL_2}]"));
        assert_eq!(
            node,
            Node::LinkedImage {
                src: ::url::Url::parse(IMAGE_URL_2).unwrap(),
                href: ::url::Url::parse(IMAGE_URL).unwrap(),
            }
        );
    }

    // Regression tests for non-image URL cases
    #[test]
    fn test_with_label_url_then_single_word() {
        // [url label] → WithLabel { href: url, label: [Text("label")] }
        let node = parse(&format!("[{LINK_URL} label]"));
        assert_eq!(
            node,
            Node::Link(Link::WithLabel {
                href: ::url::Url::parse(LINK_URL).unwrap(),
                label: vec![Node::Text("label".to_string())],
            })
        );
    }

    #[test]
    fn test_with_label_single_word_then_url() {
        // [label url] → WithLabel { href: url, label: [Text("label")] }
        let node = parse(&format!("[label {LINK_URL}]"));
        assert_eq!(
            node,
            Node::Link(Link::WithLabel {
                href: ::url::Url::parse(LINK_URL).unwrap(),
                label: vec![Node::Text("label".to_string())],
            })
        );
    }

    #[test]
    fn test_with_label_url_then_multi_word() {
        // [url some text] → WithLabel { href: url, label: [Text("some text")] }
        let node = parse(&format!("[{LINK_URL} some text]"));
        assert_eq!(
            node,
            Node::Link(Link::WithLabel {
                href: ::url::Url::parse(LINK_URL).unwrap(),
                label: vec![Node::Text("some text".to_string())],
            })
        );
    }

    #[test]
    fn test_with_label_multi_word_then_url() {
        // [some text url] → WithLabel { href: url, label: [Text("some text")] }
        let node = parse(&format!("[some text {LINK_URL}]"));
        assert_eq!(
            node,
            Node::Link(Link::WithLabel {
                href: ::url::Url::parse(LINK_URL).unwrap(),
                label: vec![Node::Text("some text".to_string())],
            })
        );
    }

    #[test]
    fn test_with_label_url_then_multiple_urls() {
        // [url1 url2 url3] → WithLabel { href: url1, label: [Text("url2 url3")] }
        let url2 = "https://example.com/other";
        let url3 = "https://example.com/another";
        let node = parse(&format!("[{LINK_URL} {url2} {url3}]"));
        assert_eq!(
            node,
            Node::Link(Link::WithLabel {
                href: ::url::Url::parse(LINK_URL).unwrap(),
                label: vec![Node::Text(format!("{url2} {url3}"))],
            })
        );
    }

    #[test]
    fn test_with_label_two_non_image_urls() {
        // [url1 url2] → WithLabel { href: url1, label: [Text(url2)] }
        // parse_nodes on a bare URL (no brackets) produces Text, not Link::Url
        let url2 = "https://example.com/other";
        let node = parse(&format!("[{LINK_URL} {url2}]"));
        assert_eq!(
            node,
            Node::Link(Link::WithLabel {
                href: ::url::Url::parse(LINK_URL).unwrap(),
                label: vec![Node::Text(url2.to_string())],
            })
        );
    }

    #[test]
    fn test_page_with_spaces() {
        // [text1 text2] → Page("text1 text2")
        let node = parse("[hello world]");
        assert_eq!(node, Node::Link(Link::Page("hello world".to_string())));
    }

    #[test]
    fn test_icon_name_with_spaces() {
        // [user name.icon] → Icon { name: "user name", count: 1 }
        let node = parse("[user name.icon]");
        assert_eq!(
            node,
            Node::Icon {
                name: "user name".to_string(),
                count: 1,
            }
        );
    }

    #[test]
    fn test_icon_empty_name_is_page_link() {
        // [.icon] has empty name — should fall through to Link::Page
        let node = parse("[.icon]");
        assert_eq!(node, Node::Link(Link::Page(".icon".to_string())));
    }

    #[test]
    fn test_page_with_multiple_spaces() {
        // [text1 text2 text3] → Page("text1 text2 text3")
        let node = parse("[hello beautiful world]");
        assert_eq!(
            node,
            Node::Link(Link::Page("hello beautiful world".to_string()))
        );
    }

    #[test]
    fn test_coordinate_basic() {
        let node = parse("[N35.6578589,E139.7474797]");
        assert_eq!(
            node,
            Node::Coordinate {
                latitude: Latitude::North(35.6578589),
                longitude: Longitude::East(139.7474797),
                zoom: None,
            }
        );
    }

    #[test]
    fn test_coordinate_with_zoom() {
        let node = parse("[N35.6578589,E139.7474797,Z14]");
        assert_eq!(
            node,
            Node::Coordinate {
                latitude: Latitude::North(35.6578589),
                longitude: Longitude::East(139.7474797),
                zoom: Some(14),
            }
        );
    }

    #[test]
    fn test_coordinate_south_west() {
        let node = parse("[S33.8688,W151.2093]");
        assert_eq!(
            node,
            Node::Coordinate {
                latitude: Latitude::South(33.8688),
                longitude: Longitude::West(151.2093),
                zoom: None,
            }
        );
    }

    #[test]
    fn test_coordinate_zoom_zero() {
        let node = parse("[N35.65,E139.74,Z0]");
        assert_eq!(
            node,
            Node::Coordinate {
                latitude: Latitude::North(35.65),
                longitude: Longitude::East(139.74),
                zoom: Some(0),
            }
        );
    }

    #[test]
    fn test_coordinate_empty_lat_falls_through() {
        // [N,E] — lat is empty, f64 parse fails → Link::Page
        let node = parse("[N,E]");
        assert_eq!(node, Node::Link(Link::Page("N,E".to_string())));
    }

    #[test]
    fn test_coordinate_invalid_dir_falls_through() {
        // [North,East] — first char not N/S → Link::Page
        let node = parse("[North,East]");
        assert_eq!(node, Node::Link(Link::Page("North,East".to_string())));
    }
}
