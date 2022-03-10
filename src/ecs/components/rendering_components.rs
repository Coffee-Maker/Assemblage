use crate::rendering::mesh::Mesh;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeshRenderer<'a> {
    pub mesh: &'a Mesh,
    pub material: &'a Material,
}
