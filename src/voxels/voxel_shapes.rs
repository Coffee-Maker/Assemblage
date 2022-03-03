use glam::IVec3;

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
    0b_1100_1111, // East
    0b_1111_0011, // West
    0b_0011_1100, // Top
    0b_1111_1111, // Bottom
];

const CORNER_STAIR: [u8; 6] = [
    0b_1111_1111, // North
    0b_1100_1111, // South
    0b_1111_1111, // East
    0b_1111_0011, // West
    0b_0011_1111, // Top
    0b_1111_1111, // Bottom
];

const SLAB: [u8; 6] = [
    0b_1100_0011, // North
    0b_1100_0011, // South
    0b_1100_0011, // East
    0b_1100_0011, // West
    0b_1100_0011, // Top
    0b_1100_0011, // Bottom
];

const INNER_PRISM_JUNCTION: [u8; 6] = [
    0b_1111_1111, // North
    0b_1000_0111, // South
    0b_1111_1111, // East
    0b_1110_0001, // West
    0b_0001_1110, // Top
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
    0b_1110_0001, // North
    0b_0000_0000, // South
    0b_1000_0111, // East
    0b_0000_0000, // West
    0b_0000_0000, // Top
    0b_1111_1111, // Bottom
];

const PRISM: [u8; 6] = [
    0b_1111_1111, // North
    0b_0000_0000, // South
    0b_1000_0111, // East
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

        let rotation = i >> 3;
        if rotation & 0b_0000_0001 != 0 {
            shape = flip_north_south(shape);
        }
        if rotation & 0b_0000_0010 != 0 {
            shape = flip_east_west(shape);
        }
        if rotation & 0b_0000_0100 != 0 {
            shape = flip_top_bottom(shape);
        }
        if rotation & 0b_0000_1000 != 0 {
            shape = rotate_x(shape);
        }
        if rotation & 0b_0001_0000 != 0 {
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
    r[0] = sides[4]; // North
    r[1] = sides[5]; // South
    r[2] = sides[2].rotate_right(2); // East
    r[3] = sides[2].rotate_right(2); // West
    r[4] = sides[1]; // Top
    r[5] = sides[0]; // Bottom
    r
}

fn rotate_z(sides: [u8; 6]) -> [u8; 6] {
    let mut r = [0; 6];
    r[0] = sides[0].rotate_right(2); // North
    r[1] = sides[1].rotate_right(2); // South
    r[2] = sides[5]; // East
    r[3] = sides[4]; // West
    r[4] = sides[2]; // Top
    r[5] = sides[3]; // Bottom
    r
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
}

#[allow(dead_code)]
#[rustfmt::skip]
pub mod voxel_shapes {
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
pub struct VoxelShape {
    data: u8,
}

impl VoxelShape {
    pub fn get_face_shape(shape: VoxelShape, direction: VoxelDirection) -> u8 {
        SHAPE_ORIENTATIONS[(shape.data + direction.data) as usize]
    }

    pub fn face_contains(&self, face: VoxelDirection, other: (VoxelShape, VoxelDirection)) -> bool {
        let other_shape = VoxelShape::get_face_shape(other.0, other.1);
        VoxelShape::get_face_shape(*self, face) & other_shape == other_shape
    }

    pub fn orient(&mut self, orientation: VoxelOrientation) {
        self.data = (self.data & 0b_0000_0111) | orientation.data;
    }
}

#[allow(dead_code)]
#[rustfmt::skip]
pub mod voxel_orientations {
    use super::VoxelOrientation;

    pub const DEFAULT: VoxelOrientation = VoxelOrientation { data: 0b_00_000 };
    pub const BOTTOM: VoxelOrientation = VoxelOrientation { data: 0b_00_000 };
    pub const BOTTOM_NORTH: VoxelOrientation = VoxelOrientation { data: 0b_00_000 };
    pub const BOTTOM_NORTH_EAST: VoxelOrientation = VoxelOrientation { data: 0b_00_000 };

    pub const BOTTOM_NORTH_WEST: VoxelOrientation = VoxelOrientation { data: 0b_00_001 };

    pub const BOTTOM_SOUTH: VoxelOrientation = VoxelOrientation { data: 0b_00_010 };
    pub const BOTTOM_SOUTH_EAST: VoxelOrientation = VoxelOrientation { data: 0b_00_010 };

    pub const BOTTOM_SOUTH_WEST: VoxelOrientation = VoxelOrientation { data: 0b_00_011 };

    pub const TOP: VoxelOrientation = VoxelOrientation { data: 0b_00_100 };
    pub const TOP_NORTH: VoxelOrientation = VoxelOrientation { data: 0b_00_100 };
    pub const TOP_NORTH_EAST: VoxelOrientation = VoxelOrientation { data: 0b_00_100 };

    pub const TOP_NORTH_WEST: VoxelOrientation = VoxelOrientation { data: 0b_00_101 };

    pub const TOP_SOUTH: VoxelOrientation = VoxelOrientation { data: 0b_00_110 };
    pub const TOP_SOUTH_EAST: VoxelOrientation = VoxelOrientation { data: 0b_00_110 };

    pub const TOP_SOUTH_WEST: VoxelOrientation = VoxelOrientation { data: 0b_00_111 };

    pub const NORTH: VoxelOrientation = VoxelOrientation { data: 0b_01_000 };
    pub const NORTH_TOP: VoxelOrientation = VoxelOrientation { data: 0b_01_000 };
    pub const NORTH_TOP_EAST: VoxelOrientation = VoxelOrientation { data: 0b_01_000 };

    pub const NORTH_TOP_WEST: VoxelOrientation = VoxelOrientation { data: 0b_01_001 };

    pub const SOUTH: VoxelOrientation = VoxelOrientation { data: 0b_01_010 };
    pub const SOUTH_BOTTOM: VoxelOrientation = VoxelOrientation { data: 0b_01_010 };
    pub const SOUTH_BOTTOM_EAST: VoxelOrientation = VoxelOrientation { data: 0b_01_010 };

    pub const SOUTH_TOP_WEST: VoxelOrientation = VoxelOrientation { data: 0b_01_011 };

    pub const NORTH_BOTTOM_EAST: VoxelOrientation = VoxelOrientation { data: 0b_01_100 };
    pub const NORTH_BOTTOM: VoxelOrientation = VoxelOrientation { data: 0b_01_100 };

    pub const NORTH_BOTTOM_WEST: VoxelOrientation = VoxelOrientation { data: 0b_01_101 };

    pub const SOUTH_BOTTOM_WEST: VoxelOrientation = VoxelOrientation { data: 0b_01_111 };

    pub const WEST: VoxelOrientation = VoxelOrientation { data: 0b_10_000 };
    pub const WEST_NORTH: VoxelOrientation = VoxelOrientation { data: 0b_10_000 };
    pub const WEST_NORTH_BOTTOM: VoxelOrientation = VoxelOrientation { data: 0b_10_000 };

    pub const WEST_NORTH_TOP: VoxelOrientation = VoxelOrientation { data: 0b_10_001 };

    pub const EAST: VoxelOrientation = VoxelOrientation { data: 0b_10_010 };
    pub const EAST_NORTH: VoxelOrientation = VoxelOrientation { data: 0b_10_010 };
    pub const EAST_NORTH_BOTTOM: VoxelOrientation = VoxelOrientation { data: 0b_10_010 };

    pub const EAST_NORTH_TOP: VoxelOrientation = VoxelOrientation { data: 0b_10_011 };

    pub const EAST_SOUTH: VoxelOrientation = VoxelOrientation { data: 0b_10_100 };
    pub const EAST_SOUTH_BOTTOM: VoxelOrientation = VoxelOrientation { data: 0b_10_100 };

    pub const WEST_SOUTH: VoxelOrientation = VoxelOrientation { data: 0b_10_101 };
    pub const WEST_SOUTH_TOP: VoxelOrientation = VoxelOrientation { data: 0b_10_101 };

    pub const WEST_SOUTH_BOTTOM: VoxelOrientation = VoxelOrientation { data: 0b_10_110 };

    pub const EAST_SOUTH_TOP: VoxelOrientation = VoxelOrientation { data: 0b_10_111 };
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Default)]
#[repr(packed(1))]
pub struct VoxelOrientation {
    pub data: u8,
}
