use crate::engine::errors::KernelError;
use crate::engine::*;
use crate::model::{
    TransactionProcessorRunInvocation, WorktopAssertContainsAmountInvocation,
    WorktopAssertContainsInvocation, WorktopAssertContainsNonFungiblesInvocation,
    WorktopDrainInvocation, WorktopPutInvocation, WorktopTakeAllInvocation,
    WorktopTakeAmountInvocation, WorktopTakeNonFungiblesInvocation,
};
use crate::types::*;
use scrypto::resource::AuthZoneDrainInvocation;

// TODO: Cleanup
pub fn parse_and_invoke_native_function<'y, 'a, Y>(
    native_function: NativeFunction,
    args: Vec<u8>,
    system_api: &'y mut Y,
) -> Result<IndexedScryptoValue, RuntimeError>
where
    Y: SystemApi + Invokable<ScryptoInvocation> + InvokableNative<'a>,
{
    match native_function {
        NativeFunction::EpochManager(EpochManagerFunction::Create) => {
            let invocation: EpochManagerCreateInvocation = scrypto_decode(&args)
                .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
            system_api
                .invoke(invocation)
                .map(|a| IndexedScryptoValue::from_typed(&a))
        }
        NativeFunction::ResourceManager(ResourceManagerFunction::BurnBucket) => {
            let invocation: ResourceManagerBurnInvocation = scrypto_decode(&args)
                .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
            system_api
                .invoke(invocation)
                .map(|a| IndexedScryptoValue::from_typed(&a))
        }
        NativeFunction::ResourceManager(ResourceManagerFunction::Create) => {
            let invocation: ResourceManagerCreateInvocation = scrypto_decode(&args)
                .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
            system_api
                .invoke(invocation)
                .map(|a| IndexedScryptoValue::from_typed(&a))
        }
        NativeFunction::TransactionProcessor(TransactionProcessorFunction::Run) => {
            let invocation: TransactionProcessorRunInvocation = scrypto_decode(&args)
                .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
            system_api
                .invoke(invocation)
                .map(|a| IndexedScryptoValue::from_typed(&a))
        }
        NativeFunction::Package(PackageFunction::Publish) => {
            let invocation: PackagePublishInvocation = scrypto_decode(&args)
                .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
            system_api
                .invoke(invocation)
                .map(|a| IndexedScryptoValue::from_typed(&a))
        }
    }
}

