use crate::middle_end::ids::{StructId, UnionId};
use crate::middle_end::ir::Program;
use crate::middle_end::middle_end_error::{MiddleEndError, TypeError};
use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;

const POINTER_SIZE: u64 = 4; // bytes

// enum constants are represented as ints
pub type EnumConstant = i32;

#[derive(Debug, Clone, PartialEq)]
pub enum IrType {
    I8,  // signed char
    U8,  // unsigned char
    I16, // signed short
    U16, // unsigned short
    I32, // signed int
    U32, // unsigned int
    I64, // signed long
    U64, // unsigned long
    F32, // float
    F64, // double
    Struct(StructId),
    Union(UnionId),
    Void,
    PointerTo(Box<IrType>),
    /// array type, array size
    ArrayOf(Box<IrType>, u64),
    /// return type, parameter types
    Function(Box<IrType>, Vec<Box<IrType>>),
}

impl IrType {
    pub fn get_byte_size(&self, prog: &Box<Program>) -> u64 {
        match &self {
            IrType::I8 | IrType::U8 => 1,
            IrType::I16 | IrType::U16 => 2,
            IrType::I32 | IrType::U32 => 4,
            IrType::I64 | IrType::U64 => 8,
            IrType::F32 => 4,
            IrType::F64 => 8,
            IrType::Struct(struct_id) => prog.get_struct_type(struct_id).unwrap().total_byte_size,
            IrType::Union(union_id) => prog.get_union_type(union_id).unwrap().total_byte_size,
            IrType::Void => 0,
            IrType::PointerTo(_) => POINTER_SIZE,
            IrType::ArrayOf(t, count) => t.get_byte_size(prog) * count,
            IrType::Function(_, _) => POINTER_SIZE,
        }
    }

    pub fn wrap_with_pointer(self) -> Box<Self> {
        Box::new(IrType::PointerTo(Box::new(self)))
    }

    pub fn wrap_with_array(self, size: u64) -> Box<Self> {
        Box::new(IrType::ArrayOf(Box::new(self), size))
    }

    pub fn wrap_with_fun(self, params: Vec<Box<IrType>>) -> Box<Self> {
        Box::new(IrType::Function(Box::new(self), params))
    }

    pub fn is_signed_integral(&self) -> bool {
        match self {
            IrType::I8 | IrType::I16 | IrType::I32 | IrType::I64 => true,
            IrType::U8
            | IrType::U16
            | IrType::U32
            | IrType::U64
            | IrType::F32
            | IrType::F64
            | IrType::Struct(_)
            | IrType::Union(_)
            | IrType::Void
            | IrType::PointerTo(_)
            | IrType::ArrayOf(_, _)
            | IrType::Function(_, _) => false,
        }
    }

    pub fn is_unsigned_integral(&self) -> bool {
        match self {
            IrType::U8 | IrType::U16 | IrType::U32 | IrType::U64 => true,
            IrType::I8
            | IrType::I16
            | IrType::I32
            | IrType::I64
            | IrType::F32
            | IrType::F64
            | IrType::Struct(_)
            | IrType::Union(_)
            | IrType::Void
            | IrType::PointerTo(_)
            | IrType::ArrayOf(_, _)
            | IrType::Function(_, _) => false,
        }
    }

    pub fn smallest_signed_equivalent(&self) -> Result<Box<Self>, MiddleEndError> {
        match self {
            IrType::U8 => Ok(Box::new(IrType::I16)), // go up one size cos might be bigger than can fit
            IrType::U16 => Ok(Box::new(IrType::I32)),
            IrType::U32 => Ok(Box::new(IrType::I64)),
            IrType::U64 => Ok(Box::new(IrType::I64)),
            IrType::I8 | IrType::I16 | IrType::I32 | IrType::I64 | IrType::F32 | IrType::F64 => {
                Ok(Box::new(self.to_owned()))
            }
            IrType::Struct(_)
            | IrType::Union(_)
            | IrType::Void
            | IrType::PointerTo(_)
            | IrType::ArrayOf(_, _)
            | IrType::Function(_, _) => {
                Err(MiddleEndError::TypeError(TypeError::TypeConversionError(
                    "Cannot convert to signed",
                    Box::new(self.to_owned()),
                    None,
                )))
            }
        }
    }

