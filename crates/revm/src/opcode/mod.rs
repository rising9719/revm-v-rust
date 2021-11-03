#[macro_use]
mod macros;
mod arithmetic;
mod bitwise;
mod codes;
pub(crate) mod gas;
mod i256;
mod misc;
mod system;

pub use codes::OpCode;

use crate::{
    error::{ExitError, ExitReason, ExitSucceed},
    machine::Machine,
    spec::{Spec, SpecId::*},
    CallScheme, Handler,
};
use core::ops::{BitAnd, BitOr, BitXor};
use primitive_types::{H256, U256};

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Control {
    Continue,
    ContinueN(usize),
    Exit(ExitReason),
    Jump(usize),
}

#[inline(always)]
pub fn eval<H: Handler, S: Spec>(
    machine: &mut Machine,
    opcode: OpCode,
    position: usize,
    handler: &mut H,
) -> Control {
    match opcode {
        OpCode::STOP => Control::Exit(ExitSucceed::Stopped.into()),
        OpCode::ADD => op2_u256_tuple!(machine, overflowing_add, gas::VERYLOW),
        OpCode::MUL => op2_u256_tuple!(machine, overflowing_mul, gas::LOW),
        OpCode::SUB => op2_u256_tuple!(machine, overflowing_sub, gas::VERYLOW),
        OpCode::DIV => op2_u256_fn!(machine, arithmetic::div, gas::LOW),
        OpCode::SDIV => op2_u256_fn!(machine, arithmetic::sdiv, gas::LOW),
        OpCode::MOD => op2_u256_fn!(machine, arithmetic::rem, gas::LOW),
        OpCode::SMOD => op2_u256_fn!(machine, arithmetic::srem, gas::LOW),
        OpCode::ADDMOD => op3_u256_fn!(machine, arithmetic::addmod, gas::MID),
        OpCode::MULMOD => op3_u256_fn!(machine, arithmetic::mulmod, gas::MID),
        OpCode::EXP => arithmetic::eval_exp::<S>(machine),
        OpCode::SIGNEXTEND => op2_u256_fn!(machine, arithmetic::signextend, gas::LOW),
        OpCode::LT => op2_u256_bool_ref!(machine, lt, gas::VERYLOW),
        OpCode::GT => op2_u256_bool_ref!(machine, gt, gas::VERYLOW),
        OpCode::SLT => op2_u256_fn!(machine, bitwise::slt, gas::VERYLOW),
        OpCode::SGT => op2_u256_fn!(machine, bitwise::sgt, gas::VERYLOW),
        OpCode::EQ => op2_u256_bool_ref!(machine, eq, gas::VERYLOW),
        OpCode::ISZERO => op1_u256_fn!(machine, bitwise::iszero, gas::VERYLOW),
        OpCode::AND => op2_u256!(machine, bitand, gas::VERYLOW),
        OpCode::OR => op2_u256!(machine, bitor, gas::VERYLOW),
        OpCode::XOR => op2_u256!(machine, bitxor, gas::VERYLOW),
        OpCode::NOT => op1_u256_fn!(machine, bitwise::not, gas::VERYLOW),
        OpCode::BYTE => op2_u256_fn!(machine, bitwise::byte, gas::VERYLOW),
        OpCode::SHL => op2_u256_fn!(
            machine,
            bitwise::shl,
            gas::VERYLOW,
            S::enabled(CONSTANTINOPLE) // EIP-145: Bitwise shifting instructions in EVM
        ),
        OpCode::SHR => op2_u256_fn!(
            machine,
            bitwise::shr,
            gas::VERYLOW,
            S::enabled(CONSTANTINOPLE) // EIP-145: Bitwise shifting instructions in EVM
        ),
        OpCode::SAR => op2_u256_fn!(
            machine,
            bitwise::sar,
            gas::VERYLOW,
            S::enabled(CONSTANTINOPLE) // EIP-145: Bitwise shifting instructions in EVM
        ),
        OpCode::CODESIZE => misc::codesize::<S>(machine),
        OpCode::CODECOPY => misc::codecopy::<S>(machine),
        OpCode::CALLDATALOAD => misc::calldataload::<S>(machine),
        OpCode::CALLDATASIZE => misc::calldatasize::<S>(machine),
        OpCode::CALLDATACOPY => misc::calldatacopy::<S>(machine),
        OpCode::POP => misc::pop::<S>(machine),
        OpCode::MLOAD => misc::mload::<S>(machine),
        OpCode::MSTORE => misc::mstore::<S>(machine),
        OpCode::MSTORE8 => misc::mstore8::<S>(machine),
        OpCode::JUMP => misc::jump::<S>(machine),
        OpCode::JUMPI => misc::jumpi::<S>(machine),
        OpCode::PC => misc::pc::<S>(machine, position),
        OpCode::MSIZE => misc::msize::<S>(machine),
        OpCode::JUMPDEST => misc::jumpdest::<S>(machine),

        OpCode::PUSH1 => misc::push::<S>(machine, 1, position),
        OpCode::PUSH2 => misc::push::<S>(machine, 2, position),
        OpCode::PUSH3 => misc::push::<S>(machine, 3, position),
        OpCode::PUSH4 => misc::push::<S>(machine, 4, position),
        OpCode::PUSH5 => misc::push::<S>(machine, 5, position),
        OpCode::PUSH6 => misc::push::<S>(machine, 6, position),
        OpCode::PUSH7 => misc::push::<S>(machine, 7, position),
        OpCode::PUSH8 => misc::push::<S>(machine, 8, position),
        OpCode::PUSH9 => misc::push::<S>(machine, 9, position),
        OpCode::PUSH10 => misc::push::<S>(machine, 10, position),
        OpCode::PUSH11 => misc::push::<S>(machine, 11, position),
        OpCode::PUSH12 => misc::push::<S>(machine, 12, position),
        OpCode::PUSH13 => misc::push::<S>(machine, 13, position),
        OpCode::PUSH14 => misc::push::<S>(machine, 14, position),
        OpCode::PUSH15 => misc::push::<S>(machine, 15, position),
        OpCode::PUSH16 => misc::push::<S>(machine, 16, position),
        OpCode::PUSH17 => misc::push::<S>(machine, 17, position),
        OpCode::PUSH18 => misc::push::<S>(machine, 18, position),
        OpCode::PUSH19 => misc::push::<S>(machine, 19, position),
        OpCode::PUSH20 => misc::push::<S>(machine, 20, position),
        OpCode::PUSH21 => misc::push::<S>(machine, 21, position),
        OpCode::PUSH22 => misc::push::<S>(machine, 22, position),
        OpCode::PUSH23 => misc::push::<S>(machine, 23, position),
        OpCode::PUSH24 => misc::push::<S>(machine, 24, position),
        OpCode::PUSH25 => misc::push::<S>(machine, 25, position),
        OpCode::PUSH26 => misc::push::<S>(machine, 26, position),
        OpCode::PUSH27 => misc::push::<S>(machine, 27, position),
        OpCode::PUSH28 => misc::push::<S>(machine, 28, position),
        OpCode::PUSH29 => misc::push::<S>(machine, 29, position),
        OpCode::PUSH30 => misc::push::<S>(machine, 30, position),
        OpCode::PUSH31 => misc::push::<S>(machine, 31, position),
        OpCode::PUSH32 => misc::push::<S>(machine, 32, position),

        OpCode::DUP1 => misc::dup::<S>(machine, 1),
        OpCode::DUP2 => misc::dup::<S>(machine, 2),
        OpCode::DUP3 => misc::dup::<S>(machine, 3),
        OpCode::DUP4 => misc::dup::<S>(machine, 4),
        OpCode::DUP5 => misc::dup::<S>(machine, 5),
        OpCode::DUP6 => misc::dup::<S>(machine, 6),
        OpCode::DUP7 => misc::dup::<S>(machine, 7),
        OpCode::DUP8 => misc::dup::<S>(machine, 8),
        OpCode::DUP9 => misc::dup::<S>(machine, 9),
        OpCode::DUP10 => misc::dup::<S>(machine, 10),
        OpCode::DUP11 => misc::dup::<S>(machine, 11),
        OpCode::DUP12 => misc::dup::<S>(machine, 12),
        OpCode::DUP13 => misc::dup::<S>(machine, 13),
        OpCode::DUP14 => misc::dup::<S>(machine, 14),
        OpCode::DUP15 => misc::dup::<S>(machine, 15),
        OpCode::DUP16 => misc::dup::<S>(machine, 16),

        OpCode::SWAP1 => misc::swap::<S>(machine, 1),
        OpCode::SWAP2 => misc::swap::<S>(machine, 2),
        OpCode::SWAP3 => misc::swap::<S>(machine, 3),
        OpCode::SWAP4 => misc::swap::<S>(machine, 4),
        OpCode::SWAP5 => misc::swap::<S>(machine, 5),
        OpCode::SWAP6 => misc::swap::<S>(machine, 6),
        OpCode::SWAP7 => misc::swap::<S>(machine, 7),
        OpCode::SWAP8 => misc::swap::<S>(machine, 8),
        OpCode::SWAP9 => misc::swap::<S>(machine, 9),
        OpCode::SWAP10 => misc::swap::<S>(machine, 10),
        OpCode::SWAP11 => misc::swap::<S>(machine, 11),
        OpCode::SWAP12 => misc::swap::<S>(machine, 12),
        OpCode::SWAP13 => misc::swap::<S>(machine, 13),
        OpCode::SWAP14 => misc::swap::<S>(machine, 14),
        OpCode::SWAP15 => misc::swap::<S>(machine, 15),
        OpCode::SWAP16 => misc::swap::<S>(machine, 16),

        OpCode::RETURN => misc::ret::<S>(machine),
        OpCode::REVERT => misc::revert::<S>(machine),
        OpCode::INVALID => Control::Exit(ExitError::DesignatedInvalid.into()),
        OpCode::SHA3 => system::sha3::<S>(machine),
        OpCode::ADDRESS => system::address::<S>(machine),
        OpCode::BALANCE => system::balance::<H, S>(machine, handler),
        OpCode::SELFBALANCE => system::selfbalance::<H, S>(machine, handler),
        OpCode::BASEFEE => system::basefee::<H, S>(machine, handler),
        OpCode::ORIGIN => system::origin::<H,S>(machine, handler),
        OpCode::CALLER => system::caller::<S>(machine),
        OpCode::CALLVALUE => system::callvalue::<S>(machine),
        OpCode::GASPRICE => system::gasprice::<H,S>(machine, handler),
        OpCode::EXTCODESIZE => system::extcodesize::<H, S>(machine, handler),
        OpCode::EXTCODEHASH => system::extcodehash::<H, S>(machine, handler),
        OpCode::EXTCODECOPY => system::extcodecopy::<H, S>(machine, handler),
        OpCode::RETURNDATASIZE => system::returndatasize::<S>(machine),
        OpCode::RETURNDATACOPY => system::returndatacopy::<S>(machine),
        OpCode::BLOCKHASH => system::blockhash::<H,S>(machine, handler),
        OpCode::COINBASE => system::coinbase::<H,S>(machine, handler),
        OpCode::TIMESTAMP => system::timestamp::<H,S>(machine, handler),
        OpCode::NUMBER => system::number::<H,S>(machine, handler),
        OpCode::DIFFICULTY => system::difficulty::<H,S>(machine, handler),
        OpCode::GASLIMIT => system::gaslimit::<H,S>(machine, handler),
        OpCode::SLOAD => system::sload::<H, S>(machine, handler),
        OpCode::SSTORE => system::sstore::<H, S>(machine, handler),
        OpCode::GAS => system::gas::<S>(machine),
        OpCode::LOG0 => system::log::<H, S>(machine, 0, handler),
        OpCode::LOG1 => system::log::<H, S>(machine, 1, handler),
        OpCode::LOG2 => system::log::<H, S>(machine, 2, handler),
        OpCode::LOG3 => system::log::<H, S>(machine, 3, handler),
        OpCode::LOG4 => system::log::<H, S>(machine, 4, handler),
        OpCode::SELFDESTRUCT => system::selfdestruct::<H, S>(machine, handler),
        OpCode::CREATE => system::create::<H, S>(machine, false, handler), //check
        OpCode::CREATE2 => system::create::<H, S>(machine, true, handler), //check
        OpCode::CALL => system::call::<H, S>(machine, CallScheme::Call, handler), //check
        OpCode::CALLCODE => system::call::<H, S>(machine, CallScheme::CallCode, handler), //check
        OpCode::DELEGATECALL => system::call::<H, S>(machine, CallScheme::DelegateCall, handler), //check
        OpCode::STATICCALL => system::call::<H, S>(machine, CallScheme::StaticCall, handler), //check
        OpCode::CHAINID => system::chainid::<H, S>(machine, handler),
    }
}
