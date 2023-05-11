use radix_engine::system::bootstrap::Bootstrapper;
use radix_engine::transaction::{
    execute_and_commit_transaction, ExecutionConfig, FeeReserveConfig,
};
use radix_engine::types::*;
use radix_engine::vm::wasm::{DefaultWasmEngine, WasmInstrumenter, WasmMeteringConfig};
use radix_engine::vm::ScryptoVm;
use radix_engine_interface::blueprints::resource::AccessRule;
use radix_engine_stores::memory_db::InMemorySubstateDatabase;
use rand::Rng;
use rand_chacha;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha8Rng;
use transaction::builder::{ManifestBuilder, TransactionBuilder};
use transaction::ecdsa_secp256k1::EcdsaSecp256k1PrivateKey;
use transaction::model::{NotarizedTransaction, TransactionHeader};
use transaction::validation::{
    NotarizedTransactionValidator, TestIntentHashManager, TransactionValidator, ValidationConfig,
};

struct TransactionFuzzer {
    rng: ChaCha8Rng,
    scrypto_interpreter: ScryptoVm<DefaultWasmEngine>,
    substate_db: InMemorySubstateDatabase,
}

impl TransactionFuzzer {
    fn new() -> Self {
        let rng = ChaCha8Rng::seed_from_u64(1234);

        let scrypto_interpreter = ScryptoVm {
            wasm_engine: DefaultWasmEngine::default(),
            wasm_instrumenter: WasmInstrumenter::default(),
            wasm_metering_config: WasmMeteringConfig::V0,
        };
        let mut substate_db = InMemorySubstateDatabase::standard();
        Bootstrapper::new(&mut substate_db, &scrypto_interpreter, false)
            .bootstrap_test_default()
            .unwrap();

        Self {
            rng,
            scrypto_interpreter,
            substate_db,
        }
    }

    fn execute_single_transaction(&mut self, transaction: NotarizedTransaction) {
        let validator = NotarizedTransactionValidator::new(ValidationConfig::simulator());

        let executable = validator
            .validate(&transaction, 0, &TestIntentHashManager::new())
            .unwrap();

        let execution_config = ExecutionConfig::default();
        let fee_reserve_config = FeeReserveConfig::default();

        execute_and_commit_transaction(
            &mut self.substate_db,
            &self.scrypto_interpreter,
            &fee_reserve_config,
            &execution_config,
            &executable,
        );
    }

    fn next_transaction(&mut self) -> NotarizedTransaction {
        let mut builder = ManifestBuilder::new();
        let instruction_count = self.rng.gen_range(0u32..20u32);
        for _ in 0..instruction_count {
            let next = self.rng.gen_range(0u32..4u32);
            match next {
                0 => {
                    let mut authority_rules = AuthorityRules::new();
                    authority_rules.set_authority(
                        "owner",
                        AccessRule::AllowAll,
                        AccessRule::AllowAll,
                    );

                    builder.new_account_advanced(authority_rules);
                }
                1 => {
                    let mut authority_rules = AuthorityRules::new();
                    authority_rules.set_authority(
                        "owner",
                        AccessRule::AllowAll,
                        AccessRule::AllowAll,
                    );
                    builder.new_account_advanced(authority_rules);
                }
                2 => {
                    let mut authority_rules = AuthorityRules::new();
                    authority_rules.set_authority(
                        "owner",
                        AccessRule::AllowAll,
                        AccessRule::AllowAll,
                    );
                    builder.new_account_advanced(authority_rules);
                }
                3 => {
                    builder.call_method(FAUCET, "lock_fee", manifest_args!(dec!("100")));
                }
                _ => panic!("Unexpected"),
            }
        }

        let manifest = builder.build();
        let private_key = EcdsaSecp256k1PrivateKey::from_u64(1).unwrap();
        let header = TransactionHeader {
            version: 1,
            network_id: NetworkDefinition::simulator().id,
            start_epoch_inclusive: 0,
            end_epoch_exclusive: 100,
            nonce: 5,
            notary_public_key: private_key.public_key().into(),
            notary_as_signatory: false,
            cost_unit_limit: 10_000_000,
            tip_percentage: 0,
        };

        TransactionBuilder::new()
            .header(header)
            .manifest(manifest)
            .sign(&private_key)
            .notarize(&private_key)
            .build()
    }
}

#[test]
fn simple_transaction_fuzz_test() {
    let mut fuzzer = TransactionFuzzer::new();
    let transactions: Vec<NotarizedTransaction> = (0..50)
        .into_iter()
        .map(|_| fuzzer.next_transaction())
        .collect();
    transactions.into_iter().for_each(|transaction| {
        fuzzer.execute_single_transaction(transaction);
    });
}
