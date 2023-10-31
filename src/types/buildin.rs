use hotel::HotelMap;
use solar_parser::ast::body::BodyItem;

use crate::{
    id::{IdItem, SSID},
    project::GlobalModules,
};

use super::Type;

#[derive(Default, Debug)]
pub struct BuildinTypeID {
    bool: u8,

    int8: u8,
    int16: u8,
    int32: u8,
    int: u8,

    uint8: u8,
    uint16: u8,
    uint32: u8,
    uint: u8,

    float32: u8,
    float: u8,
}

// Only the stdlibary is allowed to declare buildin types!
pub fn link_buildin_types(modules: &GlobalModules) -> (HotelMap<SSID, Type>, BuildinTypeID) {
    let mut tys = HotelMap::new();
    let mut ids = BuildinTypeID::default();

    // Find std library
    let stdpaths = modules
        .keys()
        .filter(|path| path.starts_with(&["std(solar-lang)".to_string()]));
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
                        "Bool" => ids.bool = id,
                        "Int8" => ids.int8 = id,
                        "Int16" => ids.int16 = id,
                        "Int32" => ids.int32 = id,
                        "Int" => ids.int = id,
                        "Uint8" => ids.uint8 = id,
                        "Uint16" => ids.uint16 = id,
                        "Uint32" => ids.uint32 = id,
                        "Uint" => ids.uint = id,
                        "Float32" => ids.float32 = id,
                        "Float" => ids.float = id,
                        x => panic!("unrecognized buildin: {x}"),
                    }
                }
            }
        }
    }

    (tys, ids)
}
