use log::trace;

use crate::middle_end::compile_time_eval::eval_integral_constant_expression;
use crate::middle_end::context::Context;
use crate::middle_end::ir::Program;
use crate::middle_end::ir_types::{EnumConstant, IrType, StructType, TypeSize, UnionType};
use crate::middle_end::middle_end_error::MiddleEndError;
use crate::parser::ast::{
    ArithmeticType, Declarator, EnumType, Enumerator, Identifier, ParameterTypeList,
    SpecifierQualifier, StorageClassSpecifier, StructType as AstStructType, TypeSpecifier,
    UnionType as AstUnionType,
};

pub type FunctionParameterBindings = Vec<(String, IrType)>;

pub fn get_type_info(
    specifier: &SpecifierQualifier,
    declarator: Option<Declarator>,
    is_duplicate_specifier: bool,
    prog: &mut Program,
    context: &mut Context,
) -> Result<
    Option<(
        IrType,
        Option<String>,
        // parameter bindings, if this is a function definition
        Option<FunctionParameterBindings>,
    )>,
    MiddleEndError,
> {
    let ir_type = match &specifier.type_specifier {
        TypeSpecifier::ArithmeticType(t) => match t {
            ArithmeticType::I8 => IrType::I8,
            ArithmeticType::U8 => IrType::U8,
            ArithmeticType::I16 => IrType::I16,
            ArithmeticType::U16 => IrType::U16,
            ArithmeticType::I32 => IrType::I32,
            ArithmeticType::U32 => IrType::U32,
            ArithmeticType::I64 => IrType::I64,
            ArithmeticType::U64 => IrType::U64,
            ArithmeticType::F32 => IrType::F32,
            ArithmeticType::F64 => IrType::F64,
        },
        TypeSpecifier::Void => IrType::Void,
        TypeSpecifier::Struct(struct_type) => match struct_type {
            AstStructType::Declaration(Identifier(struct_name)) => {
                // check if this is referencing a previous struct declaration
                match context.resolve_struct_tag_to_struct_id(struct_name) {
                    Ok(struct_id) => IrType::Struct(struct_id),
                    Err(MiddleEndError::UndeclaredStructTag(_)) => {
                        let struct_type_id =
                            prog.add_struct_type(StructType::named(struct_name.to_owned()))?;
                        context
                            .add_struct_tag(struct_name.to_owned(), struct_type_id.to_owned())?;
                        IrType::Struct(struct_type_id)
                    }
                    Err(e) => return Err(e),
                }
            }
            AstStructType::Definition(struct_name, members) => {
                let mut struct_type = match struct_name {
                    Some(Identifier(name)) => StructType::named(name.to_owned()),
                    None => StructType::unnamed(),
                };
                for member in members {
                    for decl in &member.1 {
                        let (member_type_info, member_name, _params) = match get_type_info(
                            &member.0,
                            Some(decl.to_owned()),
                            false,
                            prog,
                            context,
                        )? {
                            None => return Err(MiddleEndError::InvalidTypedefDeclaration),
                            Some(x) => x,
                        };
                        if member_name.is_none() {
                            return Err(MiddleEndError::UnnamedStructMember);
                        }
                        struct_type.push_member(
                            member_name.unwrap(),
                            member_type_info,
                            &prog.program_metadata,
                        )?;
                    }
                }
                let struct_type_id = prog.add_struct_type(struct_type)?;
                if let Some(Identifier(name)) = struct_name {
                    context.add_struct_tag(name.to_owned(), struct_type_id.to_owned())?;
                }
                IrType::Struct(struct_type_id)
            }
        },
        TypeSpecifier::Union(union_type) => match union_type {
            AstUnionType::Declaration(Identifier(union_name)) => {
                // check if this is referencing a previous union declaration
                match context.resolve_union_tag_to_union_id(union_name) {
                    Ok(union_id) => IrType::Union(union_id),
                    Err(MiddleEndError::UndeclaredUnionTag(_)) => {
                        let union_type_id =
                            prog.add_union_type(UnionType::named(union_name.to_owned()))?;
                        context.add_union_tag(union_name.to_owned(), union_type_id.to_owned())?;
                        IrType::Union(union_type_id)
                    }
                    Err(e) => return Err(e),
                }
            }
            AstUnionType::Definition(union_name, members) => {
                let mut union_type = match union_name {
                    Some(Identifier(name)) => UnionType::named(name.to_owned()),
                    None => UnionType::unnamed(),
                };
                for member in members {
                    for decl in &member.1 {
                        let (member_type_info, member_name, _params) = match get_type_info(
                            &member.0,
                            Some(decl.to_owned()),
                            false,
                            prog,
                            context,
                        )? {
                            None => return Err(MiddleEndError::InvalidTypedefDeclaration),
                            Some(x) => x,
                        };
                        if member_name.is_none() {
                            return Err(MiddleEndError::UnnamedUnionMember);
                        }
                        union_type.push_member(
                            member_name.unwrap(),
                            member_type_info,
                            &prog.program_metadata,
                        )?;
                    }
                }
                let union_type_id = prog.add_union_type(union_type)?;
                if let Some(Identifier(name)) = union_name {
                    context.add_union_tag(name.to_owned(), union_type_id.to_owned())?;
                }
                IrType::Union(union_type_id)
            }
        },
        TypeSpecifier::Enum(enum_type) => {
            match enum_type {
                EnumType::Declaration(Identifier(enum_name)) => {
                    context.resolve_identifier_to_enum_tag(enum_name)?;
                    // enums are just integers
                    IrType::I32
                }
                EnumType::Definition(enum_name, enum_constants) => {
                    let mut skip_constant_definition = false;
                    if let Some(Identifier(enum_name)) = enum_name {
                        if is_duplicate_specifier {
                            context.resolve_identifier_to_enum_tag(enum_name)?;
                            skip_constant_definition = true;
                        } else {
                            context.add_enum_tag(enum_name.to_owned())?;
                        }
                    }
                    if !skip_constant_definition {
                        let mut next_constant_value = 0;
                        for enum_constant in enum_constants {
                            match enum_constant {
                                Enumerator::Simple(Identifier(name)) => {
                                    context
                                        .add_enum_constant(name.to_owned(), next_constant_value)?;
                                    next_constant_value += 1;
                                }
                                Enumerator::WithValue(Identifier(name), value_expr) => {
                                    let value =
                                        eval_integral_constant_expression(value_expr.to_owned())?
                                            as EnumConstant;
                                    context.add_enum_constant(name.to_owned(), value)?;
                                    // value of next constant without explicit value is one more than
                                    // the last constant
                                    next_constant_value = value + 1;
                                }
                            }
                        }
                    }
                    IrType::I32
                }
            }
        }
        TypeSpecifier::CustomType(Identifier(name)) => context.resolve_typedef(name)?,
    };

    let mut is_typedef = false;
    match specifier.storage_class_specifier {
        None => {}
        Some(StorageClassSpecifier::Typedef) => {
            is_typedef = true;
        }
        Some(StorageClassSpecifier::Auto) => {
            // todo storage class specifiers
            trace!("ignoring storage class specifier: auto")
        }
        Some(StorageClassSpecifier::Extern) => {
            // todo storage class specifiers
            trace!("ignoring storage class specifier: extern")
        }
        Some(StorageClassSpecifier::Register) => {
            // todo storage class specifiers
            trace!("ignoring storage class specifier: register")
        }
        Some(StorageClassSpecifier::Static) => {
            // todo storage class specifiers
            trace!("ignoring storage class specifier: static")
        }
    }

    match declarator {
        Some(decl) => {
            let (ir_type, decl_name, param_bindings) =
                add_type_info_from_declarator(decl, ir_type, prog, context)?;
            if is_typedef {
                context.add_typedef(decl_name.unwrap(), ir_type)?;
                return Ok(None);
            }
            Ok(Some((ir_type, decl_name, param_bindings)))
        }
        None => Ok(Some((ir_type, None, None))),
    }
}

