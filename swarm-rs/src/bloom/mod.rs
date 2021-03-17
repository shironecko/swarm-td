use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{alpha1, alphanumeric1, multispace0};
use nom::combinator::recognize;
use nom::error::ErrorKind;
use nom::multi::many0;
use nom::sequence::{pair, pairc};
use nom::IResult;
use std::fmt::Display;

#[enumeration(rename_all = "lowercase")]
#[derive(Clone, Eq, PartialEq, Debug, Display, enum_utils::FromStr, enum_utils::IterVariants)]
pub enum BloomPrimitive {
    Bool,

    U8,
    U16,
    U32,
    U64,

    I8,
    I16,
    I32,
    I64,

    F32,
    F64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BloomType {
    Primitive(BloomPrimitive),
    Identifier(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BloomField {
    pub name: String,
    pub t: BloomType,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BloomStruct {
    pub name: String,
    pub fields: Vec<BloomField>,
}

named!(p_comma(&str) -> char, char!(','));
named!(p_colon(&str) -> char, char!(':'));
named!(p_open_brace(&str) -> char, char!('{'));
named!(p_close_brace(&str) -> char, char!('}'));
named!(p_struct_tag(&str) -> &str, tag!("struct"));

pub fn p_ident(input: &str) -> IResult<&str, &str> {
    recognize(pair(alpha1, many0(alt((alphanumeric1, tag("_"))))))(input)
}

pub fn p_primitive_type<'a>(input: &'a str) -> IResult<&'a str, BloomPrimitive> {
    let (tail, ident) = p_ident(input)?;
    let res = ident.parse::<BloomPrimitive>();
    if let Ok(primitive) = res {
        Ok((tail, primitive))
    } else {
        nomerr(input, ErrorKind::Tag)
    }
}

pub fn p_type<'a>(input: &'a str) -> IResult<&'a str, BloomType> {
    let res = p_primitive_type(input);
    if let Ok((tail, primitive)) = res {
        Ok((tail, BloomType::Primitive(primitive)))
    } else {
        let (tail, ident) = p_ident(input)?;
        Ok((tail, BloomType::Identifier(ident.to_owned())))
    }
}

pub fn p_field<'a>(input: &'a str) -> IResult<&'a str, BloomField> {
    let (tail, _) = multispace0(input)?;
    let (tail, name) = p_ident(tail)?;

    let (tail, _) = multispace0(tail)?;
    let (tail, _) = p_colon(tail)?;

    let (tail, _) = multispace0(tail)?;
    let (tail, t) = p_type(tail)?;

    Ok((
        tail,
        BloomField {
            name: name.to_owned(),
            t,
        },
    ))
}

pub fn p_struct<'a>(input: &'a str) -> IResult<&'a str, BloomStruct> {
    let (tail, _) = multispace0(input)?;
    let (tail, _) = p_struct_tag(tail)?;

    let (tail, _) = multispace0(tail)?;
    let (tail, name) = p_ident(tail)?;

    let (tail, _) = multispace0(tail)?;
    let (tail, _) = p_open_brace(tail)?;

    let (tail, fields) = p_fields(tail)?;

    let (tail, _) = multispace0(tail)?;
    let (tail, _) = p_close_brace(tail)?;

    Ok((
        tail,
        BloomStruct {
            name: name.to_owned(),
            fields,
        },
    ))
}

named!(p_fields(&str) -> Vec<BloomField>, separated_list0!(|i| pairc(i, multispace0, p_comma), p_field));

fn nomerr<'a, T>(
    input: &'a str,
    kind: ErrorKind,
) -> Result<T, nom::Err<nom::error::Error<&'a str>>> {
    use nom::error::Error as NomError;
    Err(nom::Err::Error(NomError::new(input, kind)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_tokens() {
        assert_eq!(p_comma(","), Ok(("", ',')));
        assert_eq!(p_comma("?"), nomerr("?", ErrorKind::Char));
        assert_eq!(p_colon(":"), Ok(("", ':')));
        assert_eq!(p_colon("?"), nomerr("?", ErrorKind::Char));
        assert_eq!(p_open_brace("{"), Ok(("", '{')));
        assert_eq!(p_open_brace("?"), nomerr("?", ErrorKind::Char));
        assert_eq!(p_close_brace("}"), Ok(("", '}')));
        assert_eq!(p_close_brace("?"), nomerr("?", ErrorKind::Char));

        assert_eq!(p_struct_tag("struct"), Ok(("", "struct")));
        assert_eq!(
            p_struct_tag("not a struct"),
            nomerr("not a struct", ErrorKind::Tag)
        )
    }

    #[test]
    fn identifiers() {
        assert_eq!(p_ident("identifier"), Ok(("", "identifier")));
        assert_eq!(
            p_ident("This_st1ll_an_1DENT"),
            Ok(("", "This_st1ll_an_1DENT"))
        );
        assert_eq!(
            p_ident("1this_is_not"),
            nomerr("1this_is_not", ErrorKind::Alpha)
        );
    }

    #[test]
    fn primitive_types() {
        for primitive in BloomPrimitive::iter() {
            let string = format!("{}", primitive).to_lowercase();
            assert_eq!(p_primitive_type(&string), Ok(("", primitive)));
        }

        assert_eq!(
            p_primitive_type("not_a_primitive"),
            nomerr("not_a_primitive", ErrorKind::Tag)
        );
    }

    #[test]
    fn types() {
        assert_eq!(
            p_type("u32"),
            Ok(("", BloomType::Primitive(BloomPrimitive::U32)))
        );
        assert_eq!(
            p_type("SomeIdent"),
            Ok(("", BloomType::Identifier("SomeIdent".to_owned())))
        );
    }

    #[test]
    fn field() {
        assert_eq!(
            p_field("    some_flag : bool"),
            Ok((
                "",
                BloomField {
                    name: "some_flag".to_owned(),
                    t: BloomType::Primitive(BloomPrimitive::Bool)
                }
            ))
        );
        assert_eq!(p_field("some_flag bool"), nomerr("bool", ErrorKind::Char));
        assert_eq!(p_field("1st"), nomerr("1st", ErrorKind::Alpha));
        assert_eq!(p_field("field: 1st"), nomerr("1st", ErrorKind::Alpha));
    }

    #[test]
    fn fields() {
        assert_eq!(p_fields(""), Ok(("", vec![])));
        assert_eq!(
            // this is a hack to prevent an Incomplete error (the hell?)
            // TODO: replace Nom with something more sane
            p_fields("field: u8}"),
            Ok((
                "}",
                vec![BloomField {
                    name: "field".to_owned(),
                    t: BloomType::Primitive(BloomPrimitive::U8)
                }]
            ))
        );
    }

    #[test]
    fn struct_parsing() {
        assert_eq!(
            p_struct("struct Test {}"),
            Ok((
                "",
                BloomStruct {
                    name: "Test".to_owned(),
                    fields: vec![]
                }
            ))
        );
        assert_eq!(
            p_struct("struct Test { b: bool, u: u32 }"),
            Ok((
                "",
                BloomStruct {
                    name: "Test".to_owned(),
                    fields: vec![
                        BloomField {
                            name: "b".to_owned(),
                            t: BloomType::Primitive(BloomPrimitive::Bool)
                        },
                        BloomField {
                            name: "u".to_owned(),
                            t: BloomType::Primitive(BloomPrimitive::U32)
                        },
                    ]
                }
            ))
        );
    }
}
