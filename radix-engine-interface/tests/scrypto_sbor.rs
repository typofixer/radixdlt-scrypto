use radix_engine_interface::math::*;
use radix_engine_interface::*;

#[derive(ScryptoSbor, LegacyDescribe, NonFungibleData)]
pub struct TestStruct {
    pub a: u32,
    #[legacy_skip]
    #[sbor(skip)]
    pub b: String,
    pub c: Decimal,
}

#[derive(ScryptoSbor, LegacyDescribe)]
pub enum TestEnum {
    A { named: String },
    B(u32, u8),
    C(Decimal),
}