// TODO: Cleanup
pub fn parse_and_invoke_native_method<'y, 'a, Y>(
    native_method: NativeMethod,
    args: Vec<u8>,
    system_api: &'y mut Y,
) -> Result<IndexedScryptoValue, RuntimeError>
where
    Y: SystemApi + Invokable<ScryptoInvocation> + InvokableNative<'a>,
{
    match native_method {
        NativeMethod::Bucket(bucket_method) => match bucket_method {
            BucketMethod::Take => {
                let invocation: BucketTakeInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
            BucketMethod::CreateProof => {
                let invocation: BucketCreateProofInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
            BucketMethod::TakeNonFungibles => {
                let invocation: BucketTakeNonFungiblesInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
            BucketMethod::GetNonFungibleIds => {
                let invocation: BucketGetNonFungibleIdsInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
            BucketMethod::GetAmount => {
                let invocation: BucketGetAmountInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
            BucketMethod::Put => {
                let invocation: BucketPutInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
            BucketMethod::GetResourceAddress => {
                let invocation: BucketGetResourceAddressInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
        },
        NativeMethod::AuthZone(auth_zone_method) => match auth_zone_method {
            AuthZoneMethod::Pop => {
                let invocation: AuthZonePopInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
            AuthZoneMethod::Push => {
                let invocation: AuthZonePushInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
            AuthZoneMethod::CreateProof => {
                let invocation: AuthZoneCreateProofInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
            AuthZoneMethod::CreateProofByAmount => {
                let invocation: AuthZoneCreateProofByAmountInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
            AuthZoneMethod::CreateProofByIds => {
                let invocation: AuthZoneCreateProofByIdsInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
            AuthZoneMethod::Clear => {
                let invocation: AuthZoneClearInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
            AuthZoneMethod::Drain => {
                let invocation: AuthZoneDrainInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
        },
        NativeMethod::Proof(proof_method) => match proof_method {
            ProofMethod::GetAmount => {
                let invocation: ProofGetAmountInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
            ProofMethod::GetNonFungibleIds => {
                let invocation: ProofGetNonFungibleIdsInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
            ProofMethod::GetResourceAddress => {
                let invocation: ProofGetResourceAddressInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
            ProofMethod::Clone => {
                let invocation: ProofCloneInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
        },
        NativeMethod::Vault(vault_method) => match vault_method {
            VaultMethod::Take => {
                let invocation: VaultTakeInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
            VaultMethod::Put => {
                let invocation: VaultPutInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
            VaultMethod::LockFee => {
                let invocation: VaultLockFeeInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
            VaultMethod::TakeNonFungibles => {
                let invocation: VaultTakeNonFungiblesInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
            VaultMethod::GetAmount => {
                let invocation: VaultGetAmountInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
            VaultMethod::GetResourceAddress => {
                let invocation: VaultGetResourceAddressInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
            VaultMethod::GetNonFungibleIds => {
                let invocation: VaultGetNonFungibleIdsInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
            VaultMethod::CreateProof => {
                let invocation: VaultCreateProofInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
            VaultMethod::CreateProofByAmount => {
                let invocation: VaultCreateProofByAmountInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
            VaultMethod::CreateProofByIds => {
                let invocation: VaultCreateProofByIdsInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
        },
        NativeMethod::Component(component_method) => match component_method {
            ComponentMethod::AddAccessCheck => {
                let invocation: ComponentAddAccessCheckInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
        },
        NativeMethod::ResourceManager(resman_method) => match resman_method {
            ResourceManagerMethod::Burn => {
                let invocation: ResourceManagerBurnInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
            ResourceManagerMethod::UpdateAuth => {
                let invocation: ResourceManagerUpdateAuthInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
            ResourceManagerMethod::LockAuth => {
                let invocation: ResourceManagerLockAuthInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
            ResourceManagerMethod::CreateVault => {
                let invocation: ResourceManagerCreateVaultInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
            ResourceManagerMethod::CreateBucket => {
                let invocation: ResourceManagerCreateBucketInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
            ResourceManagerMethod::Mint => {
                let invocation: ResourceManagerMintInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
            ResourceManagerMethod::GetMetadata => {
                let invocation: ResourceManagerGetMetadataInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
            ResourceManagerMethod::GetResourceType => {
                let invocation: ResourceManagerGetResourceTypeInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
            ResourceManagerMethod::GetTotalSupply => {
                let invocation: ResourceManagerGetTotalSupplyInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
            ResourceManagerMethod::UpdateMetadata => {
                let invocation: ResourceManagerUpdateMetadataInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
            ResourceManagerMethod::UpdateNonFungibleData => {
                let invocation: ResourceManagerUpdateNonFungibleDataInvocation =
                    scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
            ResourceManagerMethod::NonFungibleExists => {
                let invocation: ResourceManagerNonFungibleExistsInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
            ResourceManagerMethod::GetNonFungible => {
                let invocation: ResourceManagerGetNonFungibleInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
            ResourceManagerMethod::SetResourceAddress => {
                let invocation: ResourceManagerSetResourceAddressInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
        },
        NativeMethod::EpochManager(epoch_manager_method) => match epoch_manager_method {
            EpochManagerMethod::GetCurrentEpoch => {
                let invocation: EpochManagerGetCurrentEpochInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
            EpochManagerMethod::SetEpoch => {
                let invocation: EpochManagerSetEpochInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
        },
        NativeMethod::Worktop(worktop_method) => match worktop_method {
            WorktopMethod::TakeNonFungibles => {
                let invocation: WorktopTakeNonFungiblesInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
            WorktopMethod::Put => {
                let invocation: WorktopPutInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
            WorktopMethod::Drain => {
                let invocation: WorktopDrainInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
            WorktopMethod::AssertContainsNonFungibles => {
                let invocation: WorktopAssertContainsNonFungiblesInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
            WorktopMethod::AssertContains => {
                let invocation: WorktopAssertContainsInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
            WorktopMethod::AssertContainsAmount => {
                let invocation: WorktopAssertContainsAmountInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
            WorktopMethod::TakeAll => {
                let invocation: WorktopTakeAllInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
            WorktopMethod::TakeAmount => {
                let invocation: WorktopTakeAmountInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
        },
    }
}
