// use crate::middle_end::ids::ValueType;
// use crate::middle_end::instructions::Instruction;
// use crate::middle_end::instructions::Instruction::SimpleAssignment;
// use crate::middle_end::ir::{Function, Program, ProgramMetadata};
// use crate::middle_end::ir_types::IrType;
//
// pub fn convert_to_wasm_types(prog: &mut Box<Program>) {
//     for (_, function) in prog.program_instructions.functions {}
// }
//
// fn convert_function_to_wasm_types(function: Function, prog_metadata: &mut ProgramMetadata) -> Function {
//     let mut new_instrs = Vec::new();
//     for instr in function.instrs {
//         match instr {
//             Instruction::SimpleAssignment(dest, src) => {
//                 let dest_type = prog_metadata.get_var_type(dest).unwrap();
//                 if dest_type.is_wasm_type() {
//                     continue;
//                 }
//                 match *dest_type {
//                     IrType::I8 => {
//                         let temp = prog_metadata.new_var(ValueType::ModifiableLValue);
//                         new_instrs.push(SimpleAssignment(temp, src));
//                     }
//                     IrType::U8 => {}
//                     IrType::I16 => {}
//                     IrType::U16 => {}
//                     IrType::PointerTo(_) => {}
//                     _ => unreachable!()
//                 }
//             }
//             Instruction::LoadFromAddress(_, _) => {}
//             Instruction::StoreToAddress(_, _) => {}
//             Instruction::AllocateVariable(_, _) => {}
//             Instruction::AddressOf(_, _) => {}
//             Instruction::BitwiseNot(_, _) => {}
//             Instruction::LogicalNot(_, _) => {}
//             Instruction::Mult(_, _, _) => {}
//             Instruction::Div(_, _, _) => {}
//             Instruction::Mod(_, _, _) => {}
//             Instruction::Add(_, _, _) => {}
//             Instruction::Sub(_, _, _) => {}
//             Instruction::LeftShift(_, _, _) => {}
//             Instruction::RightShift(_, _, _) => {}
//             Instruction::BitwiseAnd(_, _, _) => {}
//             Instruction::BitwiseOr(_, _, _) => {}
//             Instruction::BitwiseXor(_, _, _) => {}
//             Instruction::LogicalAnd(_, _, _) => {}
//             Instruction::LogicalOr(_, _, _) => {}
//             Instruction::LessThan(_, _, _) => {}
//             Instruction::GreaterThan(_, _, _) => {}
//             Instruction::LessThanEq(_, _, _) => {}
//             Instruction::GreaterThanEq(_, _, _) => {}
//             Instruction::Equal(_, _, _) => {}
//             Instruction::NotEqual(_, _, _) => {}
//             Instruction::Call(_, _, _) => {}
//             Instruction::Ret(_) => {}
//             Instruction::Label(_) => {}
//             Instruction::Br(_) => {}
//             Instruction::BrIfEq(_, _, _) => {}
//             Instruction::BrIfNotEq(_, _, _) => {}
//             Instruction::PointerToStringLiteral(_, _) => {}
//             Instruction::I8toI16(_, _) => {}
//             Instruction::I8toU16(_, _) => {}
//             Instruction::U8toI16(_, _) => {}
//             Instruction::U8toU16(_, _) => {}
//             Instruction::I16toI32(_, _) => {}
//             Instruction::U16toI32(_, _) => {}
//             Instruction::I16toU32(_, _) => {}
//             Instruction::U16toU32(_, _) => {}
//             Instruction::I32toU32(_, _) => {}
//             Instruction::I32toU64(_, _) => {}
//             Instruction::U32toU64(_, _) => {}
//             Instruction::I64toU64(_, _) => {}
//             Instruction::I32toI64(_, _) => {}
//             Instruction::U32toI64(_, _) => {}
//             Instruction::U32toF32(_, _) => {}
//             Instruction::I32toF32(_, _) => {}
//             Instruction::U64toF32(_, _) => {}
//             Instruction::I64toF32(_, _) => {}
//             Instruction::U32toF64(_, _) => {}
//             Instruction::I32toF64(_, _) => {}
//             Instruction::U64toF64(_, _) => {}
//             Instruction::I64toF64(_, _) => {}
//             Instruction::F32toF64(_, _) => {}
//             Instruction::I32toI8(_, _) => {}
//             Instruction::U32toI8(_, _) => {}
//             Instruction::I64toI8(_, _) => {}
//             Instruction::U64toI8(_, _) => {}
//             Instruction::I32toU8(_, _) => {}
//             Instruction::U32toU8(_, _) => {}
//             Instruction::I64toU8(_, _) => {}
//             Instruction::U64toU8(_, _) => {}
//             Instruction::I64toI32(_, _) => {}
//             Instruction::U64toI32(_, _) => {}
//             Instruction::U32toPtr(_, _) => {}
//             Instruction::I32toPtr(_, _) => {}
//             Instruction::Nop => {}
//             Instruction::Break(_) => {}
//             Instruction::Continue(_) => {}
//             Instruction::EndHandledBlock(_) => {}
//             Instruction::IfEqElse(_, _, _, _) => {}
//             Instruction::IfNotEqElse(_, _, _, _) => {}
//         }
//     }
//
//     todo!()
// }
