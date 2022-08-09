#[rustfmt::skip]
pub mod test_runner;

use radix_engine::ledger::InMemorySubstateStore;
use radix_engine::wasm::InvokeError;
use scrypto::core::Network;
use scrypto::prelude::{Package, SYSTEM_COMPONENT};
use scrypto::to_struct;
use test_runner::{abi_single_fn_any_input_void_output, wat2wasm, TestRunner};
use transaction::builder::ManifestBuilder;

#[test]
fn test_loop() {
    // Arrange
    let mut store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);

    // Act
    let code = wat2wasm(&include_str!("wasm/loop.wat").replace("${n}", "2000"));
    let package = Package {
        code,
        blueprints: abi_single_fn_any_input_void_output("Test", "f"),
    };
    let package_address = test_runner.publish_package(package);
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .lock_fee(10.into(), SYSTEM_COMPONENT)
        .call_function(package_address, "Test", "f", to_struct!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_success();
}

#[test]
fn test_loop_out_of_cost_unit() {
    // Arrange
    let mut store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);

    // Act
    let code = wat2wasm(&include_str!("wasm/loop.wat").replace("${n}", "2000000"));
    let package = Package {
        code,
        blueprints: abi_single_fn_any_input_void_output("Test", "f"),
    };
    let package_address = test_runner.publish_package(package);
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .lock_fee(10.into(), SYSTEM_COMPONENT)
        .call_function(package_address, "Test", "f", to_struct!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    assert_invoke_error!(receipt.status, InvokeError::CostingError { .. })
}

#[test]
fn test_recursion() {
    // Arrange
    let mut store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);

    // Act
    // In this test case, each call frame costs 4 stack units
    let code = wat2wasm(&include_str!("wasm/recursion.wat").replace("${n}", "128"));
    let package = Package {
        code,
        blueprints: abi_single_fn_any_input_void_output("Test", "f"),
    };
    let package_address = test_runner.publish_package(package);
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .lock_fee(10.into(), SYSTEM_COMPONENT)
        .call_function(package_address, "Test", "f", to_struct!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_success();
}

#[test]
fn test_recursion_stack_overflow() {
    // Arrange
    let mut store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);

    // Act
    let code = wat2wasm(&include_str!("wasm/recursion.wat").replace("${n}", "129"));
    let package = Package {
        code,
        blueprints: abi_single_fn_any_input_void_output("Test", "f"),
    };
    let package_address = test_runner.publish_package(package);
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .lock_fee(10.into(), SYSTEM_COMPONENT)
        .call_function(package_address, "Test", "f", to_struct!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    assert_invoke_error!(receipt.status, InvokeError::WasmError { .. })
}

#[test]
fn test_grow_memory() {
    // Arrange
    let mut store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);

    // Act
    let code = wat2wasm(&include_str!("wasm/memory.wat").replace("${n}", "100"));
    let package = Package {
        code,
        blueprints: abi_single_fn_any_input_void_output("Test", "f"),
    };
    let package_address = test_runner.publish_package(package);
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .lock_fee(10.into(), SYSTEM_COMPONENT)
        .call_function(package_address, "Test", "f", to_struct!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_success();
}

#[test]
fn test_grow_memory_out_of_cost_unit() {
    // Arrange
    let mut store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);

    // Act
    let code = wat2wasm(&include_str!("wasm/memory.wat").replace("${n}", "100000"));
    let package = Package {
        code,
        blueprints: abi_single_fn_any_input_void_output("Test", "f"),
    };
    let package_address = test_runner.publish_package(package);
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .lock_fee(10.into(), SYSTEM_COMPONENT)
        .call_function(package_address, "Test", "f", to_struct!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    assert_invoke_error!(receipt.status, InvokeError::CostingError { .. })
}

#[test]
fn test_total_cost_unit_consumed() {
    // Arrange
    let mut store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);

    // Act
    let code = wat2wasm(&include_str!("wasm/syscall.wat"));
    let package = Package {
        code,
        blueprints: abi_single_fn_any_input_void_output("Test", "f"),
    };
    let (public_key, _, _) = test_runner.new_account();
    let package_address = test_runner.publish_package(package);
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .lock_fee(10.into(), SYSTEM_COMPONENT)
        .call_function(package_address, "Test", "f", to_struct!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);

    // Assert
    /*
        borrow                        :     1000
        create                        :    10000
        emit_log                      :     1050
        invoke_function               :     3025
        invoke_method                 :     5495
        read                          :     5000
        return                        :     1000
        run_function                  :    10000
        run_method                    :    35000
        run_wasm                      :    99849
        tx_decoding                   :     2032
        tx_manifest_verification      :      508
        tx_signature_verification     :     3750
        write                         :     5000
    */
    assert_eq!(
        1000 + 10000
            + 1050
            + 3025
            + 5495
            + 5000
            + 1000
            + 10000
            + 35000
            + 99849
            + 2032
            + 508
            + 3750
            + 5000,
        receipt.fee_summary.cost_unit_consumed
    );
}
