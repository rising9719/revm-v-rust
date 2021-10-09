use super::constants::*;
use crate::spec::Spec;
use crate::{error::ExitError, evm::SelfDestructResult};
use primitive_types::{H256, U256};

pub fn call_extra_check<SPEC: Spec>(gas: U256, after_gas: u64) -> Result<(), ExitError> {
    if SPEC::ERR_ON_CALL_WITH_MORE_GAS && U256::from(after_gas) < gas {
        Err(ExitError::OutOfGas)
    } else {
        Ok(())
    }
}

#[allow(clippy::collapsible_else_if)]
pub fn sstore_refund<SPEC: Spec>(original: H256, current: H256, new: H256) -> i64 {
    if SPEC::SSTORE_GAS_METERING {
        if current == new {
            0
        } else {
            if original == current && new == H256::default() {
                SPEC::REFUND_SSTORE_CLEARS
            } else {
                let mut refund = 0;

                if original != H256::default() {
                    if current == H256::default() {
                        refund -= SPEC::REFUND_SSTORE_CLEARS;
                    } else if new == H256::default() {
                        refund += SPEC::REFUND_SSTORE_CLEARS;
                    }
                }

                if original == new {
                    let (gas_sstore_reset, gas_sload) = if SPEC::INCREASE_STATE_ACCESS_GAS {
                        (
                            SPEC::GAS_SSTORE_RESET - SPEC::GAS_SLOAD_COLD,
                            SPEC::GAS_STORAGE_READ_WARM,
                        )
                    } else {
                        (SPEC::GAS_SSTORE_RESET, SPEC::GAS_SLOAD)
                    };
                    if original == H256::default() {
                        refund += (SPEC::GAS_SSTORE_SET - gas_sload) as i64;
                    } else {
                        refund += (gas_sstore_reset - gas_sload) as i64;
                    }
                }

                refund
            }
        }
    } else {
        if current != H256::default() && new == H256::default() {
            SPEC::REFUND_SSTORE_CLEARS
        } else {
            0
        }
    }
}

pub fn create2_cost(len: U256) -> Option<u64> {
    let base = U256::from(CREATE);
    // ceil(len / 32.0)
    let sha_addup_base = len / U256::from(32)
        + if len % U256::from(32) == U256::zero() {
            U256::zero()
        } else {
            U256::one()
        };
    let sha_addup = U256::from(SHA3WORD).checked_mul(sha_addup_base)?;
    let gas = base.checked_add(sha_addup)?;

    if gas > U256::from(u64::MAX) {
        return None;
    }

    Some(gas.as_u64())
}

pub fn exp_cost<SPEC: Spec>(power: U256) -> Option<u64> {
    if power == U256::zero() {
        Some(EXP)
    } else {
        let gas = U256::from(EXP).checked_add(
            U256::from(SPEC::GAS_EXPBYTE)
                .checked_mul(U256::from(super::utils::log2floor(power) / 8 + 1))?,
        )?;

        if gas > U256::from(u64::MAX) {
            return None;
        }

        Some(gas.as_u64())
    }
}

pub fn verylowcopy_cost(len: U256) -> Option<u64> {
    let wordd = len / U256::from(32);
    let wordr = len % U256::from(32);

    let gas = U256::from(VERYLOW).checked_add(U256::from(COPY).checked_mul(
        if wordr == U256::zero() {
            wordd
        } else {
            wordd + U256::one()
        },
    )?)?;

    if gas > U256::from(u64::MAX) {
        return None;
    }

    Some(gas.as_u64())
}

pub fn extcodecopy_cost<SPEC: Spec>(len: U256, is_cold: bool) -> Option<u64> {
    let wordd = len / U256::from(32);
    let wordr = len % U256::from(32);
    let gas = U256::from(hot_cold_cost::<SPEC>(is_cold, SPEC::GAS_EXT_CODE)).checked_add(
        U256::from(COPY).checked_mul(if wordr == U256::zero() {
            wordd
        } else {
            wordd + U256::one()
        })?,
    )?;

    if gas > U256::from(u64::MAX) {
        return None;
    }

    Some(gas.as_u64())
}

pub fn log_cost(n: u8, len: U256) -> Option<u64> {
    let gas = U256::from(LOG)
        .checked_add(U256::from(LOGDATA).checked_mul(len)?)?
        .checked_add(U256::from(LOGTOPIC * n as u64))?;

    if gas > U256::from(u64::MAX) {
        return None;
    }

    Some(gas.as_u64())
}

pub fn sha3_cost(len: U256) -> Option<u64> {
    let wordd = len / U256::from(32);
    let wordr = len % U256::from(32);

    let gas = U256::from(SHA3).checked_add(U256::from(SHA3WORD).checked_mul(
        if wordr == U256::zero() {
            wordd
        } else {
            wordd + U256::one()
        },
    )?)?;

    if gas > U256::from(u64::MAX) {
        return None;
    }

    Some(gas.as_u64())
}

