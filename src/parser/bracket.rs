use super::bracket_content::take_bracket_content;
use crate::CosyParserExtension;
use crate::ast::Link;
use crate::ast::Node;
use crate::tokens::{ICON_SUFFIX, LBRACKET, MATH_BRACKET_PREFIX, RBRACKET};
use crate::url::{UrlKind, infer_url_kind};
use winnow::combinator::delimited;
use winnow::error::ContextError;
use winnow::prelude::*;

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

        // 1. Math: [$ expr]
        if let Some(expr) = content.strip_prefix(MATH_BRACKET_PREFIX) {
            return Ok(Node::Math(expr.trim().to_string()));
        }

        // 2. Icon: [name.icon] or [name.icon*3]
        if content.ends_with(ICON_SUFFIX) {
            // Simple icon — reject empty names like [.icon]
            let name = content.trim_end_matches(ICON_SUFFIX);
            if !name.is_empty() {
                return Ok(Node::Icon {
                    name: name.to_string(),
                    count: 1,
                });
            }
        }
        if let Some((name_part, count_str)) = content.rsplit_once('*')
            && name_part.ends_with(ICON_SUFFIX)
            && let Ok(count) = count_str.parse::<usize>()
            && count > 0
        {
            let name = name_part.trim_end_matches(ICON_SUFFIX);
            if !name.is_empty() {
                return Ok(Node::Icon {
                    name: name.to_string(),
                    count,
                });
            }
        }

        // 3. Cross-project link: [/project], [/project/], or [/project/page]
        if let Some(rest) = content.strip_prefix('/') {
            if let Some((project, page)) = rest.split_once('/') {
                if !project.is_empty() {
                    if page.is_empty() {
                        return Ok(Node::Link(Link::Project(project.to_string())));
                    } else {
                        return Ok(Node::Link(Link::ProjectPage {
                            project: project.to_string(),
                            page: page.to_string(),
                        }));
                    }
                }
            } else if !rest.is_empty() {
                return Ok(Node::Link(Link::Project(rest.to_string())));
            }
        }

        // 4. Links (recurse on label)
        //
        // URLs never contain spaces, so the URL is always the first or last token.
        // Split at the first space to isolate a leading URL; at the last space for a trailing URL.
        if let Some((first_token, rest)) = content.split_once(' ')
            && let Some((start, last_token)) = content.rsplit_once(' ')
        {
            let first_token = first_token.trim();
            let last_token = last_token.trim();

            // Determine which end holds the URL (if any).
            let (left, right) = if infer_url_kind(first_token).is_some() {
                // [url ...label...]
                (first_token, rest.trim())
            } else if infer_url_kind(last_token).is_some() {
                // [...label... url]
                (start.trim(), last_token)
            } else {
                // [Page Name With Spaces]
                return Ok(Node::Link(Link::Page(content.to_string())));
            };

            let left_kind = infer_url_kind(left);
            let right_kind = infer_url_kind(right);

            return match (left_kind, right_kind) {
                (Some(UrlKind::Image), Some(UrlKind::Image)) => {
                    // [img1 img2] → display img2, link to img1
                    Ok(Node::LinkedImage {
                        src: right.to_string(),
                        href: left.to_string(),
                    })
                }
                (Some(UrlKind::Image), Some(UrlKind::Other)) => {
                    // [img link] → display img, link to link
                    Ok(Node::LinkedImage {
                        src: left.to_string(),
                        href: right.to_string(),
                    })
                }
                (Some(UrlKind::Other), Some(UrlKind::Image)) => {
                    // [link img] → display img, link to link
                    Ok(Node::LinkedImage {
                        src: right.to_string(),
                        href: left.to_string(),
                    })
                }
                (Some(UrlKind::Other), _) => {
                    // [url label...] → multi-word label linking to url
                    let mut label_input = right;
                    let nodes = parse_nodes(&mut label_input, extension)?;
                    let href = ::url::Url::parse(left).map_err(|_| ContextError::new())?;
                    Ok(Node::Link(Link::WithLabel { href, label: nodes }))
                }
                (_, Some(UrlKind::Other) | Some(UrlKind::Image)) => {
                    // [...label url] → multi-word label linking to url
                    let mut label_input = left;
                    let nodes = parse_nodes(&mut label_input, extension)?;
                    let href = ::url::Url::parse(right).map_err(|_| ContextError::new())?;
                    Ok(Node::Link(Link::WithLabel { href, label: nodes }))
                }
                _ => {
                    // [Page Name] - space inside page name
                    Ok(Node::Link(Link::Page(content.to_string())))
                }
            };
        }

        // 4. Simple content (Coordinate, Image, URL, Page)
        if let Some(node) = try_parse_coordinate(content) {
            return Ok(node);
        }

        match infer_url_kind(content) {
            Some(UrlKind::Image) => Ok(Node::Image(content.to_string())),
            Some(UrlKind::Other) => Ok(Node::Link(Link::Url(content.to_string()))),
            None => Ok(Node::Link(Link::Page(content.to_string()))),
        }
    }
}

/// Parses a direction prefix + numeric value from a coordinate component.
/// e.g. `"N35.65"` with `['N', 'S']` → `Some(('N', 35.65))`.
fn parse_coord(s: &str, dirs: [char; 2]) -> Option<(char, f64)> {
    let dir = s.chars().next().filter(|c| dirs.contains(c))?;
    let val = &s[1..]; // dir is always ASCII
    let parsed = val.parse::<f64>().ok()?;
    Some((dir, parsed))
}

/// Tries to parse `content` as coordinate syntax `[NS]{lat},{EW}{lon}[,Z{zoom}]`.
/// Returns `Some(Node::Coordinate {...})` on success, `None` otherwise.
fn try_parse_coordinate<T>(content: &str) -> Option<Node<T>> {
    if !matches!(content.as_bytes().first(), Some(b'N' | b'S')) {
        return None;
    }

    let parts: Vec<&str> = content.split(',').collect();
    if parts.len() < 2 || parts.len() > 3 {
        return None;
    }

    let (lat_dir, lat) = parse_coord(parts[0], ['N', 'S'])?;
    let (lon_dir, lon) = parse_coord(parts[1], ['E', 'W'])?;

    let zoom = if let Some(z_str) = parts.get(2) {
        let z = z_str.strip_prefix('Z')?;
        Some(z.parse::<u32>().ok()?)
    } else {
        None
    };

    Some(Node::Coordinate {
        lat,
        lat_dir,
        lon,
        lon_dir,
        zoom,
    })
}

#[cfg(test)]
mod tests {
    use super::parse_bracket;
    use crate::ast::{Link, Node};
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
                src: IMAGE_URL.to_string(),
                href: LINK_URL.to_string(),
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
                src: IMAGE_URL.to_string(),
                href: LINK_URL.to_string(),
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
                src: IMAGE_URL_2.to_string(),
                href: IMAGE_URL.to_string(),
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
                lat: 35.6578589,
                lat_dir: 'N',
                lon: 139.7474797,
                lon_dir: 'E',
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
                lat: 35.6578589,
                lat_dir: 'N',
                lon: 139.7474797,
                lon_dir: 'E',
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
                lat: 33.8688,
                lat_dir: 'S',
                lon: 151.2093,
                lon_dir: 'W',
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
                lat: 35.65,
                lat_dir: 'N',
                lon: 139.74,
                lon_dir: 'E',
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