    pub fn is_integral_type(&self) -> bool {
        match self {
            IrType::I8
            | IrType::U8
            | IrType::I16
            | IrType::U16
            | IrType::I32
            | IrType::U32
            | IrType::I64
            | IrType::U64 => true,
            IrType::F32
            | IrType::F64
            | IrType::PointerTo(_)
            | IrType::Struct(_)
            | IrType::Union(_)
            | IrType::Void
            | IrType::ArrayOf(_, _)
            | IrType::Function(_, _) => false,
        }
    }

    /// Returns an error if self isn't an integral type
    pub fn require_integral_type(&self) -> Result<(), MiddleEndError> {
        match self.is_integral_type() {
            true => Ok(()),
            false => Err(MiddleEndError::TypeError(TypeError::InvalidOperation(
                "Require integral type failed",
            ))),
        }
    }

    pub fn is_arithmetic_type(&self) -> bool {
        match self {
            IrType::I8
            | IrType::U8
            | IrType::I16
            | IrType::U16
            | IrType::I32
            | IrType::U32
            | IrType::I64
            | IrType::U64
            | IrType::F32
            | IrType::F64 => true,
            IrType::PointerTo(_)
            | IrType::Struct(_)
            | IrType::Union(_)
            | IrType::Void
            | IrType::ArrayOf(_, _)
            | IrType::Function(_, _) => false,
        }
    }

    /// Returns an error if self isn't an arithmetic type
    pub fn require_arithmetic_type(&self) -> Result<(), MiddleEndError> {
        match self.is_arithmetic_type() {
            true => Ok(()),
            false => Err(MiddleEndError::TypeError(TypeError::InvalidOperation(
                "Require arithmetic type failed",
            ))),
        }
    }

    pub fn is_scalar_type(&self) -> bool {
        match self {
            IrType::I8
            | IrType::U8
            | IrType::I16
            | IrType::U16
            | IrType::I32
            | IrType::U32
            | IrType::I64
            | IrType::U64
            | IrType::F32
            | IrType::F64
            | IrType::PointerTo(_) => true,
            IrType::Struct(_)
            | IrType::Union(_)
            | IrType::Void
            | IrType::ArrayOf(_, _)
            | IrType::Function(_, _) => false,
        }
    }

    /// Returns an error if self isn't a scalar type
    pub fn require_scalar_type(&self) -> Result<(), MiddleEndError> {
        match self.is_scalar_type() {
            true => Ok(()),
            false => Err(MiddleEndError::TypeError(TypeError::InvalidOperation(
                "Require scalar type failed",
            ))),
        }
    }

    pub fn is_object_pointer_type(&self) -> bool {
        match self {
            IrType::PointerTo(t) => match **t {
                IrType::Function(_, _) | IrType::Void => false,
                _ => true,
            },
            IrType::I8
            | IrType::U8
            | IrType::I16
            | IrType::U16
            | IrType::I32
            | IrType::U32
            | IrType::I64
            | IrType::U64
            | IrType::F32
            | IrType::F64
            | IrType::Struct(_)
            | IrType::Union(_)
            | IrType::Void
            | IrType::ArrayOf(_, _)
            | IrType::Function(_, _) => false,
        }
    }

    /// Returns an error if self isn't a pointer type
    pub fn require_object_pointer_type(&self) -> Result<(), MiddleEndError> {
        match self.is_object_pointer_type() {
            true => Ok(()),
            false => Err(MiddleEndError::TypeError(TypeError::InvalidOperation(
                "Require object pointer type failed",
            ))),
        }
    }

    pub fn is_pointer_type(&self) -> bool {
        match self {
            IrType::PointerTo(_) => true,
            IrType::I8
            | IrType::U8
            | IrType::I16
            | IrType::U16
            | IrType::I32
            | IrType::U32
            | IrType::I64
            | IrType::U64
            | IrType::F32
            | IrType::F64
            | IrType::Struct(_)
            | IrType::Union(_)
            | IrType::Void
            | IrType::ArrayOf(_, _)
            | IrType::Function(_, _) => false,
        }
    }

