//! Virtualises a full 32-bit (4 GiB) address space using pages.

use std::{
    fmt::Debug,
    ops::{Index, IndexMut},
};

/// Virtualises a full 32-bit address space using pages.
/// The default value at every address is zero.
/// It doesn't try to reclaim memory that's reset to all-zeroes.
#[derive(Default)]
pub struct Memory {
    root: PageRoot,
}

impl Debug for Memory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<memory using {} x 4KiB pages>", self.count_pages())
    }
}

impl Memory {
    pub fn new(data: &[u32]) -> Self {
        let mut result = Memory {
            root: PageRoot::default(),
        };
        for (i, item) in data.iter().enumerate() {
            result.set_word_aligned(i as u32 * 4, *item);
        }
        result
    }

    /// Access the word at a word-aligned (4-byte aligned) address.
    pub fn get_word_aligned(&self, addr: u32) -> u32 {
        let (a, b, c, _) = to_indices(addr);
        self.root[a]
            .as_ref()
            .and_then(|dir| dir[b].as_ref().map(|table| table[c]))
            .unwrap_or_default()
    }

    pub fn set_word_aligned(&mut self, addr: u32, value: u32) {
        let (a, b, c, _) = to_indices(addr);
        self.root[a].get_or_insert_default()[b].get_or_insert_default()[c] = value;
    }

    /// Return the number of pages in use to represent the memory of this processor.
    pub fn count_pages(&self) -> usize {
        1 + self
            .root
            .entries
            .iter()
            .filter_map(Option::as_ref)
            .map(|dir| 1 + dir.entries.iter().filter_map(Option::as_ref).count())
            .sum::<usize>()
    }
}

type PageTable = Page<u32>;
type PageDir = Page<Option<Box<PageTable>>>;
type PageRoot = Page<Option<Box<PageDir>>>;

struct Page<T> {
    entries: [T; 1 << 10],
}

impl<T> Default for Page<T>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            entries: std::array::from_fn(|_| Default::default()),
        }
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct U10(u16);

impl<T> Index<U10> for Page<T> {
    type Output = T;

    fn index(&self, index: U10) -> &Self::Output {
        &self.entries[index.0 as usize]
    }
}

impl<T> IndexMut<U10> for Page<T> {
    fn index_mut(&mut self, index: U10) -> &mut Self::Output {
        &mut self.entries[index.0 as usize]
    }
}

/// Converts an address to its page indices, together with a final offset (either 0, 1, 2, or 3).
fn to_indices(addr: u32) -> (U10, U10, U10, u32) {
    (
        U10((addr >> 22) as u16),
        U10(((addr >> 12) & 0x3FF) as u16),
        U10(((addr >> 2) & 0x3FF) as u16),
        addr % 4,
    )
}
