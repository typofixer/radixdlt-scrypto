use crate::engine::*;
use crate::types::*;
use crate::wasm::WasmEngine;
use radix_engine_interface::api::api::EngineApi;
use radix_engine_interface::api::types::*;
use radix_engine_interface::crypto::hash;

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
pub enum TransactionHashError {
    OutOfUUid,
}

impl<W: WasmEngine> ExecutableInvocation<W> for TransactionHashGetInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: ResolverApi<W>>(
        self,
        _deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::TransactionHash(TransactionHashMethod::Get)),
            ResolvedReceiver::new(RENodeId::TransactionHash(self.receiver)),
        );
        let call_frame_update = CallFrameUpdate::empty();
        let executor = NativeExecutor(self);

        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for TransactionHashGetInvocation {
    type Output = Hash;

    fn main<Y>(self, api: &mut Y) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError>,
    {
        let offset = SubstateOffset::TransactionHash(TransactionHashOffset::TransactionHash);
        let node_id = RENodeId::TransactionHash(self.receiver);
        let handle = api.lock_substate(node_id, offset, LockFlags::read_only())?;
        let substate = api.get_ref(handle)?;
        let transaction_hash_substate = substate.transaction_hash();
        Ok((
            transaction_hash_substate.hash.clone(),
            CallFrameUpdate::empty(),
        ))
    }
}

impl<W: WasmEngine> ExecutableInvocation<W> for TransactionHashGenerateUuidInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: ResolverApi<W>>(
        self,
        _deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::TransactionHash(
                TransactionHashMethod::GenerateUuid,
            )),
            ResolvedReceiver::new(RENodeId::TransactionHash(self.receiver)),
        );
        let call_frame_update = CallFrameUpdate::empty();
        let executor = NativeExecutor(self);

        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for TransactionHashGenerateUuidInvocation {
    type Output = u128;

    fn main<Y>(self, api: &mut Y) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError>,
    {
        let offset = SubstateOffset::TransactionHash(TransactionHashOffset::TransactionHash);
        let node_id = RENodeId::TransactionHash(self.receiver);
        let handle = api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;
        let mut substate_mut = api.get_ref_mut(handle)?;
        let transaction_hash_substate = substate_mut.transaction_hash();

        if transaction_hash_substate.next_id == u32::MAX {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::TransactionHashError(TransactionHashError::OutOfUUid),
            ));
        }

        let mut data = transaction_hash_substate.hash.to_vec();
        data.extend(transaction_hash_substate.next_id.to_le_bytes());
        let uuid = u128::from_le_bytes(hash(data).lower_16_bytes()); // TODO: Remove hash

        transaction_hash_substate.next_id = transaction_hash_substate.next_id + 1;

        Ok((uuid, CallFrameUpdate::empty()))
    }
}
