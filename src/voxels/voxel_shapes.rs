use glam::IVec3;
mod occlussion_shapes {
    const CUBE: [u8; 6] = [
        0b_1111_1111, // North
        0b_1111_1111, // South
        0b_1111_1111, // East
        0b_1111_1111, // West
        0b_1111_1111, // Top
        0b_1111_1111, // Bottom
    ];

    const STAIR: [u8; 6] = [
        0b_1111_1111, // North
        0b_1100_0011, // South
        0b_1111_0011, // East
        0b_1111_0011, // West
        0b_0011_1100, // Top
        0b_1111_1111, // Bottom
    ];

    const CORNER_STAIR: [u8; 6] = [
        0b_1111_1111, // North
        0b_1100_1111, // South
        0b_1111_1111, // East
        0b_1111_0011, // West
        0b_1111_1100, // Top
        0b_1111_1111, // Bottom
    ];

    const SLAB: [u8; 6] = [
        0b_1100_0011, // North
        0b_1100_0011, // South
        0b_1100_0011, // East
        0b_1100_0011, // West
        0b_0000_0000, // Top
        0b_1111_1111, // Bottom
    ];

    const INNER_PRISM_JUNCTION: [u8; 6] = [
        0b_1111_1111, // North
        0b_1000_0111, // South
        0b_1111_1111, // East
        0b_1110_0001, // West
        0b_0111_1000, // Top
        0b_1111_1111, // Bottom
    ];

    const INNER_CORNER_PRISM: [u8; 6] = [
        0b_1111_1111, // North
        0b_1000_0111, // South
        0b_1111_1111, // East
        0b_1110_0001, // West
        0b_0000_0000, // Top
        0b_1111_1111, // Bottom
    ];

    const OUTER_CORNER_PRISM: [u8; 6] = [
        0b_1000_0111, // North
        0b_0000_0000, // South
        0b_1110_0001, // East
        0b_0000_0000, // West
        0b_0000_0000, // Top
        0b_1111_1111, // Bottom
    ];

    const PRISM: [u8; 6] = [
        0b_1111_1111, // North
        0b_0000_0000, // South
        0b_1110_0001, // East
        0b_1110_0001, // West
        0b_0000_0000, // Top
        0b_1111_1111, // Bottom
    ];

    const SHAPES: [[u8; 6]; 8] = [
        CUBE,
        STAIR,
        CORNER_STAIR,
        SLAB,
        INNER_PRISM_JUNCTION,
        INNER_CORNER_PRISM,
        OUTER_CORNER_PRISM,
        PRISM,
    ];

    lazy_static! {
        pub static ref SHAPE_ORIENTATIONS: Box<[u8; 1536]> = Box::new(get_shape_permutations());
    }

    fn get_shape_permutations() -> [u8; 1536] {
        let mut r = [0; 1536];

        for i in 0_u8..255 {
            let mut shape = SHAPES[(i << 5 >> 5) as usize]; // 32 rotations per shape

            // Iterate through all possible permutations of the orientation, assigning it to a NOT RANDOM spot in the orientation array
            if i & 0b_0000_1000 != 0 {
                shape = flip_east_west(shape);
            }
            if i & 0b_0001_0000 != 0 {
                shape = flip_top_bottom(shape);
            }
            if i & 0b_0010_0000 != 0 {
                shape = flip_north_south(shape);
            }
            if i & 0b_0100_0000 != 0 {
                shape = rotate_x(shape);
            }
            if i & 0b_1000_0000 != 0 {
                shape = rotate_z(shape);
            }
            for b in 0..6 {
                r[((i as usize) * 6) + b] = shape[b];
            }
        }

        r
    }

    fn flip_north_south(sides: [u8; 6]) -> [u8; 6] {
        let mut r = [0; 6];
        r[0] = sides[1].reverse_bits(); // North
        r[1] = sides[0].reverse_bits(); // South
        r[2] = sides[2].reverse_bits(); // East
        r[3] = sides[3].reverse_bits(); // West
        r[4] = sides[4].reverse_bits().rotate_left(4); // Top
        r[5] = sides[5].reverse_bits().rotate_left(4); // Bottom
        r
    }

    fn flip_east_west(sides: [u8; 6]) -> [u8; 6] {
        let mut r = [0; 6];
        r[0] = sides[1].reverse_bits(); // North
        r[1] = sides[0].reverse_bits(); // South
        r[2] = sides[3].reverse_bits(); // East
        r[3] = sides[2].reverse_bits(); // West
        r[4] = sides[4].reverse_bits(); // Top
        r[5] = sides[5].reverse_bits(); // Bottom
        r
    }