/// Modifies the TypeInfo struct it's given, and returns the identifier name,
/// and the types of any parameters
fn add_type_info_from_declarator(
    decl: Declarator,
    type_info: IrType,
    prog: &mut Program,
    context: &mut Context,
) -> Result<(IrType, Option<String>, Option<FunctionParameterBindings>), MiddleEndError> {
    match decl {
        Declarator::Identifier(Identifier(name)) => Ok((type_info, Some(name), None)),
        Declarator::PointerDeclarator(d) => {
            add_type_info_from_declarator(*d, type_info.wrap_with_pointer(), prog, context)
        }
        Declarator::AbstractPointerDeclarator => {
            // Err(MiddleEndError::InvalidAbstractDeclarator)
            // todo handle abstract parameters in function declaration
            Ok((type_info.wrap_with_pointer(), None, None))
        }
        Declarator::ArrayDeclarator(d, size_expr) => {
            let size = match size_expr {
                None => None,
                Some(size_expr) => match eval_integral_constant_expression(*size_expr.to_owned()) {
                    Ok(size) => Some(TypeSize::CompileTime(size as u64)),
                    Err(_) => Some(TypeSize::Runtime(*size_expr)),
                },
            };
            add_type_info_from_declarator(*d, type_info.wrap_with_array(size), prog, context)
        }
        Declarator::FunctionDeclarator(d, params) => {
            let mut is_variadic = false;
            let param_decls = match params {
                None => Vec::new(),
                Some(params) => match params {
                    ParameterTypeList::Normal(params) => params,
                    ParameterTypeList::Variadic(params) => {
                        is_variadic = true;
                        params
                    }
                },
            };

            let mut param_types: Vec<IrType> = Vec::new();
            let mut param_bindings: FunctionParameterBindings = Vec::new();
            for p in param_decls {
                let (param_type, param_name, _sub_param_bindings) =
                    match get_type_info(&p.0, p.1, false, prog, context)? {
                        None => return Err(MiddleEndError::InvalidTypedefDeclaration),
                        Some(x) => x,
                    };
                param_types.push(param_type.to_owned());
                if let Some(name) = param_name {
                    param_bindings.push((name, param_type));
                }
            }

            let (type_info, name, _) = add_type_info_from_declarator(
                *d,
                type_info.wrap_with_fun(param_types, is_variadic),
                prog,
                context,
            )?;
            Ok((type_info, name, Some(param_bindings)))
        }
    }
}
