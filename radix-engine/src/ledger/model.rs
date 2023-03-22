use crate::types::*;
use utils::copy_u8_array;

/// The unique identifier of a (stored) node.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Sbor)]
#[sbor(transparent)]
pub struct NodeId([u8; 31]);

impl NodeId {
    pub const LENGTH: usize = 31;

    pub fn new(entity_byte: u8, random_bytes: &[u8; 26], index: u32) -> Self {
        let mut buf = [0u8; Self::LENGTH];
        buf[0] = entity_byte;
        buf[1..random_bytes.len() + 1].copy_from_slice(random_bytes);
        buf[random_bytes.len() + 2..].copy_from_slice(&index.to_be_bytes());
        Self(buf)
    }
}

/// The unique identifier of a node module.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Sbor)]
#[sbor(transparent)]
pub struct ModuleId(u8);

/// The unique identifier of a substate within a node module.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Sbor)]
pub enum SubstateKey {
    Static(u8),
    Dynamic(DynamicSubstateKey),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Sbor)]
#[sbor(transparent)]
pub struct DynamicSubstateKey(Vec<u8>);

impl DynamicSubstateKey {
    pub const MAX_LENGTH: usize = 128;

    fn from_slice(slice: &[u8]) -> Option<Self> {
        Self::from_bytes(slice.to_vec())
    }

    fn from_bytes(bytes: Vec<u8>) -> Option<Self> {
        // TODO: do we want to enforce more constraints on the bytes?
        if bytes.len() > Self::MAX_LENGTH {
            None
        } else {
            Some(Self(bytes))
        }
    }
}

impl AsRef<[u8]> for DynamicSubstateKey {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Into<Vec<u8>> for DynamicSubstateKey {
    fn into(self) -> Vec<u8> {
        self.0
    }
}

pub fn encode_substate_id(
    node_id: &NodeId,
    module_id: &ModuleId,
    substate_key: &SubstateKey,
) -> Vec<u8> {
    let mut buffer = Vec::new();
    buffer.extend(&node_id.0);
    buffer.push(module_id.0);
    match substate_key {
        SubstateKey::Static(offset) => {
            buffer.push(0);
            buffer.push(*offset);
        }
        SubstateKey::Dynamic(dynamic) => {
            buffer.push(1);
            buffer.extend(dynamic.as_ref()); // Length is marked by EOF
        }
    }
    buffer
}

pub fn decode_substate_id(slice: &[u8]) -> (NodeId, ModuleId, SubstateKey) {
    let node_id = NodeId(copy_u8_array(&slice[0..NodeId::LENGTH]));
    let module_id = ModuleId(slice[NodeId::LENGTH]);
    let substate_key = match slice[NodeId::LENGTH + 1] {
        0 => SubstateKey::Static(slice[NodeId::LENGTH + 2]),
        1 => SubstateKey::Dynamic(
            DynamicSubstateKey::from_slice(&slice[NodeId::LENGTH + 2..])
                .expect("Invalid dynamic substate key"),
        ),
        i => panic!("Unexpected substate key type: {}", i),
    };

    (node_id, module_id, substate_key)
}

pub fn encode_substate_value(value: &IndexedScryptoValue) -> Vec<u8> {
    value.as_slice().to_vec()
}

pub fn decode_substate_value(slice: &[u8]) -> IndexedScryptoValue {
    IndexedScryptoValue::from_slice(slice).expect("Failed to decode substate value")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_substate_id() {
        let node_id = NodeId([1u8; NodeId::LENGTH]);
        let module_id = ModuleId(2);
        let substate_key = SubstateKey::Dynamic(DynamicSubstateKey::from_bytes(vec![3]).unwrap());
        let substate_id = encode_substate_id(&node_id, &module_id, &substate_key);
        assert_eq!(
            substate_id,
            vec![
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, // node id
                2, // module id
                1, 3, // substate key
            ]
        );
        assert_eq!(
            decode_substate_id(&substate_id),
            (node_id, module_id, substate_key)
        )
    }

    #[test]
    fn test_encode_decode_substate_value() {
        let value = IndexedScryptoValue::from_typed("Hello");
        let substate_value = encode_substate_value(&value);
        assert_eq!(
            substate_value,
            vec![
                92, // prefix
                12, // string
                5,  // length
                72, 101, 108, 108, 111 // "Hello"
            ]
        );
        assert_eq!(decode_substate_value(&substate_value), value)
    }
}
