use std::cell::UnsafeCell;
use std::sync::Arc;

use crate::board::{Coordinate, PieceType};

const TT_SIZE: usize = 10_000_000;

#[repr(u8)]
#[derive(Debug, PartialEq)]
pub enum NodeType {
    PV,
    Cut,
    All,
}

impl From<u8> for NodeType {
    fn from(nt: u8) -> Self {
        match nt {
            0 => NodeType::PV,
            1 => NodeType::Cut,
            2 => NodeType::All,
            _ => panic!("Invalid node type"),
        }
    }
}

bitfield! {
  #[derive(Clone, Copy)]
  pub struct TranspositionTableEntryMoveData(u64);

  impl Debug;
  pub u8, into Coordinate, best_move_src, set_best_move_src: 7, 0;
  pub u8, into Coordinate, best_move_dest, set_best_move_dest: 15, 8;
  // Promotion piece type
  pub u8, into PieceType, best_move_ppt, set_best_move_ppt: 23, 16;
  pub best_move_is_promotion, set_best_move_is_promotion: 24;
}

impl TranspositionTableEntryMoveData {
    pub fn new(move_src: Coordinate, move_dest: Coordinate, ppt: Option<PieceType>) -> Self {
        let mut d = TranspositionTableEntryMoveData(0);
        d.set_best_move_src(move_src as u8);
        d.set_best_move_dest(move_dest as u8);
        match ppt {
            Some(ppt) => {
                d.set_best_move_is_promotion(true);
                d.set_best_move_ppt(ppt as u8);
            }
            None => {
                d.set_best_move_is_promotion(false);
            }
        }
        d
    }
}

bitfield! {
  #[derive(Clone, Copy)]
  pub struct TranspositionTableEntrySearchData(u64);

  impl Debug;

  pub i32, score, set_score: 31, 0;
  pub u8, depth, set_depth: 39, 32;
  pub u8, into NodeType, node_type, set_node_type: 41, 40;
}

#[derive(Clone, Copy, Debug)]
pub struct TranspositionTableEntry(
    u64,
    TranspositionTableEntryMoveData,
    TranspositionTableEntrySearchData,
);

impl TranspositionTableEntry {
    pub fn new(
        key: u64,
        move_data: TranspositionTableEntryMoveData,
        search_data: TranspositionTableEntrySearchData,
    ) -> Self {
        Self(key ^ move_data.0 ^ search_data.0, move_data, search_data)
    }

    pub fn is_valid(&self, key: u64) -> bool {
        self.get_key() == key
    }

    pub fn get_key(&self) -> u64 {
        self.0 ^ self.1 .0 ^ self.2 .0
    }

    pub fn get_move_data(&self) -> TranspositionTableEntryMoveData {
        self.1
    }

    pub fn get_search_data(&self) -> TranspositionTableEntrySearchData {
        self.2
    }
}

#[derive(Clone, Debug)]
pub struct TranspositionTable {
    pub data: Arc<UnsafeCell<[TranspositionTableEntry; TT_SIZE]>>,
}

unsafe impl Sync for TranspositionTable {}
unsafe impl Send for TranspositionTable {}

pub fn build_new_tt() -> TranspositionTable {
    TranspositionTable {
        data: Arc::new(UnsafeCell::new(
            [TranspositionTableEntry(
                0,
                TranspositionTableEntryMoveData(0),
                TranspositionTableEntrySearchData(0),
            ); TT_SIZE],
        )),
    }
}

pub fn get_tt_index(key: u64) -> usize {
    key as usize % TT_SIZE
}

impl TranspositionTable {
    pub fn get_entry(&self, key: u64) -> TranspositionTableEntry {
        let index = get_tt_index(key);
        unsafe { (*self.data.get())[index] }
    }

    pub fn set_entry(&self, key: u64, entry: TranspositionTableEntry) {
        let index = get_tt_index(key);
        unsafe {
            (*self.data.get())[index] = entry;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_tt_entry_validity() {
        let key: u64 = 0b11110110110101101011;
        let move_data = TranspositionTableEntryMoveData(0b1010101011110);
        let search_data = TranspositionTableEntrySearchData(0b111110110);
        let entry = TranspositionTableEntry::new(key, move_data, search_data);
        assert!(entry.is_valid(key));
        assert!(!entry.is_valid(key + 1));
    }

    #[test]
    fn test_move_data_with_promotion() {
        let move_data = TranspositionTableEntryMoveData::new(
            Coordinate::E2,
            Coordinate::E4,
            Some(PieceType::Knight),
        );
        assert_eq!(move_data.best_move_src(), Coordinate::E2);
        assert_eq!(move_data.best_move_dest(), Coordinate::E4);
        assert_eq!(move_data.best_move_is_promotion(), true);
        assert_eq!(move_data.best_move_ppt(), PieceType::Knight);
    }

    #[test]
    fn test_move_data_without_promotion() {
        let move_data = TranspositionTableEntryMoveData::new(Coordinate::G1, Coordinate::F3, None);
        assert_eq!(move_data.best_move_src(), Coordinate::G1);
        assert_eq!(move_data.best_move_dest(), Coordinate::F3);
        assert_eq!(move_data.best_move_is_promotion(), false);
    }
}
