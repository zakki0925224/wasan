pub mod instruction;
pub mod module;
pub mod opecode;
pub mod section;
pub mod types;

#[cfg(test)]
mod tests {
    use crate::binary::{
        instruction::Instruction,
        module::Module,
        section::Function,
        types::{FuncType, FunctionLocal, ValueType},
    };
    use anyhow::Result;

    #[test]
    fn decode_simplest_module() -> Result<()> {
        let wasm = wat::parse_str("(module)")?;
        let module = Module::new(&wasm)?;

        assert_eq!(module, Module::default());
        Ok(())
    }

    #[test]
    fn decode_simplest_func() -> Result<()> {
        let wasm = wat::parse_str("(module (func))")?;
        let module = Module::new(&wasm)?;
        assert_eq!(
            module,
            Module {
                type_section: Some(vec![FuncType::default()]),
                function_section: Some(vec![0]),
                code_section: Some(vec![Function {
                    locals: vec![],
                    code: vec![Instruction::End],
                }]),
                ..Default::default()
            }
        );

        Ok(())
    }

    #[test]
    fn decode_func_param() -> Result<()> {
        let wasm = wat::parse_str("(module (func (param i32 i64)))")?;
        let module = Module::new(&wasm)?;
        assert_eq!(
            module,
            Module {
                type_section: Some(vec![FuncType {
                    params: vec![ValueType::I32, ValueType::I64],
                    results: vec![]
                }]),
                function_section: Some(vec![0]),
                code_section: Some(vec![Function {
                    locals: vec![],
                    code: vec![Instruction::End]
                }]),
                ..Default::default()
            }
        );

        Ok(())
    }

    #[test]
    fn decode_func_local() -> Result<()> {
        let wasm = wat::parse_file("src/fixtures/func_local.wat")?;
        let module = Module::new(&wasm)?;
        assert_eq!(
            module,
            Module {
                type_section: Some(vec![FuncType::default()]),
                function_section: Some(vec![0]),
                code_section: Some(vec![Function {
                    locals: vec![
                        FunctionLocal {
                            type_count: 1,
                            value_type: ValueType::I32
                        },
                        FunctionLocal {
                            type_count: 2,
                            value_type: ValueType::I64
                        },
                    ],
                    code: vec![Instruction::End],
                }]),
                ..Default::default()
            }
        );
        Ok(())
    }

    #[test]
    fn decode_func_add() -> Result<()> {
        let wasm = wat::parse_file("src/fixtures/func_add.wat")?;
        let module = Module::new(&wasm)?;
        assert_eq!(
            module,
            Module {
                type_section: Some(vec![FuncType {
                    params: vec![ValueType::I32, ValueType::I32],
                    results: vec![ValueType::I32],
                }]),
                function_section: Some(vec![0]),
                code_section: Some(vec![Function {
                    locals: vec![],
                    code: vec![
                        Instruction::LocalGet(0),
                        Instruction::LocalGet(1),
                        Instruction::I32Add,
                        Instruction::End
                    ],
                }]),
                ..Default::default()
            }
        );
        Ok(())
    }
}