    /// Returns an error if self isn't a pointer type
    pub fn require_pointer_type(&self) -> Result<(), MiddleEndError> {
        match self.is_pointer_type() {
            true => Ok(()),
            false => Err(MiddleEndError::TypeError(TypeError::InvalidOperation(
                "Require pointer type failed",
            ))),
        }
    }

    pub fn is_struct_or_union_type(&self) -> bool {
        match self {
            IrType::Struct(_) | IrType::Union(_) => true,
            IrType::I8
            | IrType::U8
            | IrType::I16
            | IrType::U16
            | IrType::I32
            | IrType::U32
            | IrType::I64
            | IrType::U64
            | IrType::F32
            | IrType::F64
            | IrType::Void
            | IrType::PointerTo(_)
            | IrType::ArrayOf(_, _)
            | IrType::Function(_, _) => false,
        }
    }

    /// Returns an error if self isn't a struct or union type
    pub fn require_struct_or_union_type(&self) -> Result<(), MiddleEndError> {
        match self.is_struct_or_union_type() {
            true => Ok(()),
            false => Err(MiddleEndError::TypeError(TypeError::InvalidOperation(
                "Require struct/union type failed",
            ))),
        }
    }

    /// ISO C standard unary type conversions
    pub fn unary_convert(&self) -> Box<Self> {
        match self {
            IrType::I8 | IrType::U8 | IrType::U16 | IrType::I16 | IrType::I32 => {
                Box::new(IrType::I32)
            }
            IrType::U32
            | IrType::I64
            | IrType::U64
            | IrType::F32
            | IrType::F64
            | IrType::Struct(_)
            | IrType::Union(_)
            | IrType::PointerTo(_)
            | IrType::Void => Box::new(self.to_owned()),
            IrType::ArrayOf(t, _) => Box::new(IrType::PointerTo(t.to_owned())),
            IrType::Function(_, _) => Box::new(IrType::PointerTo(Box::new(self.to_owned()))),
        }
    }

    /// Return the type that this type points to, or an error if not a pointer type
    pub fn dereference_pointer_type(&self) -> Result<Box<Self>, MiddleEndError> {
        match self {
            IrType::PointerTo(t) => Ok(t.to_owned()),
            t => Err(MiddleEndError::TypeError(
                TypeError::DereferenceNonPointerType(Box::new(t.to_owned())),
            )),
        }
    }
}

