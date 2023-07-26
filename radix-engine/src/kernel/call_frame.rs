use crate::track::interface::{
    AcquireLockError, NodeSubstates, RemoveSubstateError, SetSubstateError, StoreAccess,
    StoreAccessInfo, SubstateStore,
};
use crate::types::*;
use radix_engine_interface::api::field_api::LockFlags;
use radix_engine_interface::types::{LockHandle, NodeId, SubstateKey};
use radix_engine_store_interface::db_key_mapper::SubstateKeyContent;

use super::heap::{Heap, HeapOpenSubstateError, HeapRemoveModuleError, HeapRemoveNodeError};
use super::kernel_api::LockInfo;

/// A message used for communication between call frames.
///
/// Note that it's just an intent, not checked/allowed by kernel yet.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub copy_references: Vec<NodeId>,
    pub move_nodes: Vec<NodeId>,
    pub copy_transient_references: Vec<NodeId>,
    pub copy_direct_references: Vec<NodeId>,
}

impl Message {
    pub fn from_indexed_scrypto_value(value: &IndexedScryptoValue) -> Self {
        Self {
            copy_references: value.references().clone(),
            move_nodes: value.owned_nodes().clone(),
            copy_transient_references: vec![],
            copy_direct_references: vec![],
        }
    }

    pub fn add_copy_reference(&mut self, node_id: NodeId) {
        self.copy_references.push(node_id)
    }

    pub fn add_move_node(&mut self, node_id: NodeId) {
        self.move_nodes.push(node_id)
    }
}

/// A lock on a substate controlled by a call frame
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubstateLock<L> {
    pub node_id: NodeId,
    pub partition_num: PartitionNumber,
    pub substate_key: SubstateKey,
    pub non_global_references: IndexSet<NodeId>,
    pub owned_nodes: IndexSet<NodeId>,
    pub flags: LockFlags,
    pub store_handle: Option<u32>,
    pub data: L,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StableReferenceType {
    Global,
    DirectAccess,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TransientReference {
    ref_count: usize,
    ref_origin: ReferenceOrigin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReferenceOrigin {
    Heap,
    Global(GlobalAddress),
    DirectlyAccessed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Visibility {
    StableReference(StableReferenceType),
    FrameOwned,
    Borrowed(ReferenceOrigin),
}

impl Visibility {
    pub fn is_direct_access(&self) -> bool {
        matches!(
            self,
            Self::StableReference(StableReferenceType::DirectAccess)
        )
    }

    pub fn is_normal(&self) -> bool {
        !self.is_direct_access()
    }
}

pub struct NodeVisibility(pub BTreeSet<Visibility>);

impl NodeVisibility {
    /// Note that system may enforce further constraints on this.
    /// For instance, system currently only allows substates of actor,
    /// actor's outer object, and any visible key value store.
    pub fn is_visible(&self) -> bool {
        !self.0.is_empty()
    }

    pub fn can_be_invoked(&self, direct_access: bool) -> bool {
        if direct_access {
            self.0.iter().any(|x| x.is_direct_access())
        } else {
            self.0.iter().any(|x| x.is_normal())
        }
    }

    pub fn can_be_referenced_in_substate(&self) -> bool {
        self.0.iter().any(|x| x.is_normal())
    }

    pub fn can_be_reference_copied_to_frame(&self) -> Option<StableReferenceType> {
        for v in &self.0 {
            if let Visibility::StableReference(t) = v {
                return Some(t.clone());
            }
        }
        return None;
    }

    pub fn reference_origin(&self, node_id: NodeId) -> Option<ReferenceOrigin> {
        let mut found_direct_access = false;
        for v in &self.0 {
            match v {
                Visibility::StableReference(StableReferenceType::Global) => {
                    return Some(ReferenceOrigin::Global(GlobalAddress::new_or_panic(
                        node_id.0,
                    )));
                }
                Visibility::StableReference(StableReferenceType::DirectAccess) => {
                    found_direct_access = true
                }
                Visibility::Borrowed(root_node_type) => return Some(root_node_type.clone()),
                Visibility::FrameOwned => {
                    return Some(ReferenceOrigin::Heap);
                }
            }
        }

        if found_direct_access {
            return Some(ReferenceOrigin::DirectlyAccessed);
        }

        return None;
    }
}

pub trait CallFrameEventHandler {
    fn on_persist_node<S: SubstateStore>(
        &mut self,
        heap: &mut Heap,
        store: &mut S,
        node_id: &NodeId,
    ) -> Result<(), String>;
}

/// A call frame is the basic unit that forms a transaction call stack, which keeps track of the
/// owned objects and references by this function.
pub struct CallFrame<C, L> {
    /// The frame id
    depth: usize,

    /// Call frame system layer data
    call_frame_data: C,

    /// Owned nodes which by definition must live on heap
    /// Also keeps track of number of locks on this node, to prevent locked node from moving.
    owned_root_nodes: IndexMap<NodeId, usize>,

    /// References to non-GLOBAL nodes, obtained from substate loading, ref counted.
    /// These references may NOT be passed between call frames as arguments
    transient_references: NonIterMap<NodeId, TransientReference>,

    /// Stable references points to nodes in track, which can't moved/deleted.
    /// Current two types: `GLOBAL` (root, stored) and `DirectAccess`.
    /// These references MAY be passed between call frames
    stable_references: NonIterMap<NodeId, StableReferenceType>,

    next_lock_handle: LockHandle,
    locks: IndexMap<LockHandle, SubstateLock<L>>,
}

/// Represents an error when creating a new frame.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum CreateFrameError {
    PassMessageError(PassMessageError),
}

/// Represents an error when passing message between frames.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum PassMessageError {
    TakeNodeError(TakeNodeError),
    StableRefNotFound(NodeId),
    TransientRefNotFound(NodeId),
    DirectRefNotFound(NodeId),
}

/// Represents an error when attempting to lock a substate.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum OpenSubstateError {
    NodeNotVisible(NodeId),
    HeapError(HeapOpenSubstateError),
    TrackError(Box<AcquireLockError>),
}

/// Represents an error when dropping a substate lock.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum CloseSubstateError {
    LockNotFound(LockHandle),
    ContainsDuplicatedOwns,
    TakeNodeError(TakeNodeError),
    RefNotFound(NodeId),
    NonGlobalRefNotAllowed(NodeId),
    CantDropNodeInStore(NodeId),
    PersistNodeError(PersistNodeError),
}

