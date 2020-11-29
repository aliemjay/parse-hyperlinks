//! This module implements parsers for HTML hyperlinks.
//! The code follows [HTML 5.2: 4.5. Text-level
//! semantics](https://www.w3.org/TR/html52/textlevel-semantics.html#the-a-element)
#![allow(dead_code)]

use nom::branch::alt;
use nom::bytes::complete::is_not;
use nom::bytes::complete::tag;
use nom::character::complete::alphanumeric1;

/// Parse an HTML hyperlink.
/// The parser expects to start at the link start (`<`) to succeed.
/// ```
/// use parse_hyperlinks::parser::html::html_link;
/// assert_eq!(
///   html_link(r#"<a href="destination" title="title">name</a>abc"#),
///   Ok(("abc", ("name", "destination", "title")))
/// );
/// ```
/// It returns either `Ok((i, (link_name, link_destination, link_title)))` or some error.
pub fn html_link(i: &str) -> nom::IResult<&str, (&str, &str, &str)> {
    let (i, ((link_destination, link_title), link_name)) = nom::sequence::terminated(
        nom::sequence::pair(tag_a_opening, nom::bytes::complete::take_until("</a>")),
        tag("</a>"),
    )(i)?;
    Ok((i, (link_name, link_destination, link_title)))
}

/// Parses a `<a ...>` opening tag and returns
/// either `Ok((i, (link_destination, link_title)))` or some error.
fn tag_a_opening(i: &str) -> nom::IResult<&str, (&str, &str)> {
    nom::sequence::delimited(
        tag("<a "),
        nom::combinator::map_parser(is_not(">"), parse_attributes),
        tag(">"),
    )(i)
}

/// Parses attributes and returns `Ok((name, value))`.
/// Boolean attributes are ignored, but silently consumed.
fn attribute(i: &str) -> nom::IResult<&str, (&str, &str)> {
    alt((
        nom::sequence::pair(
            nom::combinator::verify(alphanumeric1, |s: &str| {
                s.chars().next().unwrap().is_alphabetic()
            }),
            nom::sequence::delimited(tag("=\""), is_not("\""), tag("\"")),
        ),
        // Consume boolean attributes.
        nom::combinator::value(
            ("", ""),
            nom::combinator::verify(alphanumeric1, |s: &str| {
                s.chars().next().unwrap().is_alphabetic()
            }),
        ),
    ))(i)
}

/// Parses a whitespace separated list of attributes and returns a vector of (name, value).
fn attribute_list<'a>(i: &'a str) -> nom::IResult<&'a str, Vec<(&'a str, &'a str)>> {
    let i = i.trim();
    nom::multi::separated_list1(nom::character::complete::multispace1, attribute)(i)
}

/// Extracts the `href` and `title` attributes and returns
/// `Ok((link_destination, link_title))`. `link_title` can be empty,
/// `link_destination` not.
fn parse_attributes(i: &str) -> nom::IResult<&str, (&str, &str)> {
    let (i, attributes) = attribute_list(i)?;
    let mut href = "";
    let mut title = "";

    for (name, value) in attributes {
        if name == "href" {
            // Make sure `href` is empty, it can appear only
            // once.
            let _ = nom::combinator::eof(href)?;
            href = value;
        };
        if name == "title" {
            // Make sure `title` is empty, it can appear only
            // once.
            let _ = nom::combinator::eof(title)?;
            title = value;
        }
    }

    // Assure that `href` is not empty.
    let _ = nom::character::complete::anychar(href)?;

    Ok((i, (href, title)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_link() {
        let expected = ("abc", ("W3Schools", "https://www.w3schools.com/", "W3S"));
        assert_eq!(
            html_link(r#"<a title="W3S" href="https://www.w3schools.com/">W3Schools</a>abc"#)
                .unwrap(),
            expected
        );
    }

    #[test]
    fn test_tag_a_opening() {
        let expected = ("abc", ("http://getreu.net", "My blog"));
        assert_eq!(
            tag_a_opening(r#"<a href="http://getreu.net" title="My blog">abc"#).unwrap(),
            expected
        );
    }

    #[test]
    fn test_parse_attributes() {
        let expected = ("", ("http://getreu.net", "My blog"));
        assert_eq!(
            parse_attributes(r#"abc href="http://getreu.net" abc title="My blog" abc"#).unwrap(),
            expected
        );

        let expected = nom::Err::Error(nom::error::Error::new(
            "http://getreu.net",
            nom::error::ErrorKind::Eof,
        ));
        assert_eq!(
            parse_attributes(r#" href="http://getreu.net" href="http://blog.getreu.net" "#)
                .unwrap_err(),
            expected
        );

        let expected = nom::Err::Error(nom::error::Error::new("a", nom::error::ErrorKind::Eof));
        assert_eq!(
            parse_attributes(r#" href="http://getreu.net" title="a" title="b" "#).unwrap_err(),
            expected
        );

        let expected = nom::Err::Error(nom::error::Error::new("", nom::error::ErrorKind::Eof));
        assert_eq!(
            parse_attributes(r#" title="title" "#).unwrap_err(),
            expected
        );
    }

    #[test]
    fn test_attribute_list() {
        let expected = (
            "",
            vec![
                ("", ""),
                ("href", "http://getreu.net"),
                ("", ""),
                ("title", "My blog"),
                ("", ""),
            ],
        );
        assert_eq!(
            attribute_list(r#"abc href="http://getreu.net" abc title="My blog" abc"#).unwrap(),
            expected
        );
    }
    #[test]
    fn test_attribute() {
        let expected = (" abc", ("href", "http://getreu.net"));
        assert_eq!(
            attribute(r#"href="http://getreu.net" abc"#).unwrap(),
            expected
        );

        let expected = (" abc", ("", ""));
        assert_eq!(attribute("bool abc").unwrap(), expected);

        let expected = nom::Err::Error(nom::error::Error::new(
            "1name",
            nom::error::ErrorKind::Verify,
        ));
        assert_eq!(attribute("1name").unwrap_err(), expected);

        let expected = nom::Err::Error(nom::error::Error::new(
            r#"1name="http://getreu.net"#,
            nom::error::ErrorKind::Verify,
        ));
        assert_eq!(
            attribute(r#"1name="http://getreu.net"#).unwrap_err(),
            expected
        );
    }
}