impl fmt::Display for IrType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            IrType::I8 => {
                write!(f, "signed char")
            }
            IrType::U8 => {
                write!(f, "unsigned char")
            }
            IrType::I16 => {
                write!(f, "signed short")
            }
            IrType::U16 => {
                write!(f, "unsigned short")
            }
            IrType::I32 => {
                write!(f, "signed int")
            }
            IrType::U32 => {
                write!(f, "unsigned int")
            }
            IrType::I64 => {
                write!(f, "signed long")
            }
            IrType::U64 => {
                write!(f, "unsigned long")
            }
            IrType::F32 => {
                write!(f, "float")
            }
            IrType::F64 => {
                write!(f, "double")
            }
            IrType::Struct(struct_id) => {
                write!(f, "struct {}", struct_id)
            }
            IrType::Union(union_id) => {
                write!(f, "union {}", union_id)
            }
            IrType::Void => {
                write!(f, "void")
            }
            IrType::PointerTo(t) => {
                write!(f, "*({})", t)
            }
            IrType::ArrayOf(t, size) => {
                write!(f, "({})[{}]", t, size)
            }
            IrType::Function(ret, params) => {
                write!(f, "({})(", ret)?;
                for param in &params[..params.len() - 1] {
                    write!(f, "{}, ", param)?;
                }
                write!(f, "{})", params[params.len() - 1])
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructType {
    pub name: Option<String>,
    /// store members' names and types
    pub member_types: HashMap<String, Box<IrType>>,
    pub member_byte_offsets: HashMap<String, u64>,
    pub total_byte_size: u64,
}

impl StructType {
    pub fn named(name: String) -> Self {
        StructType {
            name: Some(name),
            member_types: HashMap::new(),
            member_byte_offsets: HashMap::new(),
            total_byte_size: 0,
        }
    }

    pub fn unnamed() -> Self {
        StructType {
            name: None,
            member_types: HashMap::new(),
            member_byte_offsets: HashMap::new(),
            total_byte_size: 0,
        }
    }

    pub fn push_member(
        &mut self,
        member_name: String,
        member_type: Box<IrType>,
        prog: &Box<Program>,
    ) -> Result<(), MiddleEndError> {
        // check if member with same name already exists
        if self.has_member(&member_name) {
            return Err(MiddleEndError::DuplicateStructMember);
        }
        let byte_size = member_type.get_byte_size(prog);
        self.member_types
            .insert(member_name.to_owned(), member_type);
        // store byte offset of this member and update total byte size of struct
        self.member_byte_offsets
            .insert(member_name, self.total_byte_size);
        self.total_byte_size += byte_size;
        Ok(())
    }

    pub fn has_member(&self, member_name: &str) -> bool {
        self.member_types.contains_key(member_name)
    }

    pub fn get_member_type(&self, member_name: &str) -> Result<Box<IrType>, MiddleEndError> {
        match self.member_types.get(member_name) {
            None => Err(MiddleEndError::StructMemberNotFound(format!(
                "{}.{}",
                self.name.to_owned().unwrap_or("".to_owned()),
                member_name
            ))),
            Some(t) => Ok(t.to_owned()),
        }
    }

    pub fn get_member_byte_offset(&self, member_name: &str) -> Result<u64, MiddleEndError> {
        match self.member_byte_offsets.get(member_name) {
            None => Err(MiddleEndError::StructMemberNotFound(format!(
                "{}.{}",
                self.name.to_owned().unwrap_or("".to_owned()),
                member_name
            ))),
            Some(offset) => Ok(offset.to_owned()),
        }
    }
}

impl fmt::Display for StructType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.name {
            None => write!(f, "unnamed struct")?,
            Some(name) => write!(f, "struct \"{}\"", name)?,
        }
        write!(f, "\nMembers:")?;
        for (member_name, member_type) in &self.member_types {
            let byte_offset = self.member_byte_offsets.get(member_name).unwrap();
            write!(
                f,
                "\n\"{}\" at byte {}: {}",
                member_name, byte_offset, member_type
            )?;
        }
        write!(f, "\nTotal byte size: {}", self.total_byte_size)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnionType {
    pub name: Option<String>,
    /// store members' names and types
    pub member_types: HashMap<String, Box<IrType>>,
    pub total_byte_size: u64,
}

impl UnionType {
    pub fn named(name: String) -> Self {
        UnionType {
            name: Some(name),
            member_types: HashMap::new(),
            total_byte_size: 0,
        }
    }

    pub fn unnamed() -> Self {
        UnionType {
            name: None,
            member_types: HashMap::new(),
            total_byte_size: 0,
        }
    }

    pub fn push_member(
        &mut self,
        member_name: String,
        member_type: Box<IrType>,
        prog: &Box<Program>,
    ) -> Result<(), MiddleEndError> {
        // check if another member with same name already exists
        if self.has_member(&member_name) {
            return Err(MiddleEndError::DuplicateUnionMember);
        }
        let byte_size = member_type.get_byte_size(prog);
        self.member_types.insert(member_name, member_type);
        // total size of union is the size of the largest member
        if byte_size > self.total_byte_size {
            self.total_byte_size = byte_size;
        }
        Ok(())
    }

    pub fn has_member(&self, member_name: &str) -> bool {
        self.member_types.contains_key(member_name)
    }

    pub fn get_member_type(&self, member_name: &str) -> Result<Box<IrType>, MiddleEndError> {
        match self.member_types.get(member_name) {
            None => Err(MiddleEndError::UnionMemberNotFound(format!(
                "{}.{}",
                self.name.to_owned().unwrap_or("".to_owned()),
                member_name
            ))),
            Some(t) => Ok(t.to_owned()),
        }
    }
}

impl fmt::Display for UnionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.name {
            None => write!(f, "unnamed union")?,
            Some(name) => write!(f, "union \"{}\"", name)?,
        }
        write!(f, "\nMembers:")?;
        for (member_name, member_type) in &self.member_types {
            write!(f, "\n\"{}\": {}", member_name, member_type)?;
        }
        write!(f, "\nTotal byte size: {}", self.total_byte_size)
    }
}
