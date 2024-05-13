//! Serialization traits and adapters.

pub mod allocator;
pub mod sharing;
pub mod writer;

use ::core::{alloc::Layout, ptr::NonNull};
use rancor::Strategy;

#[doc(inline)]
pub use self::{
    allocator::Allocator,
    sharing::{Sharing, SharingExt},
    writer::{Positional, Writer, WriterExt},
};
#[cfg(feature = "alloc")]
use crate::ser::{allocator::ArenaHandle, sharing::Share};
use crate::ser::{allocator::SubAllocator, sharing::Unshare};

/// A serializer built from composeable pieces.
#[derive(Debug, Default)]
pub struct Serializer<W, A, S> {
    /// The writer of the serializer.
    pub writer: W,
    /// The allocator of the serializer.
    pub allocator: A,
    /// The pointer sharing of the serializer.
    pub sharing: S,
}

impl<W, A, S> Serializer<W, A, S> {
    /// Creates a new serializer from a writer, allocator, and pointer sharing.
    pub fn new(writer: W, allocator: A, sharing: S) -> Self {
        Self {
            writer,
            allocator,
            sharing,
        }
    }

    /// Consumes the serializer and returns the components.
    pub fn into_raw_parts(self) -> (W, A, S) {
        (self.writer, self.allocator, self.sharing)
    }

    /// Consumes the serializer and returns the writer.
    ///
    /// The allocator and pointer sharing are discarded.
    pub fn into_writer(self) -> W {
        self.writer
    }
}

impl<W: Positional, A, S> Positional for Serializer<W, A, S> {
    fn pos(&self) -> usize {
        self.writer.pos()
    }
}

impl<W: Writer<E>, A, S, E> Writer<E> for Serializer<W, A, S> {
    fn write(&mut self, bytes: &[u8]) -> Result<(), E> {
        self.writer.write(bytes)
    }
}

unsafe impl<W, A: Allocator<E>, S, E> Allocator<E> for Serializer<W, A, S> {
    unsafe fn push_alloc(
        &mut self,
        layout: Layout,
    ) -> Result<NonNull<[u8]>, E> {
        // SAFETY: The safety requirements for `A::push_alloc()` are the same as
        // the safety requirements for `push_alloc()`.
        unsafe { self.allocator.push_alloc(layout) }
    }

    unsafe fn pop_alloc(
        &mut self,
        ptr: NonNull<u8>,
        layout: Layout,
    ) -> Result<(), E> {
        // SAFETY: The safety requirements for `A::pop_alloc()` are the same as
        // the safety requirements for `pop_alloc()`.
        unsafe { self.allocator.pop_alloc(ptr, layout) }
    }
}

impl<W, A, S: Sharing<E>, E> Sharing<E> for Serializer<W, A, S> {
    fn get_shared_ptr(&self, address: usize) -> Option<usize> {
        self.sharing.get_shared_ptr(address)
    }

    fn add_shared_ptr(&mut self, address: usize, pos: usize) -> Result<(), E> {
        self.sharing.add_shared_ptr(address, pos)
    }
}

/// A serializer suitable for environments where allocations cannot be made.
pub type CoreSerializer<'a, W, E> =
    Strategy<Serializer<W, SubAllocator<'a>, Unshare>, E>;

/// A general-purpose serializer suitable for environments where allocations can
/// be made.
#[cfg(feature = "alloc")]
pub type DefaultSerializer<'a, W, E> =
    Strategy<Serializer<W, ArenaHandle<'a>, Share>, E>;
