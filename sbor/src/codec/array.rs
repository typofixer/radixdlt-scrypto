use crate::rust::mem::MaybeUninit;
use crate::type_id::*;
use crate::*;

impl<X: CustomTypeId, E: Encoder<X>, T: Encode<X, E> + TypeId<X>> Encode<X, E> for [T] {
    #[inline]
    fn encode_type_id(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_type_id(Self::type_id())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_type_id(T::type_id())?;
        encoder.write_size(self.len())?;
        if T::type_id() == SborTypeId::U8 || T::type_id() == SborTypeId::I8 {
            let ptr = self.as_ptr().cast::<u8>();
            let slice = unsafe { sbor::rust::slice::from_raw_parts(ptr, self.len()) };
            encoder.write_slice(slice)?;
        } else {
            for v in self {
                encoder.encode_deeper_body(v)?;
            }
        }
        Ok(())
    }
}

impl<X: CustomTypeId, E: Encoder<X>, T: Encode<X, E> + TypeId<X>, const N: usize> Encode<X, E>
    for [T; N]
{
    #[inline]
    fn encode_type_id(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_type_id(Self::type_id())
    }
    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.as_slice().encode_body(encoder)
    }
}

impl<X: CustomTypeId, D: Decoder<X>, T: Decode<X, D> + TypeId<X>, const N: usize> Decode<X, D>
    for [T; N]
{
    #[inline]
    fn decode_body_with_type_id(
        decoder: &mut D,
        type_id: SborTypeId<X>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_type_id(type_id, Self::type_id())?;
        let element_type_id = decoder.read_and_check_type_id(T::type_id())?;
        decoder.read_and_check_size(N)?;

        // Please read:
        // * https://doc.rust-lang.org/stable/std/mem/union.MaybeUninit.html#initializing-an-array-element-by-element
        // * https://github.com/rust-lang/rust/issues/61956
        //
        // TODO: replace with `uninit_array` and `assume_array_init` once they're stable

        // Create an uninitialized array
        let mut data: [MaybeUninit<T>; N] = unsafe { MaybeUninit::uninit().assume_init() };

        // Decode element by element
        for elem in &mut data[..] {
            elem.write(decoder.decode_deeper_body_with_type_id(element_type_id)?);
        }

        // Use &mut as an assertion of unique "ownership"
        let ptr = &mut data as *mut _ as *mut [T; N];
        let res = unsafe { ptr.read() };
        core::mem::forget(data);

        Ok(res)
    }
}

#[cfg(feature = "schema")]
pub use schema::*;

#[cfg(feature = "schema")]
mod schema {
    use super::*;

    impl<C: CustomTypeKind<GlobalTypeId>, T: NewDescribe<C> + TypeId<C::CustomTypeId>>
        NewDescribe<C> for [T]
    {
        const SCHEMA_TYPE_REF: GlobalTypeId = if T::IS_U8 {
            GlobalTypeId::well_known(well_known_basic_types::BYTES_ID)
        } else {
            GlobalTypeId::complex("Array", &[T::SCHEMA_TYPE_REF])
        };

        fn get_local_type_data() -> Option<TypeData<C, GlobalTypeId>> {
            if T::IS_U8 {
                None
            } else {
                Some(TypeData {
                    kind: TypeKind::Array {
                        element_sbor_type_id: T::type_id().as_u8(),
                        element_type: T::SCHEMA_TYPE_REF,
                        length_validation: LengthValidation::none(),
                    },
                    metadata: TypeMetadata::named_no_child_names("Array"),
                })
            }
        }

        fn add_all_dependencies(aggregator: &mut TypeAggregator<C>) {
            aggregator.add_child_type_and_descendents::<T>();
        }
    }

    #[cfg(feature = "schema")]
    impl<
            C: CustomTypeKind<GlobalTypeId>,
            T: NewDescribe<C> + TypeId<C::CustomTypeId>,
            const N: usize,
        > NewDescribe<C> for [T; N]
    {
        const SCHEMA_TYPE_REF: GlobalTypeId =
            GlobalTypeId::complex_sized("Array", &[T::SCHEMA_TYPE_REF], N);

        fn get_local_type_data() -> Option<TypeData<C, GlobalTypeId>> {
            let size = N
                .try_into()
                .expect("The array length is too large for a u32 for the SBOR schema");
            let type_name = if T::type_id() == SborTypeId::U8 {
                "Bytes"
            } else {
                "Array"
            };
            Some(TypeData {
                kind: TypeKind::Array {
                    element_sbor_type_id: T::type_id().as_u8(),
                    element_type: T::SCHEMA_TYPE_REF,
                    length_validation: LengthValidation {
                        min: Some(size),
                        max: Some(size),
                    },
                },
                metadata: TypeMetadata::named_no_child_names(type_name),
            })
        }

        fn add_all_dependencies(aggregator: &mut TypeAggregator<C>) {
            aggregator.add_child_type_and_descendents::<T>();
        }
    }
}