pub fn sload_cost<SPEC: Spec>(is_cold: bool) -> u64 {
    if SPEC::INCREASE_STATE_ACCESS_GAS {
        if is_cold {
            SPEC::GAS_SLOAD_COLD
        } else {
            SPEC::GAS_STORAGE_READ_WARM
        }
    } else {
        SPEC::GAS_SLOAD
    }
}

#[allow(clippy::collapsible_else_if)]
pub fn sstore_cost<SPEC: Spec>(
    original: H256,
    current: H256,
    new: H256,
    gas: u64,
    is_cold: bool,
) -> Option<u64> {
    let (gas_sload, gas_sstore_reset) = if SPEC::INCREASE_STATE_ACCESS_GAS {
        (
            SPEC::GAS_STORAGE_READ_WARM,
            SPEC::GAS_SSTORE_RESET - SPEC::GAS_SLOAD_COLD,
        )
    } else {
        (SPEC::GAS_SLOAD, SPEC::GAS_SSTORE_RESET)
    };
    let gas_cost = if SPEC::SSTORE_GAS_METERING {
        if SPEC::SSTORE_REVERT_UNDER_STIPEND && gas <= SPEC::CALL_STIPEND {
            return None;
        }

        if new == current {
            gas_sload
        } else {
            if original == current {
                if original == H256::zero() {
                    SPEC::GAS_SSTORE_SET
                } else {
                    gas_sstore_reset
                }
            } else {
                gas_sload
            }
        }
    } else {
        if current == H256::zero() && new != H256::zero() {
            SPEC::GAS_SSTORE_SET
        } else {
            gas_sstore_reset
        }
    };
    // In EIP-2929 we charge extra if the slot has not been used yet in this transaction
    if is_cold {
        Some(gas_cost + SPEC::GAS_SLOAD_COLD)
    } else {
        Some(gas_cost)
    }
}

pub fn selfdestruct_cost<SPEC: Spec>(res: SelfDestructResult) -> u64 {
    let eip161 = !SPEC::EMPTY_CONSIDERED_EXISTS;
    let should_charge_topup = if eip161 {
        res.had_value && !res.exists
    } else {
        !res.exists
    };

    let selfdestruct_gas_topup = if should_charge_topup {
        SPEC::GAS_SELFDESTRUCT_NEW_ACCOUNT
    } else {
        0
    };

    let mut gas = SPEC::GAS_SELFDESTRUCT + selfdestruct_gas_topup;
    if SPEC::INCREASE_STATE_ACCESS_GAS && res.is_cold {
        gas += SPEC::GAS_ACCOUNT_ACCESS_COLD
    }
    gas
}

pub fn call_cost<SPEC: Spec>(
    value: U256,
    is_new: bool,
    is_cold: bool,
    is_call_or_callcode: bool,
    is_call_or_staticcall: bool,
) -> u64 {
    let transfers_value = value != U256::default();
    hot_cold_cost::<SPEC>(is_cold, SPEC::GAS_CALL)
        + xfer_cost(is_call_or_callcode, transfers_value)
        + new_cost::<SPEC>(
            is_call_or_staticcall,
            is_new,
            transfers_value,
        )
}

#[inline(always)]
pub fn hot_cold_cost<SPEC: Spec>(is_cold: bool, regular_value: u64) -> u64 {
    if SPEC::INCREASE_STATE_ACCESS_GAS {
        if is_cold {
            SPEC::GAS_ACCOUNT_ACCESS_COLD
        } else {
            SPEC::GAS_STORAGE_READ_WARM
        }
    } else {
        regular_value
    }
}

fn xfer_cost(is_call_or_callcode: bool, transfers_value: bool) -> u64 {
    if is_call_or_callcode && transfers_value {
        CALLVALUE
    } else {
        0
    }
}

fn new_cost<SPEC: Spec>(
    is_call_or_staticcall: bool,
    is_new: bool,
    transfers_value: bool,
) -> u64 {
    if is_call_or_staticcall {
        if !SPEC::EMPTY_CONSIDERED_EXISTS {
            if transfers_value && is_new {
                NEWACCOUNT
            } else {
                0
            }
        } else if is_new {
            NEWACCOUNT
        } else {
            0
        }
    } else {
        0
    }
}

pub fn memory_gas(a: usize) -> Result<u64, ExitError> {
    let a = a as u64;
    MEMORY
        .checked_mul(a)
        .ok_or(ExitError::OutOfGas)?
        .checked_add(a.checked_mul(a).ok_or(ExitError::OutOfGas)? / 512)
        .ok_or(ExitError::OutOfGas)
}
