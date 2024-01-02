use std::ops::Deref;

use hotel::HotelMap;

use crate::{compile::StaticExpression, id::SSID};

pub enum FunctionInfo {
    Partly,
    Complete(StaticExpression),
}

/// Container to hold information about compiled functions
#[derive(Default)]
pub struct FunctionStore {
    functions: HotelMap<SSID, FunctionInfo>,
}

impl FunctionStore {
    pub fn new() -> Self {
        FunctionStore::default()
    }

    /// Reserve a slot in the hotel map
    pub fn reserve(&mut self, key: SSID) -> usize {
        self.functions.insert(key, FunctionInfo::Partly)
    }

    pub fn update_complete_function(&mut self, index: usize, expr: StaticExpression) {
        self.functions
            .set_by_index(index, FunctionInfo::Complete(expr));
    }
}

impl Deref for FunctionStore {
    type Target = HotelMap<SSID, FunctionInfo>;

    fn deref(&self) -> &Self::Target {
        &self.functions
    }
}