/// Represents an error when creating a node.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum CreateNodeError {
    TakeNodeError(TakeNodeError),
    RefNotFound(NodeId),
    NonGlobalRefNotAllowed(NodeId),
    PersistNodeError(PersistNodeError),
}

/// Represents an error when dropping a node.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum DropNodeError {
    TakeNodeError(TakeNodeError),
    NodeBorrowed(NodeId, usize),
}

/// Represents an error when persisting a node into store.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum PersistNodeError {
    NotAllowed(NodeId, String),
    ContainsNonGlobalRef(NodeId),
    NodeBorrowed(NodeId, usize),
}

/// Represents an error when taking a node from current frame.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum TakeNodeError {
    OwnNotFound(NodeId),
    OwnLocked(NodeId),
}

/// Represents an error when listing the node modules of a node.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum ListNodeModuleError {
    NodeNotVisible(NodeId),
    NodeNotInHeap(NodeId),
}

/// Represents an error when moving modules from one node to another.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum MoveModuleError {
    NodeNotAvailable(NodeId),
    HeapRemoveModuleErr(HeapRemoveModuleError),
    NonGlobalRefNotAllowed(NodeId),
    TrackSetSubstateError(SetSubstateError),
    PersistNodeError(PersistNodeError),
}

/// Represents an error when reading substates.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum ReadSubstateError {
    LockNotFound(LockHandle),
}

