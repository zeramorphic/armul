//! Virtualises a full 32-bit (4 GiB) address space using pages.

use std::{
    fmt::Debug,
    ops::{Index, IndexMut},
};

/// Virtualises a full 32-bit address space using pages.
/// It doesn't try to reclaim memory that's reset to the default value.
/// We emulate a little-endian architecture.
pub struct Memory {
    root: PageRoot,
    default_word: u32,
}

impl Debug for Memory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<memory using {} x 4KiB pages>", self.count_pages())
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self::new(0xAAAAAAAA)
    }
}

impl Memory {
    pub fn new(default_word: u32) -> Self {
        Memory {
            root: Default::default(),
            default_word,
        }
    }

    /// Access the word at a word-aligned (4-byte aligned) address.
    pub fn get_word_aligned(&self, addr: u32) -> u32 {
        let (a, b, c, _) = to_indices(addr);
        self.root[a]
            .as_ref()
            .and_then(|dir| dir[b].as_ref().map(|table| table[c]))
            .unwrap_or(self.default_word)
    }

    pub fn get_words_aligned(&self, addr: u32, result: &mut [u32]) {
        for (offset, value) in result.iter_mut().enumerate() {
            *value = self.get_word_aligned(addr.wrapping_add(4 * offset as u32))
        }
    }

    pub fn get_byte(&self, addr: u32) -> u8 {
        self.get_word_aligned(addr >> 2 << 2).to_le_bytes()[addr as usize % 4]
    }

    pub fn set_word_aligned(&mut self, addr: u32, value: u32) {
        let (a, b, c, _) = to_indices(addr);
        self.root[a].get_or_insert_default()[b].get_or_insert_with(|| {
            Box::new(Page {
                entries: std::array::from_fn(|_| self.default_word),
            })
        })[c] = value;
    }

    pub fn set_words_aligned(&mut self, addr: u32, values: &[u32]) {
        for (offset, value) in values.iter().enumerate() {
            self.set_word_aligned(addr.wrapping_add(4 * offset as u32), *value);
        }
    }

    pub fn set_byte(&mut self, addr: u32, value: u8) {
        let (a, b, c, d) = to_indices(addr);
        let location = &mut self.root[a].get_or_insert_default()[b].get_or_insert_with(|| {
            Box::new(Page {
                entries: std::array::from_fn(|_| self.default_word),
            })
        })[c];
        let mut bytes = location.to_le_bytes();
        bytes[d as usize] = value;
        *location = u32::from_le_bytes(bytes)
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
#[inline]
fn to_indices(addr: u32) -> (U10, U10, U10, u32) {
    (
        U10((addr >> 22) as u16),
        U10(((addr >> 12) & 0x3FF) as u16),
        U10(((addr >> 2) & 0x3FF) as u16),
        addr % 4,
    )
}
