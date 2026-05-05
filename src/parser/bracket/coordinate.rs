use crate::ast::Node;
use crate::ast::{Latitude, Longitude};
use winnow::Result as PResult;
use winnow::ascii::{dec_uint, float};
use winnow::combinator::{alt, eof, opt, preceded, terminated};
use winnow::prelude::*;

fn parse_latitude(input: &mut &str) -> PResult<Latitude> {
    alt((
        preceded('N', float).map(Latitude::North),
        preceded('S', float).map(Latitude::South),
    ))
    .parse_next(input)
}

fn parse_longitude(input: &mut &str) -> PResult<Longitude> {
    alt((
        preceded('E', float).map(Longitude::East),
        preceded('W', float).map(Longitude::West),
    ))
    .parse_next(input)
}

pub(super) fn parse_coordinate<T>(input: &mut &str) -> PResult<Node<T>> {
    terminated(
        (
            parse_latitude,
            preceded(',', parse_longitude),
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

#[cfg(test)]
mod tests {
    use super::super::parse_bracket;
    use crate::ast::{Latitude, Link, Longitude, Node};
    use winnow::Parser;

    fn parse(input: &str) -> Node<()> {
        let mut s = input;
        parse_bracket(&()).parse_next(&mut s).unwrap()
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
