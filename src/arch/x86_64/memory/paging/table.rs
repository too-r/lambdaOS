use arch::memory::paging::entry::EntryFlags;
use arch::memory::paging::entry::*;
use arch::memory::paging::ENTRY_COUNT;
use arch::memory::FrameAllocator;
use core::ops::{Index, IndexMut};
use core::marker::PhantomData;

/// This physical address will point to the highest-level P4 table.
pub const P4: *mut Table<Level4> = 0xffffffff_fffff000 as *mut _;

pub struct Table<L: TableLevel> {
    entries: [Entry; ENTRY_COUNT],
    level: PhantomData<L>,
}

impl<L> Table<L>
where
    L: TableLevel,
{
    /// Set each entry of the page table as unused - leaves them free.
    pub fn zero(&mut self) {
        for entry in self.entries.iter_mut() {
            entry.set_unused();
        }
    }
}

impl<L> Table<L>
where
    L: HierarchicalLevel,
{
    /// Get the address of the next-lowest page table, using the passed index which should be the
    /// index of the next page table in the current-level page table.
    fn next_table_address(&self, index: usize) -> Option<usize> {
        let entry_flags = self[index].flags();
        if entry_flags.contains(EntryFlags::PRESENT) && !entry_flags.contains(EntryFlags::HUGE_PAGE)
        {
            let table_address = self as *const _ as usize;
            Some((table_address << 9) | (index << 12))
        } else {
            None
        }
    }
    
    /// Return a reference to the next table.
    pub fn next_table(&self, index: usize) -> Option<&Table<L::NextLevel>> {
        self.next_table_address(index)
            .map(|address| unsafe { &*(address as *const _) })
    }
    
    /// Return a mutable reference to the next table.
    pub fn next_table_mut(&mut self, index: usize) -> Option<&mut Table<L::NextLevel>> {
        self.next_table_address(index)
            .map(|address| unsafe { &mut *(address as *mut _) })
    }
    
    /// Allocate a single 4096-byte physical frame for the next level page table and set the given
    /// index to point to the new frame. Set important flags on the table so we can modify it and
    /// return a mutable reference. Note - this code will panic if the given index has flags set on
    /// it to indicate a huge page.
    pub fn next_table_create<A>(
        &mut self,
        index: usize,
        allocator: &mut A,
    ) -> &mut Table<L::NextLevel>
    where
        A: FrameAllocator,
    {
        if self.next_table(index).is_none() {
            assert!(
                !self.entries[index].flags().contains(EntryFlags::HUGE_PAGE),
                "mapping code does not support huge pages"
            );
            let frame = allocator.allocate_frame(1).expect("no frames available");
            self.entries[index].set(frame, EntryFlags::PRESENT | EntryFlags::WRITABLE);
            self.next_table_mut(index).unwrap().zero();
        }
        self.next_table_mut(index).unwrap()
    }
}

impl<L> Index<usize> for Table<L>
where
    L: TableLevel,
{
    type Output = Entry;

    fn index(&self, index: usize) -> &Entry {
        &self.entries[index]
    }
}

impl<L> IndexMut<usize> for Table<L>
where
    L: TableLevel,
{
    fn index_mut(&mut self, index: usize) -> &mut Entry {
        &mut self.entries[index]
    }
}

pub trait TableLevel {}

pub enum Level4 {}
#[allow(dead_code)]
pub enum Level3 {}
#[allow(dead_code)]
pub enum Level2 {}
pub enum Level1 {}

impl TableLevel for Level4 {}
impl TableLevel for Level3 {}
impl TableLevel for Level2 {}
impl TableLevel for Level1 {}

pub trait HierarchicalLevel: TableLevel {
    type NextLevel: TableLevel;
}

impl HierarchicalLevel for Level4 {
    type NextLevel = Level3;
}

impl HierarchicalLevel for Level3 {
    type NextLevel = Level2;
}

impl HierarchicalLevel for Level2 {
    type NextLevel = Level1;
}
