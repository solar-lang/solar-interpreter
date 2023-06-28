use hotel::HotelMap;
use solar_parser::ast::body::BodyItem;

use crate::{
    id::{IdItem, SSID},
    project::{GlobalModules, ProjectInfo},
};

use super::Type;

#[derive(Default)]
struct BuildinTypeID {
    bool: u8,

    int8: u8,
    int16: u8,
    int32: u8,
    int64: u8,

    uint8: u8,
    uint16: u8,
    uint32: u8,
    uint64: u8,

    float32: u8,
    float64: u8,
}

// Only the stdlibary is allowed to declare buildin types!
fn link_buildin_types(
    projects: &ProjectInfo,
    modules: &GlobalModules,
) -> (HotelMap<SSID, Type>, BuildinTypeID) {
    let mut tys = HotelMap::new();
    let mut ids = BuildinTypeID::default();

    // Find std library
    let stdpaths = modules
        .keys()
        .filter(|path| path.starts_with(&["std".to_string()]));
    for module in stdpaths {
        let std = modules.get(module).unwrap();

        for (fid, f) in std.files.iter().enumerate() {
            for (iid, item) in f.ast.items.iter().enumerate() {
                if let BodyItem::BuildinTypeDecl(item) = item {
                    assert!(
                        item.generic_symbols.is_none(),
                        "can't construct static type from buildin with generics"
                    );

                    let ssid: SSID = (
                        (module.to_vec(), fid as u16, IdItem::Type(iid as u16)),
                        Vec::new(),
                    );
                    let ty = Type {
                        module: module.to_vec(),
                        field_layout: Vec::new(),
                        size_in_bytes: 0,
                    };

                    let id = tys.insert(ssid, ty) as u8;

                    match item.name.value {
                        "bool" => ids.bool = id,
                        "int8" => ids.int8 = id,
                        "int16" => ids.int16 = id,
                        "int32" => ids.int32 = id,
                        "int64" => ids.int64 = id,
                        "uint8" => ids.uint8 = id,
                        "uint16" => ids.uint16 = id,
                        "uint32" => ids.uint32 = id,
                        "uint64" => ids.uint64 = id,
                        "float32" => ids.float32 = id,
                        "float64" => ids.float64 = id,
                        x => panic!("unrecognized buildin: {x}"),
                    }
                }
            }
        }
    }

    (tys, ids)
}
