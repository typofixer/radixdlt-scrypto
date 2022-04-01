use crate::resource::*;
use crate::rust::vec;
use crate::rust::vec::Vec;
use sbor::*;
use scrypto::math::Decimal;

/// TODO: add documentation for public types once they're stable.

#[derive(Debug, Clone, PartialEq, Eq, Hash, Describe, TypeId, Encode, Decode)]
pub enum SoftResource {
    Static(ResourceAddress),
    Dynamic(SchemaPath),
}

impl From<ResourceAddress> for SoftResource {
    fn from(resource_address: ResourceAddress) -> Self {
        SoftResource::Static(resource_address)
    }
}

impl From<SchemaPath> for SoftResource {
    fn from(path: SchemaPath) -> Self {
        SoftResource::Dynamic(path)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Describe, TypeId, Encode, Decode)]
pub enum SoftResourceOrNonFungible {
    StaticNonFungible(NonFungibleAddress),
    StaticResource(ResourceAddress),
    Dynamic(SchemaPath),
}

impl From<NonFungibleAddress> for SoftResourceOrNonFungible {
    fn from(non_fungible_address: NonFungibleAddress) -> Self {
        SoftResourceOrNonFungible::StaticNonFungible(non_fungible_address)
    }
}

impl From<ResourceAddress> for SoftResourceOrNonFungible {
    fn from(resource_address: ResourceAddress) -> Self {
        SoftResourceOrNonFungible::StaticResource(resource_address)
    }
}

impl From<SchemaPath> for SoftResourceOrNonFungible {
    fn from(path: SchemaPath) -> Self {
        SoftResourceOrNonFungible::Dynamic(path)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Describe, TypeId, Encode, Decode)]
pub enum SoftResourceOrNonFungibleList {
    Static(Vec<SoftResourceOrNonFungible>),
    Dynamic(SchemaPath),
}

impl From<SchemaPath> for SoftResourceOrNonFungibleList {
    fn from(path: SchemaPath) -> Self {
        SoftResourceOrNonFungibleList::Dynamic(path)
    }
}

/// Authorization Rule
#[derive(Debug, Clone, PartialEq, Eq, Hash, Describe, TypeId, Encode, Decode)]
pub enum ProofRule {
    This(SoftResourceOrNonFungible),
    AmountOf(Decimal, SoftResource),
    CountOf(u8, SoftResourceOrNonFungibleList),
    AllOf(SoftResourceOrNonFungibleList),
    AnyOf(SoftResourceOrNonFungibleList),
}

impl From<NonFungibleAddress> for ProofRule {
    fn from(non_fungible_address: NonFungibleAddress) -> Self {
        ProofRule::This(non_fungible_address.into())
    }
}

impl From<ResourceAddress> for ProofRule {
    fn from(resource_address: ResourceAddress) -> Self {
        ProofRule::This(resource_address.into())
    }
}

#[macro_export]
macro_rules! resource_list {
  ($($resource: expr),*) => ({
      let mut list: Vec<::scrypto::resource::SoftResourceOrNonFungible> = Vec::new();
      $(
        list.push($resource.into());
      )*
      ::scrypto::resource::SoftResourceOrNonFungibleList::Static(list)
  });
}

#[macro_export]
macro_rules! this {
    ($resource:expr) => {{
        ::scrypto::resource::ProofRule::This($resource.into())
    }};
}

#[macro_export]
macro_rules! any_of {
    ($list:expr) => ({
        ::scrypto::resource::ProofRule::AnyOf($list.into())
    });
    ($left:expr, $($right:expr),+) => ({
        ::scrypto::resource::ProofRule::AnyOf(resource_list!($left, $($right),+))
    });
}

#[macro_export]
macro_rules! all_of {
    ($list:expr) => ({
        ::scrypto::resource::ProofRule::AllOf($list.into())
    });
    ($left:expr, $($right:expr),+) => ({
        ::scrypto::resource::ProofRule::AllOf(resource_list!($left, $($right),+))
    });
}

#[macro_export]
macro_rules! min_n_of {
    ($count:expr, $list:expr) => ({
        ::scrypto::resource::ProofRule::CountOf($count, $list.into())
    });
    ($count:expr, $left:expr, $($right:expr),+) => ({
        ::scrypto::resource::ProofRule::CountOf($count, resource_list!($left, $($right),+))
    });
}

#[macro_export]
macro_rules! min_amount_of {
    ($amount:expr, $resource:expr) => {
        ProofRule::AmountOf($amount, $resource.into())
    };
}