    fn flip_top_bottom(sides: [u8; 6]) -> [u8; 6] {
        let mut r = [0; 6];
        r[0] = sides[0].reverse_bits().rotate_left(4); // North
        r[1] = sides[1].reverse_bits().rotate_left(4); // South
        r[2] = sides[2].reverse_bits().rotate_left(4); // East
        r[3] = sides[3].reverse_bits().rotate_left(4); // West
        r[4] = sides[5].reverse_bits(); // Top
        r[5] = sides[4].reverse_bits(); // Bottom
        r
    }

    fn rotate_x(sides: [u8; 6]) -> [u8; 6] {
        let mut r = [0; 6];
        r[0] = sides[5]; // North
        r[1] = sides[4]; // South
        r[2] = sides[2].rotate_right(2); // East
        r[3] = sides[3].rotate_right(2); // West
        r[4] = sides[0]; // Top
        r[5] = sides[1]; // Bottom
        r
    }

    fn rotate_z(sides: [u8; 6]) -> [u8; 6] {
        let mut r = [0; 6];
        r[0] = sides[0].rotate_right(2); // North
        r[1] = sides[1].rotate_right(2); // South
        r[2] = sides[4]; // East
        r[3] = sides[5]; // West
        r[4] = sides[2]; // Top
        r[5] = sides[3]; // Bottom
        r
    }
}

pub struct OrientedVoxelDirections {
    pub directions: [VoxelDirection; 6],
}

impl OrientedVoxelDirections {
    pub fn new(directions: [VoxelDirection; 6]) -> Self {
        Self { directions }
    }
    pub fn get_direction(&self, direction: VoxelDirection) -> VoxelDirection {
        self.directions[direction.data as usize]
    }
}

#[allow(dead_code)]
#[rustfmt::skip]
pub mod voxel_directions {
    use super::VoxelDirection;

    pub const NORTH: VoxelDirection     = VoxelDirection { data: 0 };
    pub const SOUTH: VoxelDirection     = VoxelDirection { data: 1 };
    pub const EAST: VoxelDirection      = VoxelDirection { data: 2 };
    pub const WEST: VoxelDirection      = VoxelDirection { data: 3 };
    pub const UP: VoxelDirection        = VoxelDirection { data: 4 };
    pub const DOWN: VoxelDirection      = VoxelDirection { data: 5 };

    pub const ALL: [VoxelDirection; 6] = [
        NORTH,
        SOUTH,
        EAST,
        WEST,
        UP,
        DOWN,
    ];
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Default)]
#[repr(packed(1))]
pub struct VoxelDirection {
    pub data: u8,
}

lazy_static! {
    static ref VEC_MAPPING: [IVec3; 6] = [
        IVec3::Z,
        -IVec3::Z,
        IVec3::X,
        -IVec3::X,
        IVec3::Y,
        -IVec3::Y,
    ];
}

impl VoxelDirection {
    pub fn as_vec(&self) -> IVec3 {
        VEC_MAPPING[self.data as usize]
    }

    pub fn flip(&self) -> VoxelDirection {
        VoxelDirection {
            data: if self.data & 1 == 0 {
                self.data + 1
            } else {
                self.data - 1
            },
        }
    }

    pub fn get_oriented_directions(orientation: VoxelOrientation) -> OrientedVoxelDirections {
        let mut r = voxel_directions::ALL;
        if orientation.extract_rotate_z() {
            (r[2], r[3], r[4], r[5]) = (r[5], r[4], r[2], r[3]);
        }
        if orientation.extract_rotate_x() {
            (r[0], r[1], r[4], r[5]) = (r[4], r[5], r[1], r[0]);
        }
        if orientation.extract_flip_z() {
            (r[0], r[1]) = (r[1], r[0]);
        }
        if orientation.extract_flip_y() {
            (r[4], r[5]) = (r[5], r[4]);
        }
        if orientation.extract_flip_x() {
            (r[2], r[3]) = (r[3], r[2]);
        }
        OrientedVoxelDirections::new(r)
    }
}

#[cfg(test)]
mod direction_tests {
    use glam::IVec3;

    use crate::voxels::voxel_shapes::voxel_directions;

    #[test]
    fn flip_test() {
        assert_eq!(voxel_directions::NORTH.flip(), voxel_directions::SOUTH);
        assert_eq!(voxel_directions::SOUTH.flip(), voxel_directions::NORTH);
        assert_eq!(voxel_directions::EAST.flip(), voxel_directions::WEST);
        assert_eq!(voxel_directions::WEST.flip(), voxel_directions::EAST);
        assert_eq!(voxel_directions::UP.flip(), voxel_directions::DOWN);
        assert_eq!(voxel_directions::DOWN.flip(), voxel_directions::UP);
    }

