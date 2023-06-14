use crate::util::IdPath;


pub type SymbolId = (IdModule, IdFile, IdItem);

pub type IdModule = IdPath;
pub type IdFile = u32;
pub type IdItem = u32;