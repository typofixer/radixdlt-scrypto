use super::*;
use crate::internal_prelude::*;

//=================================================================================
// NOTE:
// See versioned.rs for tests and a demonstration for the calculation of hashes etc
//=================================================================================

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))] // For toolkit
#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
pub struct SignedIntentV1 {
    pub intent: IntentV1,
    pub intent_signatures: IntentSignaturesV1,
}

impl TransactionPayloadEncode for SignedIntentV1 {
    type EncodablePayload<'a> =
        SborEnumVariant<{ TransactionDiscriminator::V1SignedIntent as u8 }, &'a Self>;

    type Prepared = PreparedSignedIntentV1;

    fn as_payload<'a>(&'a self) -> Self::EncodablePayload<'a> {
        SborEnumVariant::new(self)
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))] // For toolkit
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PreparedSignedIntentV1 {
    pub intent: PreparedIntentV1,
    pub intent_signatures: PreparedIntentSignaturesV1,
    pub summary: Summary,
}

impl HasSummary for PreparedSignedIntentV1 {
    fn get_summary(&self) -> &Summary {
        &self.summary
    }
}

impl TransactionFullChildPreparable for PreparedSignedIntentV1 {
    fn prepare_as_full_body_child(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        // When embedded as an child, it's SBOR encoded as a struct
        let ((intent, intent_signatures), summary) =
            ConcatenatedDigest::prepare_from_transaction_child_struct(
                decoder,
                TransactionDiscriminator::V1SignedIntent,
            )?;
        Ok(Self {
            intent,
            intent_signatures,
            summary,
        })
    }
}

impl TransactionPayloadPreparable for PreparedSignedIntentV1 {
    fn prepare_for_payload(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        // When embedded as full payload, it's SBOR encoded as an enum
        let ((intent, intent_signatures), summary) =
            ConcatenatedDigest::prepare_from_transaction_payload_enum(
                decoder,
                TransactionDiscriminator::V1SignedIntent,
            )?;
        Ok(Self {
            intent,
            intent_signatures,
            summary,
        })
    }
}

impl HasIntentHash for PreparedSignedIntentV1 {
    fn intent_hash(&self) -> IntentHash {
        self.intent.intent_hash()
    }
}

impl HasSignedIntentHash for PreparedSignedIntentV1 {
    fn signed_intent_hash(&self) -> SignedIntentHash {
        SignedIntentHash::from_hash(self.summary.hash)
    }
}