    #[test]
    fn as_vec_test() {
        assert_eq!(voxel_directions::NORTH.as_vec(), IVec3::Z);
        assert_eq!(voxel_directions::SOUTH.as_vec(), -IVec3::Z);
        assert_eq!(voxel_directions::EAST.as_vec(), IVec3::X);
        assert_eq!(voxel_directions::WEST.as_vec(), -IVec3::X);
        assert_eq!(voxel_directions::UP.as_vec(), IVec3::Y);
        assert_eq!(voxel_directions::DOWN.as_vec(), -IVec3::Y);
    }
}

#[allow(dead_code)]
#[rustfmt::skip]
pub mod voxel_shape {
    use super::VoxelShape;

    pub const CUBE: VoxelShape                  = VoxelShape { data: 0 };
    pub const STAIR: VoxelShape                 = VoxelShape { data: 1 };
    pub const CORNER_STAIR: VoxelShape          = VoxelShape { data: 2 };
    pub const SLAB: VoxelShape                  = VoxelShape { data: 3 };
    pub const INNER_PRISM_JUNCTION: VoxelShape  = VoxelShape { data: 4 };
    pub const INNER_CORNER_PRISM: VoxelShape    = VoxelShape { data: 5 };
    pub const OUTER_CORNER_PRISM: VoxelShape    = VoxelShape { data: 6 };
    pub const PRISM: VoxelShape                 = VoxelShape { data: 7 };
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Default)]
#[repr(packed(1))]
// TODO: Refactor this to just wrap the u8 type
pub struct VoxelShape {
    pub data: u8,
}

// [7] Rotate Z
// [6] Rotate X
// [5] Flip Y
// [4] Flip X
// [3] Flip Z
// [2, 1, 0] Index of shape

impl VoxelShape {
    pub fn get_face_shape(shape: VoxelShape, direction: VoxelDirection) -> u8 {
        occlussion_shapes::SHAPE_ORIENTATIONS
            [(((shape.data as u32) * 6) + direction.data as u32) as usize]
    }

    pub fn face_contains(&self, face: VoxelDirection, other: (VoxelShape, VoxelDirection)) -> bool {
        let other_shape = VoxelShape::get_face_shape(other.0, other.1);
        VoxelShape::get_face_shape(*self, face) & other_shape == other_shape
    }

    pub fn orient_self(&mut self, orientation: VoxelOrientation) {
        self.data = (self.data & 0b_0000_0111) | orientation.data;
    }

    pub fn oriented(&self, orientation: VoxelOrientation) -> VoxelShape {
        VoxelShape {
            data: (self.data & 0b_0000_0111) | orientation.data,
        }
    }

    pub fn extract_shape(&self) -> u8 {
        self.data << 5 >> 5
    }

    pub fn extract_flip_x(&self) -> bool {
        self.data & 0b_0000_1000 == 0b_0000_1000
    }
    pub fn extract_flip_y(&self) -> bool {
        self.data & 0b_0001_0000 == 0b_0001_0000
    }
    pub fn extract_flip_z(&self) -> bool {
        self.data & 0b_0010_0000 == 0b_0010_0000
    }
    pub fn extract_rotate_x(&self) -> bool {
        self.data & 0b_0100_0000 == 0b_0100_0000
    }
    pub fn extract_rotate_z(&self) -> bool {
        self.data & 0b_1000_0000 == 0b_1000_0000
    }

    pub fn extract_orientation(&self) -> VoxelOrientation {
        VoxelOrientation {
            data: self.data & 0b_1111_1000,
        }
    }
}

#[allow(dead_code)]
#[rustfmt::skip]
pub mod voxel_orientations {
    use super::VoxelOrientation;

    pub const DEFAULT: VoxelOrientation = VoxelOrientation { data: 0b_00_000_000 };
    pub const BOTTOM: VoxelOrientation = VoxelOrientation { data: 0b_00_000_000 };
    pub const BOTTOM_NORTH: VoxelOrientation = VoxelOrientation { data: 0b_00_000_000 };
    pub const BOTTOM_NORTH_EAST: VoxelOrientation = VoxelOrientation { data: 0b_00_000_000 };

    pub const BOTTOM_NORTH_WEST: VoxelOrientation = VoxelOrientation { data: 0b_00_001_000 };
    
    pub const TOP: VoxelOrientation = VoxelOrientation { data: 0b_00_010_000 };
    pub const TOP_NORTH: VoxelOrientation = VoxelOrientation { data: 0b_00_010_000 };
    pub const TOP_NORTH_EAST: VoxelOrientation = VoxelOrientation { data: 0b_00_010_000 };
    
    pub const TOP_NORTH_WEST: VoxelOrientation = VoxelOrientation { data: 0b_00_011_000 };   
    
