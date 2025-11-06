use crate::binary::{instruction::Instruction, section::*, types::*};
use nom::{
    IResult, Parser,
    bytes::complete::{tag, take},
    multi::many0,
    number::complete::{le_u8, le_u32},
};
use nom_leb128::leb128_u32;
use num_traits::FromPrimitive as _;

#[derive(Debug, PartialEq, Eq)]
pub struct Module {
    pub magic: String,
    pub version: u32,
    pub type_section: Option<Vec<FuncType>>,
    pub function_section: Option<Vec<u32>>,
    pub code_section: Option<Vec<Function>>,
}

impl Default for Module {
    fn default() -> Self {
        Self {
            magic: "\0asm".to_string(),
            version: 1,
            type_section: None,
            function_section: None,
            code_section: None,
        }
    }
}

impl Module {
    pub fn new(input: &[u8]) -> anyhow::Result<Self> {
        let (_, module) =
            Self::decode(input).map_err(|e| anyhow::anyhow!("Failed to parse wasm: {}", e))?;
        Ok(module)
    }

    fn decode(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, _) = tag(b"\0asm".as_slice())(input)?;
        let (input, version) = le_u32(input)?;

        let mut module = Self {
            magic: "\0asm".into(),
            version,
            ..Default::default()
        };

        let mut remaining = input;

        while !remaining.is_empty() {
            match decode_section_header(remaining) {
                Ok((input, (code, size))) => {
                    let (rest, section_contents) = take(size)(input)?;

                    match code {
                        SectionCode::Type => {
                            let (_, types) = decode_type_section(section_contents)?;
                            module.type_section = Some(types);
                        }
                        SectionCode::Function => {
                            let (_, func_idx_list) = decode_function_section(section_contents)?;
                            module.function_section = Some(func_idx_list);
                        }
                        SectionCode::Code => {
                            let (_, funcs) = decode_code_section(section_contents)?;
                            module.code_section = Some(funcs);
                        }
                        _ => todo!(),
                    };

                    remaining = rest;
                }
                Err(err) => return Err(err),
            }
        }

        Ok((input, module))
    }
}

fn decode_section_header(input: &[u8]) -> IResult<&[u8], (SectionCode, u32)> {
    let (input, code) = le_u8(input)?;
    let (input, size) = match leb128_u32::<&[u8], ()>(input) {
        Ok(result) => result,
        Err(_) => {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Verify,
            )));
        }
    };
    Ok((
        input,
        (
            SectionCode::from_u8(code).expect("unexpected section code"),
            size,
        ),
    ))
}

fn decode_value_type(input: &[u8]) -> IResult<&[u8], ValueType> {
    let (input, value_type) = le_u8(input)?;
    Ok((input, value_type.into()))
}

fn decode_type_section(input: &[u8]) -> IResult<&[u8], Vec<FuncType>> {
    let mut func_types: Vec<FuncType> = vec![];

    let (mut input, count) = match leb128_u32::<&[u8], ()>(input) {
        Ok(result) => result,
        Err(_) => {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Verify,
            )));
        }
    };

    for _ in 0..count {
        let (rest, _) = le_u8(input)?;
        let mut func = FuncType::default();

        let (rest, size) = match leb128_u32::<&[u8], ()>(rest) {
            Ok(result) => result,
            Err(_) => {
                return Err(nom::Err::Error(nom::error::Error::new(
                    rest,
                    nom::error::ErrorKind::Verify,
                )));
            }
        };
        let (rest, types) = take(size)(rest)?;
        let (_, types) = many0(decode_value_type).parse(types)?;
        func.params = types;

        let (rest, size) = match leb128_u32::<&[u8], ()>(rest) {
            Ok(result) => result,
            Err(_) => {
                return Err(nom::Err::Error(nom::error::Error::new(
                    rest,
                    nom::error::ErrorKind::Verify,
                )));
            }
        };
        let (rest, types) = take(size)(rest)?;
        let (_, types) = many0(decode_value_type).parse(types)?;
        func.results = types;

        func_types.push(func);
        input = rest;
    }

    Ok((&[], func_types))
}

fn decode_function_section(input: &[u8]) -> IResult<&[u8], Vec<u32>> {
    let mut func_idx_list = vec![];
    let (mut input, count) = match leb128_u32::<&[u8], ()>(input) {
        Ok(result) => result,
        Err(_) => {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Verify,
            )));
        }
    };

    for _ in 0..count {
        let (rest, idx) = match leb128_u32::<&[u8], ()>(input) {
            Ok(result) => result,
            Err(_) => {
                return Err(nom::Err::Error(nom::error::Error::new(
                    input,
                    nom::error::ErrorKind::Verify,
                )));
            }
        };
        func_idx_list.push(idx);
        input = rest;
    }

    Ok((&[], func_idx_list))
}

fn decode_function_body(input: &[u8]) -> IResult<&[u8], Function> {
    let mut body = Function::default();

    let (mut input, count) = match leb128_u32::<&[u8], ()>(input) {
        Ok(result) => result,
        Err(_) => {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Verify,
            )));
        }
    };

    for _ in 0..count {
        let (rest, type_count) = match leb128_u32::<&[u8], ()>(input) {
            Ok(result) => result,
            Err(_) => {
                return Err(nom::Err::Error(nom::error::Error::new(
                    input,
                    nom::error::ErrorKind::Verify,
                )));
            }
        };
        let (rest, value_type) = le_u8(rest)?;
        body.locals.push(FunctionLocal {
            type_count,
            value_type: value_type.into(),
        });
        input = rest;
    }

    body.code = vec![Instruction::End];
    Ok((&[], body))
}

fn decode_code_section(input: &[u8]) -> IResult<&[u8], Vec<Function>> {
    let mut functions = vec![];
    let (mut input, count) = match leb128_u32::<&[u8], ()>(input) {
        Ok(result) => result,
        Err(_) => {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Verify,
            )));
        }
    };

    for _ in 0..count {
        let (rest, size) = match leb128_u32::<&[u8], ()>(input) {
            Ok(result) => result,
            Err(_) => {
                return Err(nom::Err::Error(nom::error::Error::new(
                    input,
                    nom::error::ErrorKind::Verify,
                )));
            }
        };
        let (rest, body) = take(size)(rest)?;
        let (_, body) = decode_function_body(body)?;
        functions.push(body);
        input = rest;
    }

    Ok((&[], functions))
}
