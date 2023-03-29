use crate::blueprints::transaction_processor::TransactionProcessorError;
use crate::errors::*;
use crate::system::kernel_modules::costing::FinalizingFeeReserve;
use crate::system::kernel_modules::costing::{CostingError, FeeReserveError};
use crate::system::kernel_modules::costing::{FeeSummary, SystemLoanFeeReserve};
use crate::system::node_modules::type_info::TypeInfoSubstate;
use crate::system::node_substates::{
    PersistedSubstate, RuntimeSubstate, SubstateRef, SubstateRefMut,
};
use crate::transaction::BalanceChange;
use crate::transaction::RejectResult;
use crate::transaction::StateUpdateSummary;
use crate::transaction::TransactionOutcome;
use crate::transaction::TransactionResult;
use crate::transaction::{AbortReason, AbortResult, CommitResult};
use crate::types::*;
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::blueprints::resource::LiquidFungibleResource;
use radix_engine_interface::blueprints::resource::VAULT_BLUEPRINT;
use radix_engine_interface::blueprints::transaction_processor::InstructionOutput;
use radix_engine_interface::crypto::hash;
use radix_engine_interface::types::Level;
use radix_engine_interface::types::*;
use radix_engine_stores::interface::{
    AcquireLockError, StateDependencies, StateUpdates, SubstateDatabase, SubstateStore,
};
use sbor::rust::collections::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Sbor)]
pub enum LockState {
    Read(usize),
    Write,
}

impl LockState {
    pub fn no_lock() -> Self {
        Self::Read(0)
    }
}

#[derive(Debug)]
pub enum ExistingMetaState {
    Loaded,
    Updated(Option<PersistedSubstate>),
}

#[derive(Debug)]
pub enum SubstateMetaState {
    New,
    Existing {
        old_version: u32,
        state: ExistingMetaState,
    },
}

#[derive(Debug)]
pub struct LoadedSubstate {
    substate: RuntimeSubstate,
    lock_state: LockState,
    metastate: SubstateMetaState,
}

/// Transaction-wide states and side effects
pub struct Track<'s> {
    substate_db: &'s dyn SubstateDatabase,
    loaded_substates: IndexMap<(NodeId, ModuleId, SubstateKey), LoadedSubstate>,
}

impl<'s> SubstateStore for Track<'s> {
    fn acquire_lock(
        &mut self,
        node_id: &NodeId,
        module_id: ModuleId,
        substate_key: &SubstateKey,
        flags: LockFlags,
    ) -> Result<u32, AcquireLockError> {
        todo!()
    }

    fn release_lock(&mut self, handle: u32) {
        todo!()
    }

    fn get_substate(&self, handle: u32) -> &IndexedScryptoValue {
        todo!()
    }

    fn put_substate(&mut self, handle: u32, substate_value: IndexedScryptoValue) {
        todo!()
    }

    fn insert_substate(
        &mut self,
        node_id: NodeId,
        module_id: ModuleId,
        substate_key: SubstateKey,
        substate_value: IndexedScryptoValue,
    ) {
        todo!()
    }

    fn list_substates(
        &mut self,
        node_id: &NodeId,
        module_id: ModuleId,
    ) -> Box<dyn Iterator<Item = (SubstateKey, IndexedScryptoValue)>> {
        todo!()
    }

    fn revert_non_force_write_changes(&mut self) {
        todo!()
    }

    fn finalize(self) -> (StateUpdates, StateDependencies) {
        todo!()
    }
}