    pub const TOP_SOUTH: VoxelOrientation = VoxelOrientation { data: 0b_00_110_000 };
    pub const TOP_SOUTH_EAST: VoxelOrientation = VoxelOrientation { data: 0b_00_110_000 };
    
    pub const TOP_SOUTH_WEST: VoxelOrientation = VoxelOrientation { data: 0b_00_111_000 };
    
    pub const BOTTOM_SOUTH: VoxelOrientation = VoxelOrientation { data: 0b_00_100_000 };
    pub const BOTTOM_SOUTH_EAST: VoxelOrientation = VoxelOrientation { data: 0b_00_100_000 };
    
    pub const BOTTOM_SOUTH_WEST: VoxelOrientation = VoxelOrientation { data: 0b_00_101_000 };
    
    pub const NORTH: VoxelOrientation = VoxelOrientation { data: 0b_01_000_000 };
    pub const NORTH_TOP: VoxelOrientation = VoxelOrientation { data: 0b_01_000_000 };
    pub const NORTH_TOP_EAST: VoxelOrientation = VoxelOrientation { data: 0b_01_000_000 };
    
    pub const NORTH_TOP_WEST: VoxelOrientation = VoxelOrientation { data: 0b_01_001_000 };
    
    pub const SOUTH: VoxelOrientation = VoxelOrientation { data: 0b_01_010_000 };
    pub const SOUTH_TOP: VoxelOrientation = VoxelOrientation { data: 0b_01_010_000 };
    pub const SOUTH_TOP_EAST: VoxelOrientation = VoxelOrientation { data: 0b_01_010_000 };
    
    pub const SOUTH_TOP_WEST: VoxelOrientation = VoxelOrientation { data: 0b_01_011_000 };
    
    pub const NORTH_BOTTOM: VoxelOrientation = VoxelOrientation { data: 0b_01_100_000 };
    pub const NORTH_BOTTOM_EAST: VoxelOrientation = VoxelOrientation { data: 0b_01_100_000 };
    
    pub const NORTH_BOTTOM_WEST: VoxelOrientation = VoxelOrientation { data: 0b_01_101_000 };
    
    pub const SOUTH_BOTTOM_WEST: VoxelOrientation = VoxelOrientation { data: 0b_01_111_000 };
    
    pub const WEST: VoxelOrientation = VoxelOrientation { data: 0b_10_000_000 };
    pub const WEST_NORTH: VoxelOrientation = VoxelOrientation { data: 0b_10_000_000 };
    pub const WEST_NORTH_BOTTOM: VoxelOrientation = VoxelOrientation { data: 0b_10_000_000 };
    
    pub const WEST_NORTH_TOP: VoxelOrientation = VoxelOrientation { data: 0b_10_001_000 };
    
    pub const EAST: VoxelOrientation = VoxelOrientation { data: 0b_10_010_000 };
    pub const EAST_NORTH: VoxelOrientation = VoxelOrientation { data: 0b_10_010_000 };
    pub const EAST_NORTH_BOTTOM: VoxelOrientation = VoxelOrientation { data: 0b_10_010_000 };
    
    pub const EAST_NORTH_TOP: VoxelOrientation = VoxelOrientation { data: 0b_10_011_000 };
    
    pub const EAST_SOUTH: VoxelOrientation = VoxelOrientation { data: 0b_10_100_000 };
    pub const EAST_SOUTH_BOTTOM: VoxelOrientation = VoxelOrientation { data: 0b_10_100_000 };
    
    pub const WEST_SOUTH: VoxelOrientation = VoxelOrientation { data: 0b_10_101_000 };
    pub const WEST_SOUTH_TOP: VoxelOrientation = VoxelOrientation { data: 0b_10_101_000 };
    
    pub const WEST_SOUTH_BOTTOM: VoxelOrientation = VoxelOrientation { data: 0b_10_110_000 };
    
    pub const EAST_SOUTH_TOP: VoxelOrientation = VoxelOrientation { data: 0b_10_111_000 };
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Default)]
#[repr(packed(1))]
pub struct VoxelOrientation {
    pub data: u8,
}

impl VoxelOrientation {
    pub fn extract_flip_x(&self) -> bool {
        self.data & 0b_0000_1000 == 0b_0000_1000
    }
    pub fn extract_flip_y(&self) -> bool {
        self.data & 0b_0001_0000 == 0b_0001_0000
    }
    pub fn extract_flip_z(&self) -> bool {
        self.data & 0b_0010_0000 == 0b_0010_0000
    }
    pub fn extract_rotate_x(&self) -> bool {
        self.data & 0b_0100_0000 == 0b_0100_0000
    }
    pub fn extract_rotate_z(&self) -> bool {
        self.data & 0b_1000_0000 == 0b_1000_0000
    }
}
