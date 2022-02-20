use scrypto::abi;
use scrypto::buffer::*;
use scrypto::crypto::sha256;
use scrypto::engine::types::*;
use scrypto::rust::borrow::ToOwned;
use scrypto::rust::string::ToString;
use scrypto::rust::vec::Vec;

use crate::engine::*;
use crate::errors::*;
use crate::ledger::*;
use crate::model::*;

/// An interface for exporting the ABI of a blueprint.
pub trait AbiProvider {
    /// Exports the ABI of a blueprint.
    fn export_abi(
        &self,
        package_id: PackageId,
        blueprint_name: &str,
    ) -> Result<abi::Blueprint, RuntimeError>;

    /// Exports the ABI of the blueprint, from which the given component is instantiated.
    fn export_abi_component(
        &self,
        component_id: ComponentId,
    ) -> Result<abi::Blueprint, RuntimeError>;
}

/// Provides ABIs for blueprints either installed during bootstrap or added manually.
pub struct BasicAbiProvider {
    ledger: InMemorySubstateStore,
    trace: bool,
}

impl BasicAbiProvider {
    pub fn new(trace: bool) -> Self {
        Self {
            ledger: InMemorySubstateStore::with_bootstrap(),
            trace,
        }
    }

    pub fn with_package(&mut self, package_id: PackageId, code: Vec<u8>) -> &mut Self {
        self.ledger.put_package(package_id, Package::new(code));
        self
    }

    pub fn with_component(
        &mut self,
        component_id: ComponentId,
        package_id: PackageId,
        blueprint_name: &str,
        component_state: Vec<u8>,
    ) -> &mut Self {
        self.ledger.put_component(
            component_id,
            Component::new(package_id, blueprint_name.to_owned(), component_state),
        );
        self
    }
}

impl AbiProvider for BasicAbiProvider {
    fn export_abi(
        &self,
        package_id: PackageId,
        blueprint_name: &str,
    ) -> Result<abi::Blueprint, RuntimeError> {
        // Deterministic transaction context
        let mut ledger = self.ledger.clone();
        let transaction_hash = sha256([]);

        // Start a process and run abi generator
        let mut track = Track::new(&mut ledger, transaction_hash, Vec::new());
        let mut proc = track.start_process(self.trace);
        let output: (Vec<abi::Function>, Vec<abi::Method>) = proc
            .call_abi(package_id, blueprint_name)
            .and_then(|rtn| scrypto_decode(&rtn.raw).map_err(RuntimeError::AbiValidationError))?;

        // Return ABI
        Ok(abi::Blueprint {
            package_id: package_id.to_string(),
            blueprint_name: blueprint_name.to_owned(),
            functions: output.0,
            methods: output.1,
        })
    }

    fn export_abi_component(
        &self,
        component_id: ComponentId,
    ) -> Result<abi::Blueprint, RuntimeError> {
        let component = self
            .ledger
            .get_component(component_id)
            .ok_or(RuntimeError::ComponentNotFound(component_id))?;
        self.export_abi(component.package_id(), component.blueprint_name())
    }
}