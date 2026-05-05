use crate::ast::Node;
use crate::tokens::MATH_BRACKET_PREFIX;
use winnow::Result as PResult;
use winnow::combinator::preceded;
use winnow::prelude::*;

pub(super) fn parse_math<T>(input: &mut &str) -> PResult<Node<T>> {
    preceded(MATH_BRACKET_PREFIX, winnow::token::rest)
        .map(|s: &str| Node::Math(s.trim().to_string()))
        .parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::super::parse_bracket;
    use crate::ast::Node;
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
}
