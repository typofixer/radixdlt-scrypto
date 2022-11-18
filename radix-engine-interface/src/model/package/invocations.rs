use crate::api::api::{ScryptoNativeInvocation, SysInvocation};

use crate::api::wasm_input::{
    NativeFnInvocation, NativeFunctionInvocation, PackageFunctionInvocation,
};
use crate::crypto::Blob;
use crate::model::*;
use crate::scrypto;

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct PackagePublishInvocation {
    pub code: Blob,
    pub abi: Blob,
}

impl SysInvocation for PackagePublishInvocation {
    type Output = PackageAddress;
}

impl ScryptoNativeInvocation for PackagePublishInvocation {}

impl Into<NativeFnInvocation> for PackagePublishInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Function(NativeFunctionInvocation::Package(
            PackageFunctionInvocation::Publish(self),
        ))
    }
}