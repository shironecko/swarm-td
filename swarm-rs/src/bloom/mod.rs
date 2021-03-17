use nom::IResult;
use nom::combinator::recognize;
use nom::sequence::pair;
use nom::branch::alt;
use nom::character::complete::{alpha1, alphanumeric1};
use nom::multi::many0;
use nom::bytes::complete::tag;

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

pub enum BloomType {
    Primitive(BloomPrimitive),
    Struct(String),
}

pub struct BloomField {
    pub name: String,
    pub t: BloomType,
}

pub struct BloomStruct {
    pub name: String,
    pub fields: Vec<BloomType>,
}

named!(p_comma(&str) -> char, char!(','));
named!(p_colon(&str) -> char, char!(':'));
named!(p_open_brace(&str) -> char, char!('{'));
named!(p_close_brace(&str) -> char, char!('}'));
named!(p_struct_tag(&str) -> &str, tag!("struct"));

pub fn p_ident(input: &str) -> IResult<&str, &str> {
    recognize(
        pair(
            alpha1,
            many0(alt((alphanumeric1, tag("_"))))
        )
    )(input)
}
