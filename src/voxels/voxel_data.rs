use super::voxel_shapes::VoxelShape;

#[derive(Clone, Copy)]
#[repr(packed(4))]
pub struct VoxelData {
    pub shape: VoxelShape,
    pub state: u8,
    pub id: u16,
}
