use crate::{expr::Expr, Program};

mod nom {
    pub use nom::{
        branch::*, character::complete::*, combinator::*, error::*, multi::*, sequence::*, *,
    };
    pub use nom_locate::{position, LocatedSpan};
}

type Span<'a> = nom::LocatedSpan<&'a str>;

fn remove_comments(input: &str) -> String {
    let mut s = String::new();
    for line in input.lines() {
        s.push_str(line.splitn(2, '#').next().unwrap());
        s.push('\n');
    }
    s
}

fn term<'a, E: nom::ParseError<Span<'a>>>(
    c: char,
) -> impl Fn(Span<'a>) -> nom::IResult<Span<'a>, char, E> {
    nom::preceded(nom::multispace0, nom::char(c))
}

#[derive(Clone, Copy, Debug)]
pub struct ParseError {
    pub line: usize,
    pub column: usize,
}

pub fn parse(input: &str) -> Result<Program, ParseError> {
    let input = remove_comments(input);
    let result = nom::all_consuming(program)(Span::new(&input));
    result
        .map(|(_, expr)| Program(expr))
        .map_err(|err: nom::Err<(Span, nom::ErrorKind)>| {
            let (span, _) = match err {
                nom::Err::Error(err) => err,
                nom::Err::Failure(err) => err,
                _ => unreachable!(),
            };
            ParseError {
                line: span.location_line() as usize,
                column: span.get_column(),
            }
        })
}

fn program<'a, E: nom::ParseError<Span<'a>>>(input: Span<'a>) -> nom::IResult<Span<'a>, Expr, E> {
    nom::terminated(cc_expr, nom::multispace0)(input)
}

fn cc_expr<'a, E: nom::ParseError<Span<'a>>>(input: Span<'a>) -> nom::IResult<Span<'a>, Expr, E> {
    nom::fold_many0(expr, Expr::I, Expr::apply)(input)
}

fn expr<'a, E: nom::ParseError<Span<'a>>>(input: Span<'a>) -> nom::IResult<Span<'a>, Expr, E> {
    nom::alt((nom::value(Expr::I, term('i')), expr_))(input)
}

fn iota_expr<'a, E: nom::ParseError<Span<'a>>>(input: Span<'a>) -> nom::IResult<Span<'a>, Expr, E> {
    nom::alt((nom::value(Expr::Iota, term('i')), expr_))(input)
}

fn expr_<'a, E: nom::ParseError<Span<'a>>>(input: Span<'a>) -> nom::IResult<Span<'a>, Expr, E> {
    nom::alt((
        nom::value(Expr::I, term('I')),
        nom::value(Expr::K, term('K')),
        nom::value(Expr::K, term('k')),
        nom::value(Expr::S, term('S')),
        nom::value(Expr::S, term('s')),
        jot_expr,
        nom::map(
            nom::preceded(term('`'), nom::cut(nom::pair(expr, expr))),
            |(lhs, rhs)| lhs.apply(rhs),
        ),
        nom::map(
            nom::preceded(term('*'), nom::cut(nom::pair(iota_expr, iota_expr))),
            |(lhs, rhs)| lhs.apply(rhs),
        ),
        nom::preceded(term('('), nom::cut(nom::terminated(cc_expr, term(')')))),
    ))(input)
}

fn jot_expr<'a, E: nom::ParseError<Span<'a>>>(input: Span<'a>) -> nom::IResult<Span<'a>, Expr, E> {
    nom::fold_many1(
        nom::alt((nom::value(false, term('0')), nom::value(true, term('1')))),
        Expr::I,
        #[allow(clippy::match_bool)]
        |expr, c| match c {
            false => expr.apply(Expr::S).apply(Expr::K),
            true => Expr::S.apply(Expr::K.apply(expr)),
        },
    )(input)
}
