#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Default)]
pub struct VoxelShape {
    data: u8,
}

#[allow(dead_code)]
#[rustfmt::skip]
pub mod voxel_shapes {
    use super::VoxelShape;

    pub const EMPTY: VoxelShape             = VoxelShape { data: 0b_0000_0000 };
    pub const ALL: VoxelShape               = VoxelShape { data: 0b_1111_1111 };

    pub const BOTTOM_SOUTH_WEST: VoxelShape = VoxelShape { data: 0b_0000_0001 };
    pub const BOTTOM_NORTH_WEST: VoxelShape = VoxelShape { data: 0b_0000_0010 };
    pub const BOTTOM_NORTH_EAST: VoxelShape = VoxelShape { data: 0b_0000_0100 };
    pub const BOTTOM_SOUTH_EAST: VoxelShape = VoxelShape { data: 0b_0000_1000 };

    pub const TOP_SOUTH_WEST: VoxelShape    = VoxelShape { data: 0b_0001_0000 };
    pub const TOP_NORTH_WEST: VoxelShape    = VoxelShape { data: 0b_0010_0000 };
    pub const TOP_NORTH_EAST: VoxelShape    = VoxelShape { data: 0b_0100_0000 };
    pub const TOP_SOUTH_EAST: VoxelShape    = VoxelShape { data: 0b_1000_0000 };

    pub const BOTTOM_WEST: VoxelShape       = VoxelShape { data: 0b_0000_0011 };
    pub const BOTTOM_NORTH: VoxelShape      = VoxelShape { data: 0b_0000_0110 };
    pub const BOTTOM_EAST: VoxelShape       = VoxelShape { data: 0b_0000_1100 };
    pub const BOTTOM_SOUTH: VoxelShape      = VoxelShape { data: 0b_0000_1001 };

    pub const TOP_WEST: VoxelShape          = VoxelShape { data: 0b_0011_0000 };
    pub const TOP_NORTH: VoxelShape         = VoxelShape { data: 0b_0110_0000 };
    pub const TOP_EAST: VoxelShape          = VoxelShape { data: 0b_1100_0000 };
    pub const TOP_SOUTH: VoxelShape         = VoxelShape { data: 0b_1001_0000 };

    pub const WEST: VoxelShape              = VoxelShape { data: 0b_0011_0011 };
    pub const NORTH: VoxelShape             = VoxelShape { data: 0b_0110_0110 };
    pub const EAST: VoxelShape              = VoxelShape { data: 0b_1100_1100 };
    pub const SOUTH: VoxelShape             = VoxelShape { data: 0b_1001_1001 };

    pub const BOTTOM: VoxelShape            = VoxelShape { data: 0b_0000_1111 };
    pub const TOP: VoxelShape               = VoxelShape { data: 0b_1111_0000 };
}

impl VoxelShape {
    pub fn contains(&self, shape: VoxelShape) -> bool {
        self.data & shape.data == shape.data
    }

    pub fn overlaps(&self, shape: VoxelShape) -> bool {
        self.data & shape.data > 0
    }

    pub fn available(&self, shape: VoxelShape) -> bool {
        self.data & shape.data == 0
    }

    pub fn append(self, shape: VoxelShape) -> VoxelShape {
        VoxelShape {
            data: self.data | shape.data,
        }
    }

    pub fn mask(self, shape: VoxelShape) -> VoxelShape {
        VoxelShape {
            data: self.data & shape.data,
        }
    }
}

#[derive(Clone, Copy)]
pub struct VoxelData {
    pub shape: VoxelShape,
    pub density: f32,
}
