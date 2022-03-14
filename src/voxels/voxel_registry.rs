use std::fs;

use glam::Vec4;
use multi_map::MultiMap;

type VoxelMap = MultiMap<u16, String, VoxelProfile>;

lazy_static! {
    static ref VOXELS: VoxelMap = load_voxels();
}

fn load_voxels() -> VoxelMap {
    let paths = fs::read_dir("./src/resources/voxel_profiles").unwrap();

    let mut map = MultiMap::new();

    map.insert(
        0,
        "Empty".to_string(),
        VoxelProfile {
            id: 0,
            name: "Empty".to_string(),
            color: Vec4::ZERO,
        },
    );

    let mut id: u16 = 1;
    for voxel_file in paths.into_iter() {
        let voxel_file = voxel_file.unwrap();
        let file_contents = fs::read_to_string(voxel_file.path()).unwrap();
        let json: serde_json::Value =
            serde_json::from_str(&file_contents).expect("JSON failed to parse");
        let color = decode_color(json.get("color").map_or("#ffff", |v| v.as_str().unwrap()));
        let name = voxel_file
            .file_name()
            .to_string_lossy()
            .replace(".json", "");

        let profile = VoxelProfile {
            name: name.clone(),
            id,
            color,
        };
        map.insert(id, name.clone(), profile);

        println!("==Created Voxel Profile==");
        println!("Name: {name}");
        println!("id: {id}");
        println!("color: {color}");
        println!("");

        id += 1;
    }

    return map;
}

fn decode_color(color_string: &str) -> Vec4 {
    let len = color_string.len() - 1; // -1 because of the hashtag at the front of the string
                                      // RGB
    if len == 3 {
        let r = u8::from_str_radix(&color_string.get(1..2).unwrap(), 16).unwrap() as f32 / 15.0;
        let g = u8::from_str_radix(&color_string.get(2..3).unwrap(), 16).unwrap() as f32 / 15.0;
        let b = u8::from_str_radix(&color_string.get(3..4).unwrap(), 16).unwrap() as f32 / 15.0;
        return Vec4::new(r, g, b, 1.0);
    }
    // RGBA
    if len == 4 {
        let r = u8::from_str_radix(&color_string.get(1..2).unwrap(), 16).unwrap() as f32 / 15.0;
        let g = u8::from_str_radix(&color_string.get(2..3).unwrap(), 16).unwrap() as f32 / 15.0;
        let b = u8::from_str_radix(&color_string.get(3..4).unwrap(), 16).unwrap() as f32 / 15.0;
        let a = u8::from_str_radix(&color_string.get(4..5).unwrap(), 16).unwrap() as f32 / 15.0;
        return Vec4::new(r, g, b, a);
    }
    // RRGGBB
    if len == 6 {
        let r = u8::from_str_radix(&color_string.get(1..3).unwrap(), 16).unwrap() as f32 / 255.0;
        let g = u8::from_str_radix(&color_string.get(3..5).unwrap(), 16).unwrap() as f32 / 255.0;
        let b = u8::from_str_radix(&color_string.get(5..7).unwrap(), 16).unwrap() as f32 / 255.0;
        return Vec4::new(r, g, b, 1.0);
    }
    // RRGGBBAA
    if len == 8 {
        let r = u8::from_str_radix(&color_string.get(1..3).unwrap(), 16).unwrap() as f32 / 255.0;
        let g = u8::from_str_radix(&color_string.get(3..5).unwrap(), 16).unwrap() as f32 / 255.0;
        let b = u8::from_str_radix(&color_string.get(5..7).unwrap(), 16).unwrap() as f32 / 255.0;
        let a = u8::from_str_radix(&color_string.get(7..9).unwrap(), 16).unwrap() as f32 / 255.0;
        return Vec4::new(r, g, b, a);
    }

    return Vec4::new(0.0, 0.0, 0.0, 1.0);
}

pub fn get_voxel_by_name(name: String) -> Option<&'static VoxelProfile> {
    return VOXELS.get_alt(&name).clone();
}

pub fn get_voxel_by_id(id: u16) -> Option<&'static VoxelProfile> {
    return VOXELS.get(&id);
}

#[derive(Clone)]
pub struct VoxelProfile {
    pub id: u16,
    pub name: String,
    pub color: Vec4,
}
