use super::gas;
use crate::{machine::Machine, util, Return, Spec, SpecId::*};
use primitive_types::{H256, U256};

#[inline(always)]
pub fn codesize(machine: &mut Machine) -> Return {
    //gas!(machine, gas::BASE);
    let size = U256::from(machine.contract.code_size);
    push!(machine, size);
    Return::Continue
}

#[inline(always)]
pub fn codecopy(machine: &mut Machine) -> Return {
    pop!(machine, memory_offset, code_offset, len);
    gas_or_fail!(machine, gas::verylowcopy_cost(len));
    let len = as_usize_or_fail!(len, Return::OutOfGas);
    if len == 0 {
        return Return::Continue;
    }
    let memory_offset = as_usize_or_fail!(memory_offset, Return::OutOfGas);
    let code_offset = as_usize_saturated!(code_offset);
    memory_resize!(machine, memory_offset, len);

    machine
        .memory
        .set_data(memory_offset, code_offset, len, &machine.contract.code);

    Return::Continue
}

#[inline(always)]
pub fn calldataload(machine: &mut Machine) -> Return {
    //gas!(machine, gas::VERYLOW);

    pop!(machine, index);

    let mut load = [0u8; 32];
    #[allow(clippy::needless_range_loop)]
    for i in 0..32 {
        if let Some(p) = index.checked_add(U256::from(i)) {
            if p <= U256::from(usize::MAX) {
                let p = p.as_usize();
                if p < machine.contract.input.len() {
                    load[i] = machine.contract.input[p];
                }
            }
        }
    }

    push_h256!(machine, H256::from(load));
    Return::Continue
}

#[inline(always)]
pub fn calldatasize(machine: &mut Machine) -> Return {
    //gas!(machine, gas::BASE);

    let len = U256::from(machine.contract.input.len());
    push!(machine, len);
    Return::Continue
}

#[inline(always)]
pub fn calldatacopy(machine: &mut Machine) -> Return {
    pop!(machine, memory_offset, data_offset, len);
    gas_or_fail!(machine, gas::verylowcopy_cost(len));
    let len = as_usize_or_fail!(len, Return::OutOfGas);
    if len == 0 {
        return Return::Continue;
    }
    let memory_offset = as_usize_or_fail!(memory_offset, Return::OutOfGas);
    let data_offset = as_usize_saturated!(data_offset);
    memory_resize!(machine, memory_offset, len);

    machine
        .memory
        .set_data(memory_offset, data_offset, len, &machine.contract.input);
    Return::Continue
}

#[inline(always)]
pub fn pop(machine: &mut Machine) -> Return {
    //gas!(machine, gas::BASE);
    pop!(machine, _val);
    Return::Continue
}

#[inline(always)]
pub fn mload(machine: &mut Machine) -> Return {
    //gas!(machine, gas::VERYLOW);
    pop!(machine, index);

    let index = as_usize_or_fail!(index, Return::OutOfGas);
    memory_resize!(machine, index, 32);
    push!(
        machine,
        util::be_to_u256(machine.memory.get_slice(index, 32))
    );
    Return::Continue
}

#[inline(always)]
pub fn mstore(machine: &mut Machine) -> Return {
    //gas!(machine, gas::VERYLOW);

    pop!(machine, index);
    pop!(machine, value);

    let index = as_usize_or_fail!(index, Return::OutOfGas);
    memory_resize!(machine, index, 32);
    machine.memory.set_u256(index, value);
    Return::Continue
}

#[inline(always)]
pub fn mstore8(machine: &mut Machine) -> Return {
    //gas!(machine, gas::VERYLOW);

    pop!(machine, index, value);

    let index = as_usize_or_fail!(index, Return::OutOfGas);
    memory_resize!(machine, index, 1);
    let value = (value.low_u32() & 0xff) as u8;
    // SAFETY: we resized our memory two lines above.
    unsafe { machine.memory.set_byte(index, value) }
    Return::Continue
}

#[inline(always)]
pub fn jump(machine: &mut Machine) -> Return {
    //gas!(machine, gas::MID);

    pop!(machine, dest);
    let dest = as_usize_or_fail!(dest, Return::InvalidJump);

    if machine.contract.is_valid_jump(dest) {
        machine.program_counter = unsafe {machine.contract.code.as_ptr().offset(dest as isize)};
        Return::Continue
    } else {
        Return::InvalidJump
    }
}

#[inline(always)]
pub fn jumpi(machine: &mut Machine) -> Return {
    //gas!(machine, gas::HIGH);

    pop!(machine, dest, value);

    if !value.is_zero() {
        let dest = as_usize_or_fail!(dest, Return::InvalidJump);
        if machine.contract.is_valid_jump(dest) {
            machine.program_counter = unsafe {machine.contract.code.as_ptr().offset(dest as isize)};
            Return::Continue
        } else {
            Return::InvalidJump
        }
    } else {
        machine.add_next_gas_block()
    }
}

#[inline(always)]
pub fn jumpdest(machine: &mut Machine) -> Return {
    gas!(machine, gas::JUMPDEST);
    machine.add_next_gas_block()
}

#[inline(always)]
pub fn pc(machine: &mut Machine) -> Return {
    //gas!(machine, gas::BASE);
    push!(machine, U256::from(machine.program_counter()-1));
    Return::Continue
}

#[inline(always)]
pub fn msize(machine: &mut Machine) -> Return {
    //gas!(machine, gas::BASE);
    push!(machine, U256::from(machine.memory.effective_len()));
    Return::Continue
}

// code padding is needed for contracts
#[inline(always)]
pub fn push<const N: usize>(machine: &mut Machine) -> Return {
    //gas!(machine, gas::VERYLOW);

    let start = machine.program_counter;
    machine.program_counter = unsafe {machine.program_counter.offset(N as isize)};
    machine.stack.push_slice::<N>(unsafe {core::slice::from_raw_parts(start,N)})
}

#[inline(always)]
pub fn dup<const N: usize>(machine: &mut Machine) -> Return {
    gas!(machine, gas::VERYLOW);

    machine.stack.dup::<N>()
}

#[inline(always)]
pub fn swap<const N: usize>(machine: &mut Machine) -> Return {
    gas!(machine, gas::VERYLOW);
    machine.stack.swap::<N>()
}

#[inline(always)]
pub fn ret(machine: &mut Machine) -> Return {
    // zero gas cost gas!(machine,gas::ZERO);
    pop!(machine, start, len);
    let len = as_usize_or_fail!(len, Return::OutOfGas);
    if len == 0 {
        machine.return_range = usize::MAX..usize::MAX;
    } else {
        let offset = as_usize_or_fail!(start, Return::OutOfGas);
        memory_resize!(machine, offset, len);
        machine.return_range = offset..(offset + len);
    }
    Return::Return
}

#[inline(always)]
pub fn revert<SPEC: Spec>(machine: &mut Machine) -> Return {
    check!(SPEC::enabled(BYZANTINE)); // EIP-140: REVERT instruction
                                      // zero gas cost gas!(machine,gas::ZERO);
    pop!(machine, start, len);
    let len = as_usize_or_fail!(len, Return::OutOfGas);
    if len == 0 {
        machine.return_range = usize::MAX..usize::MAX;
    } else {
        let offset = as_usize_or_fail!(start, Return::OutOfGas);
        memory_resize!(machine, offset, len);
        machine.return_range = offset..(offset + len);
    }
    Return::Revert
}