/// Represents an error when writing substates.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum WriteSubstateError {
    LockNotFound(LockHandle),
    NoWritePermission,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum CallFrameSetSubstateError {
    NodeNotVisible(NodeId),
    StoreError(SetSubstateError),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum CallFrameRemoveSubstateError {
    NodeNotVisible(NodeId),
    StoreError(RemoveSubstateError),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum CallFrameScanKeysError {
    NodeNotVisible(NodeId),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum CallFrameDrainSubstatesError {
    NodeNotVisible(NodeId),
    OwnedNodeNotSupported(NodeId),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum CallFrameScanSortedSubstatesError {
    NodeNotVisible(NodeId),
    OwnedNodeNotSupported(NodeId),
}

impl<C, L: Clone> CallFrame<C, L> {
    pub fn new_root(call_frame_data: C) -> Self {
        Self {
            depth: 0,
            call_frame_data,
            stable_references: NonIterMap::new(),
            transient_references: NonIterMap::new(),
            owned_root_nodes: index_map_new(),
            next_lock_handle: 0u32,
            locks: index_map_new(),
        }
    }

    pub fn new_child_from_parent(
        parent: &mut CallFrame<C, L>,
        call_frame_data: C,
        message: Message,
    ) -> Result<Self, CreateFrameError> {
        let mut frame = Self {
            depth: parent.depth + 1,
            call_frame_data,
            stable_references: NonIterMap::new(),
            transient_references: NonIterMap::new(),
            owned_root_nodes: index_map_new(),
            next_lock_handle: 0u32,
            locks: index_map_new(),
        };

        // Copy references and move nodes
        Self::pass_message(parent, &mut frame, message)
            .map_err(CreateFrameError::PassMessageError)?;

        Ok(frame)
    }

    pub fn pass_message(
        from: &mut CallFrame<C, L>,
        to: &mut CallFrame<C, L>,
        message: Message,
    ) -> Result<(), PassMessageError> {
        for node_id in message.move_nodes {
            // Note that this has no impact on the `transient_references` because
            // we don't allow move of "locked nodes".
            from.take_node_internal(&node_id)
                .map_err(PassMessageError::TakeNodeError)?;
            to.owned_root_nodes.insert(node_id, 0);
        }

        // Only allow move of `Global` and `DirectAccess` references
        for node_id in message.copy_references {
            if let Some(t) = from
                .get_node_visibility(&node_id)
                .can_be_reference_copied_to_frame()
            {
                // Note that GLOBAL and DirectAccess references are mutually exclusive,
                // so okay to overwrite
                to.stable_references.insert(node_id, t);
            } else {
                return Err(PassMessageError::StableRefNotFound(node_id));
            }
        }

        // TODO: Move this logic into system layer
        for node_id in message.copy_transient_references {
            if from.depth >= to.depth {
                panic!("Transient references only supported for downstream calls.");
            }

            if let Some(root_node_type) =
                from.get_node_visibility(&node_id).reference_origin(node_id)
            {
                to.transient_references
                    .entry(node_id.clone())
                    .or_insert(TransientReference {
                        ref_count: 0usize,
                        ref_origin: root_node_type,
                    })
                    .ref_count
                    .add_assign(1);

                if let ReferenceOrigin::Global(global_address) = root_node_type {
                    to.stable_references
                        .insert(global_address.into_node_id(), StableReferenceType::Global);
                }
            } else {
                return Err(PassMessageError::TransientRefNotFound(node_id));
            }
        }

        for node_id in message.copy_direct_references {
            if from.get_node_visibility(&node_id).can_be_invoked(true) {
                to.stable_references
                    .insert(node_id, StableReferenceType::DirectAccess);
            } else {
                return Err(PassMessageError::DirectRefNotFound(node_id));
            }
        }

        Ok(())
    }

    pub fn depth(&self) -> usize {
        self.depth
    }

    pub fn data(&self) -> &C {
        &self.call_frame_data
    }

    pub fn acquire_lock<S: SubstateStore>(
        &mut self,
        heap: &mut Heap,
        store: &mut S,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
        flags: LockFlags,
        default: Option<fn() -> IndexedScryptoValue>,
        data: L,
    ) -> Result<(LockHandle, usize, StoreAccessInfo), OpenSubstateError> {
        let node_visibility = self.get_node_visibility(node_id);
        let root_node_type =
            if let Some(root_node_type) = node_visibility.reference_origin(node_id.clone()) {
                root_node_type
            } else {
                return Err(OpenSubstateError::NodeNotVisible(node_id.clone()));
            };

        // Lock and read the substate
        let mut store_handle = None;
        let mut store_access = StoreAccessInfo::new();
        let substate_value = if heap.contains_node(node_id) {
            // FIXME: we will have to move locking logic to heap because references moves between frames.
            if flags.contains(LockFlags::UNMODIFIED_BASE) {
                return Err(OpenSubstateError::HeapError(
                    HeapOpenSubstateError::LockUnmodifiedBaseOnHeapNode,
                ));
            }
            if let Some(compute_default) = default {
                heap.get_substate_virtualize(node_id, partition_num, substate_key, compute_default)
            } else {
                heap.get_substate(node_id, partition_num, substate_key)
                    .ok_or_else(|| {
                        OpenSubstateError::HeapError(HeapOpenSubstateError::SubstateNotFound(
                            node_id.clone(),
                            partition_num,
                            substate_key.clone(),
                        ))
                    })?
            }
        } else {
            let (handle, store_access_info) = store
                .acquire_lock_virtualize(node_id, partition_num, substate_key, flags, || {
                    default.map(|f| f())
                })
                .map_err(|x| OpenSubstateError::TrackError(Box::new(x)))?;
            store_handle = Some(handle);
            store_access = store_access_info;
            let (value, read_store_access_info) = store.read_substate(handle);
            store_access.extend(&read_store_access_info);
            value
        };

        // Analyze owns and references in the substate
        let mut non_global_references = index_set_new(); // du-duplicated
        let mut owned_nodes = index_set_new();
        for node_id in substate_value.references() {
            if node_id.is_global() {
                // Again, safe to overwrite because Global and DirectAccess are exclusive.
                self.stable_references
                    .insert(node_id.clone(), StableReferenceType::Global);
            } else {
                non_global_references.insert(node_id.clone());
            }
        }
        for node_id in substate_value.owned_nodes() {
            if !owned_nodes.insert(node_id.clone()) {
                panic!("Duplicated own found in substate");
            }
        }

        // Expand transient reference set
        for reference in &non_global_references {
            self.transient_references
                .entry(reference.clone())
                .or_insert(TransientReference {
                    ref_count: 0usize,
                    ref_origin: root_node_type,
                })
                .ref_count
                .add_assign(1);
        }
        for own in &owned_nodes {
            self.transient_references
                .entry(own.clone())
                .or_insert(TransientReference {
                    ref_count: 0usize,
                    ref_origin: root_node_type,
                })
                .ref_count
                .add_assign(1);
        }

        // Issue lock handle
        let lock_handle = self.next_lock_handle;
        self.locks.insert(
            lock_handle,
            SubstateLock {
                node_id: node_id.clone(),
                partition_num,
                substate_key: substate_key.clone(),
                non_global_references,
                owned_nodes,
                flags,
                store_handle,
                data,
            },
        );
        self.next_lock_handle = self.next_lock_handle + 1;

        // Update lock count on the node
        if let Some(counter) = self.owned_root_nodes.get_mut(node_id) {
            *counter += 1;
        }

        Ok((lock_handle, substate_value.len(), store_access))
    }

    pub fn close_substate<S: SubstateStore>(
        &mut self,
        heap: &mut Heap,
        store: &mut S,
        handler: &mut impl CallFrameEventHandler,
        lock_handle: LockHandle,
    ) -> Result<StoreAccessInfo, CloseSubstateError> {
        let substate_lock = self
            .locks
            .remove(&lock_handle)
            .ok_or_else(|| CloseSubstateError::LockNotFound(lock_handle))?;

        let node_id = &substate_lock.node_id;
        let partition_num = substate_lock.partition_num;
        let substate_key = &substate_lock.substate_key;
        let mut store_access = StoreAccessInfo::new();

        if substate_lock.flags.contains(LockFlags::MUTABLE) {
            let substate = if let Some(handle) = substate_lock.store_handle {
                let (substate, read_store_access) = store.read_substate(handle);
                store_access.extend(&read_store_access);
                substate
            } else {
                heap.get_substate(node_id, partition_num, substate_key)
                    .expect("Substate locked but missing")
            }
            .clone();

            //==============
            // Process owns
            //==============
            let mut new_owned_nodes: IndexSet<NodeId> = index_set_new();
            for own in substate.owned_nodes() {
                if !new_owned_nodes.insert(own.clone()) {
                    return Err(CloseSubstateError::ContainsDuplicatedOwns);
                }
            }
            for own in &new_owned_nodes {
                if !substate_lock.owned_nodes.contains(own) {
                    // Node no longer owned by frame
                    self.take_node_internal(own)
                        .map_err(CloseSubstateError::TakeNodeError)?;

                    // Move the node to store, if its owner is already in store
                    if !heap.contains_node(&node_id) {
                        Self::move_node_to_store(heap, store, handler, own)
                            .map_err(CloseSubstateError::PersistNodeError)?;
                    }
                }
            }
            for own in &substate_lock.owned_nodes {
                if !new_owned_nodes.contains(own) {
                    // Node detached
                    if !heap.contains_node(node_id) {
                        return Err(CloseSubstateError::CantDropNodeInStore(own.clone()));
                    }
                    // Owned nodes discarded by the substate go back to the call frame,
                    // and must be explicitly dropped.
                    // FIXME(Yulong): I suspect this is buggy as one can detach a locked non-root
                    // node, move and drop; which will cause invalid lock handle in previous frames.
                    // FIXME(Josh): Would prefer removing this case entirely as this edge case
                    // means that a component's logic may or may not work depending on whether
                    // it's in the store or the heap, which I think feels very unintuitive.
                    // Rather, let's fix the specific worktop drop bucket issue
                    self.owned_root_nodes.insert(own.clone(), 0);
                }
            }

            //====================
            // Process references
            //====================
            let mut new_references: IndexSet<NodeId> = index_set_new();
            for own in substate.references() {
                // Deduplicate
                new_references.insert(own.clone());
            }
            for reference in &new_references {
                if !substate_lock.non_global_references.contains(reference) {
                    // handle added references

                    if !self
                        .get_node_visibility(reference)
                        .can_be_referenced_in_substate()
                    {
                        return Err(CloseSubstateError::RefNotFound(reference.clone()));
                    }

                    if !heap.contains_node(node_id) && !reference.is_global() {
                        return Err(CloseSubstateError::NonGlobalRefNotAllowed(*reference));
                    }

                    if heap.contains_node(reference) {
                        heap.increase_borrow_count(reference);
                    } else {
                        // No op
                    }
                }
            }
            for reference in &substate_lock.non_global_references {
                if !new_references.contains(reference) {
                    // handle removed references

                    if heap.contains_node(reference) {
                        heap.decrease_borrow_count(reference);
                    }
                }
            }
        }

        // Shrink transient reference set
        for reference in substate_lock.non_global_references {
            let mut transient_ref = self.transient_references.remove(&reference).unwrap();
            if transient_ref.ref_count > 1 {
                transient_ref.ref_count -= 1;
                self.transient_references.insert(reference, transient_ref);
            }
        }
        for own in substate_lock.owned_nodes {
            let mut transient_ref = self.transient_references.remove(&own).unwrap();
            if transient_ref.ref_count > 1 {
                transient_ref.ref_count -= 1;
                self.transient_references.insert(own, transient_ref);
            }
        }

        // Update node lock count
        if let Some(counter) = self.owned_root_nodes.get_mut(&substate_lock.node_id) {
            *counter -= 1;
        }

        // Release track lock
        if let Some(handle) = substate_lock.store_handle {
            store_access.extend(&store.close_substate(handle));
        }

        Ok(store_access)
    }

    pub fn get_lock_info(&self, lock_handle: LockHandle) -> Option<LockInfo<L>> {
        self.locks.get(&lock_handle).map(|substate_lock| LockInfo {
            node_id: substate_lock.node_id,
            partition_num: substate_lock.partition_num,
            substate_key: substate_lock.substate_key.clone(),
            flags: substate_lock.flags,
            data: substate_lock.data.clone(),
        })
    }

    pub fn read_substate<'f, S: SubstateStore>(
        &mut self,
        heap: &'f mut Heap,
        store: &'f mut S,
        lock_handle: LockHandle,
    ) -> Result<(&'f IndexedScryptoValue, StoreAccessInfo), ReadSubstateError> {
        let SubstateLock {
            node_id,
            partition_num,
            substate_key,
            store_handle,
            ..
        } = self
            .locks
            .get(&lock_handle)
            .ok_or(ReadSubstateError::LockNotFound(lock_handle))?;

        if let Some(store_handle) = store_handle {
            Ok(store.read_substate(*store_handle))
        } else {
            Ok((
                heap.get_substate(node_id, *partition_num, substate_key)
                    .expect("Substate missing in heap"),
                StoreAccessInfo::new(),
            ))
        }
    }

    pub fn write_substate<'f, S: SubstateStore>(
        &mut self,
        heap: &'f mut Heap,
        store: &'f mut S,
        lock_handle: LockHandle,
        substate: IndexedScryptoValue,
    ) -> Result<StoreAccessInfo, WriteSubstateError> {
        let SubstateLock {
            node_id,
            partition_num,
            substate_key,
            store_handle,
            flags,
            ..
        } = self
            .locks
            .get(&lock_handle)
            .ok_or(WriteSubstateError::LockNotFound(lock_handle))?;

        if !flags.contains(LockFlags::MUTABLE) {
            return Err(WriteSubstateError::NoWritePermission);
        }

        if let Some(store_handle) = store_handle {
            Ok(store.update_substate(*store_handle, substate))
        } else {
            heap.set_substate(*node_id, *partition_num, substate_key.clone(), substate);
            Ok(StoreAccessInfo::new())
        }
    }

    pub fn create_node<'f, S: SubstateStore>(
        &mut self,
        node_id: NodeId,
        node_substates: NodeSubstates,
        heap: &mut Heap,
        store: &'f mut S,
        handler: &mut impl CallFrameEventHandler,
    ) -> Result<StoreAccessInfo, CreateNodeError> {
        let push_to_store = node_id.is_global();
        for (_partition_number, module) in &node_substates {
            for (_substate_key, substate_value) in module {
                //==============
                // Process owns
                //==============
                for own in substate_value.owned_nodes() {
                    self.take_node_internal(own)
                        .map_err(CreateNodeError::TakeNodeError)?;
                    if push_to_store {
                        Self::move_node_to_store(heap, store, handler, own)
                            .map_err(CreateNodeError::PersistNodeError)?;
                    }
                }

                //===================
                // Process reference
                //===================
                for reference in substate_value.references() {
                    if !self
                        .get_node_visibility(reference)
                        .can_be_referenced_in_substate()
                    {
                        return Err(CreateNodeError::RefNotFound(reference.clone()));
                    }

                    if push_to_store && !reference.is_global() {
                        return Err(CreateNodeError::NonGlobalRefNotAllowed(*reference));
                    }

                    if heap.contains_node(reference) {
                        heap.increase_borrow_count(reference);
                    } else {
                        // No op
                    }
                }
            }
        }

        let store_access = if push_to_store {
            self.stable_references
                .insert(node_id, StableReferenceType::Global);
            store.create_node(node_id, node_substates)
        } else {
            heap.create_node(node_id, node_substates);
            self.owned_root_nodes.insert(node_id, 0);
            StoreAccessInfo::new()
        };

        Ok(store_access)
    }

    /// Removes node from call frame and owned nodes will be possessed by this call frame.
    pub fn drop_node(
        &mut self,
        heap: &mut Heap,
        node_id: &NodeId,
    ) -> Result<NodeSubstates, DropNodeError> {
        self.take_node_internal(node_id)
            .map_err(DropNodeError::TakeNodeError)?;
        let node_substates = match heap.remove_node(node_id) {
            Ok(substates) => substates,
            Err(HeapRemoveNodeError::NodeNotFound(node_id)) => {
                panic!("Frame owned node {:?} not found in heap", node_id)
            }
            Err(HeapRemoveNodeError::NodeBorrowed(node_id, count)) => {
                return Err(DropNodeError::NodeBorrowed(node_id, count));
            }
        };
        for (_, module) in &node_substates {
            for (_, substate_value) in module {
                //=============
                // Process own
                //=============
                for own in substate_value.owned_nodes() {
                    // FIXME: This is problematic, as owned node must have been locked
                    // In general, we'd like to move node locking/borrowing to heap.
                    self.owned_root_nodes.insert(own.clone(), 0);
                }

                //====================
                // Process references
                //====================
                for reference in substate_value.references() {
                    if reference.is_global() {
                        // Expand stable references
                        // We keep all global references even if the owning substates are dropped.
                        // Revisit this if the reference model is changed.
                        self.stable_references
                            .insert(reference.clone(), StableReferenceType::Global);
                    } else {
                        if heap.contains_node(reference) {
                            // This substate is dropped and no longer borrows the heap node.
                            heap.decrease_borrow_count(reference);
                        }
                    }
                }
            }
        }
        Ok(node_substates)
    }

    pub fn move_module<'f, S: SubstateStore>(
        &mut self,
        src_node_id: &NodeId,
        src_partition_number: PartitionNumber,
        dest_node_id: &NodeId,
        dest_partition_number: PartitionNumber,
        heap: &'f mut Heap,
        store: &'f mut S,
        handler: &mut impl CallFrameEventHandler,
    ) -> Result<StoreAccessInfo, MoveModuleError> {
        // Check ownership (and visibility)
        if self.owned_root_nodes.get(src_node_id) != Some(&0) {
            return Err(MoveModuleError::NodeNotAvailable(src_node_id.clone()));
        }

        // Check visibility
        if !self.get_node_visibility(dest_node_id).is_visible() {
            return Err(MoveModuleError::NodeNotAvailable(dest_node_id.clone()));
        }

        let mut store_access = Vec::<StoreAccess>::new();

        // Move
        let module = heap
            .remove_module(src_node_id, src_partition_number)
            .map_err(MoveModuleError::HeapRemoveModuleErr)?;
        let to_heap = heap.contains_node(dest_node_id);
        for (substate_key, substate_value) in module {
            if to_heap {
                heap.set_substate(
                    *dest_node_id,
                    dest_partition_number,
                    substate_key,
                    substate_value,
                );
            } else {
                // Recursively move nodes to store
                for own in substate_value.owned_nodes() {
                    Self::move_node_to_store(heap, store, handler, own)
                        .map_err(MoveModuleError::PersistNodeError)?;
                }

                for reference in substate_value.references() {
                    if !reference.is_global() {
                        return Err(MoveModuleError::NonGlobalRefNotAllowed(reference.clone()));
                    }
                }

                store_access.extend(
                    store
                        .set_substate(
                            *dest_node_id,
                            dest_partition_number,
                            substate_key,
                            substate_value,
                        )
                        .map_err(MoveModuleError::TrackSetSubstateError)?,
                );
            }
        }

        Ok(store_access)
    }

    pub fn add_global_reference(&mut self, address: GlobalAddress) {
        self.stable_references
            .insert(address.into_node_id(), StableReferenceType::Global);
    }

    pub fn add_direct_access_reference(&mut self, address: InternalAddress) {
        self.stable_references
            .insert(address.into_node_id(), StableReferenceType::DirectAccess);
    }

    //====================================================================================
    // Note that reference model isn't fully implemented for set/remove/scan/take APIs.
    // They're intended for internal use only and extra caution must be taken.
    //====================================================================================

    // Substate Virtualization does not apply to this call
    // Should this be prevented at this layer?
    pub fn set_substate<'f, S: SubstateStore>(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        key: SubstateKey,
        value: IndexedScryptoValue,
        heap: &'f mut Heap,
        store: &'f mut S,
    ) -> Result<StoreAccessInfo, CallFrameSetSubstateError> {
        // Check node visibility
        if !self.get_node_visibility(node_id).is_visible() {
            return Err(CallFrameSetSubstateError::NodeNotVisible(node_id.clone()));
        }

        let store_access = if heap.contains_node(node_id) {
            heap.set_substate(*node_id, partition_num, key, value);
            StoreAccessInfo::new()
        } else {
            store
                .set_substate(*node_id, partition_num, key, value)
                .map_err(|e| CallFrameSetSubstateError::StoreError(e))?
        };

        Ok(store_access)
    }

    pub fn remove_substate<'f, S: SubstateStore>(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        key: &SubstateKey,
        heap: &'f mut Heap,
        store: &'f mut S,
    ) -> Result<(Option<IndexedScryptoValue>, StoreAccessInfo), CallFrameRemoveSubstateError> {
        // Check node visibility
        if !self.get_node_visibility(node_id).is_visible() {
            return Err(CallFrameRemoveSubstateError::NodeNotVisible(
                node_id.clone(),
            ));
        }

        let (removed, store_access) = if heap.contains_node(node_id) {
            (
                heap.remove_substate(node_id, partition_num, key),
                StoreAccessInfo::new(),
            )
        } else {
            store
                .remove_substate(node_id, partition_num, key)
                .map_err(|e| CallFrameRemoveSubstateError::StoreError(e))?
        };

        Ok((removed, store_access))
    }

    pub fn scan_keys<'f, K: SubstateKeyContent, S: SubstateStore>(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        count: u32,
        heap: &'f mut Heap,
        store: &'f mut S,
    ) -> Result<(Vec<SubstateKey>, StoreAccessInfo), CallFrameScanKeysError> {
        // Check node visibility
        if !self.get_node_visibility(node_id).is_visible() {
            return Err(CallFrameScanKeysError::NodeNotVisible(node_id.clone()));
        }

        let (keys, store_access) = if heap.contains_node(node_id) {
            (
                heap.scan_keys(node_id, partition_num, count),
                StoreAccessInfo::new(),
            )
        } else {
            store.scan_keys::<K>(node_id, partition_num, count)
        };

        Ok((keys, store_access))
    }

    pub fn drain_substates<'f, K: SubstateKeyContent, S: SubstateStore>(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        count: u32,
        heap: &'f mut Heap,
        store: &'f mut S,
    ) -> Result<
        (Vec<(SubstateKey, IndexedScryptoValue)>, StoreAccessInfo),
        CallFrameDrainSubstatesError,
    > {
        // Check node visibility
        if !self.get_node_visibility(node_id).is_visible() {
            return Err(CallFrameDrainSubstatesError::NodeNotVisible(
                node_id.clone(),
            ));
        }

        let (substates, store_access) = if heap.contains_node(node_id) {
            (
                heap.drain_substates(node_id, partition_num, count),
                StoreAccessInfo::new(),
            )
        } else {
            store.drain_substates::<K>(node_id, partition_num, count)
        };

        for (_key, substate) in &substates {
            for reference in substate.references() {
                if reference.is_global() {
                    self.stable_references
                        .insert(reference.clone(), StableReferenceType::Global);
                } else {
                    return Err(CallFrameDrainSubstatesError::OwnedNodeNotSupported(
                        reference.clone(),
                    ));
                }
            }
        }

        Ok((substates, store_access))
    }

    // Substate Virtualization does not apply to this call
    // Should this be prevented at this layer?
    pub fn scan_sorted<'f, S: SubstateStore>(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        count: u32,
        heap: &'f mut Heap,
        store: &'f mut S,
    ) -> Result<(Vec<IndexedScryptoValue>, StoreAccessInfo), CallFrameScanSortedSubstatesError>
    {
        // Check node visibility
        if !self.get_node_visibility(node_id).is_visible() {
            return Err(CallFrameScanSortedSubstatesError::NodeNotVisible(
                node_id.clone(),
            ));
        }

        let (substates, store_access) = if heap.contains_node(node_id) {
            // This should never be triggered because sorted index store is
            // used by consensus manager only.
            panic!("Unexpected code path")
        } else {
            store.scan_sorted_substates(node_id, partition_num, count)
        };

        for substate in &substates {
            for reference in substate.references() {
                if reference.is_global() {
                    self.stable_references
                        .insert(reference.clone(), StableReferenceType::Global);
                } else {
                    return Err(CallFrameScanSortedSubstatesError::OwnedNodeNotSupported(
                        reference.clone(),
                    ));
                }
            }
        }

        Ok((substates, store_access))
    }

    pub fn drop_all_locks<S: SubstateStore>(
        &mut self,
        heap: &mut Heap,
        store: &mut S,
        handler: &mut impl CallFrameEventHandler,
    ) -> Result<(), CloseSubstateError> {
        let lock_handles: Vec<LockHandle> = self.locks.keys().cloned().collect();

        for lock_handle in lock_handles {
            self.close_substate(heap, store, handler, lock_handle)?;
        }

        Ok(())
    }

    fn take_node_internal(&mut self, node_id: &NodeId) -> Result<(), TakeNodeError> {
        match self.owned_root_nodes.remove(node_id) {
            None => {
                return Err(TakeNodeError::OwnNotFound(node_id.clone()));
            }
            Some(lock_count) => {
                if lock_count == 0 {
                    Ok(())
                } else {
                    Err(TakeNodeError::OwnLocked(node_id.clone()))
                }
            }
        }
    }

    pub fn owned_nodes(&self) -> Vec<NodeId> {
        self.owned_root_nodes.keys().cloned().collect()
    }

    pub fn move_node_to_store<S: SubstateStore>(
        heap: &mut Heap,
        store: &mut S,
        handler: &mut impl CallFrameEventHandler,
        node_id: &NodeId,
    ) -> Result<(), PersistNodeError> {
        handler
            .on_persist_node(heap, store, node_id)
            .map_err(|e| PersistNodeError::NotAllowed(node_id.clone(), e))?;

        let node_substates = match heap.remove_node(node_id) {
            Ok(substates) => substates,
            Err(HeapRemoveNodeError::NodeNotFound(node_id)) => {
                panic!("Frame owned node {:?} not found in heap", node_id)
            }
            Err(HeapRemoveNodeError::NodeBorrowed(node_id, count)) => {
                return Err(PersistNodeError::NodeBorrowed(node_id, count));
            }
        };
        for (_partition_number, module_substates) in &node_substates {
            for (_substate_key, substate_value) in module_substates {
                for reference in substate_value.references() {
                    if !reference.is_global() {
                        return Err(PersistNodeError::ContainsNonGlobalRef(*reference));
                    }
                }

                for node_id in substate_value.owned_nodes() {
                    Self::move_node_to_store(heap, store, handler, node_id)?;
                }
            }
        }

        store.create_node(node_id.clone(), node_substates);

        Ok(())
    }

    pub fn get_node_visibility(&self, node_id: &NodeId) -> NodeVisibility {
        let mut visibilities = BTreeSet::<Visibility>::new();

        // Stable references
        if let Some(reference_type) = self.stable_references.get(node_id) {
            visibilities.insert(Visibility::StableReference(reference_type.clone()));
        }
        if ALWAYS_VISIBLE_GLOBAL_NODES.contains(node_id) {
            visibilities.insert(Visibility::StableReference(StableReferenceType::Global));
        }

        // Frame owned nodes
        if self.owned_root_nodes.contains_key(node_id) {
            visibilities.insert(Visibility::FrameOwned);
        }

        // Borrowed from substate loading
        if let Some(transient_ref) = self.transient_references.get(node_id) {
            visibilities.insert(Visibility::Borrowed(transient_ref.ref_origin));
        }

        NodeVisibility(visibilities)
    }
}
