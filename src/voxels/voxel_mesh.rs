use crate::rendering::mesh::Mesh;

use super::voxel_shapes::VoxelShape;

#[rustfmt::skip]
mod voxel_meshes {
    use crate::{rendering::{mesh::Mesh, vertex::Vertex}, voxels::voxel_mesh::VoxelMesh};

    lazy_static! {
        pub static ref CUBE_MESH: VoxelMesh = VoxelMesh {
            always: Mesh::new(),
            north:  add_quad(Mesh::new(), [[1.0, 0.0, 1.0], [1.0, 1.0, 1.0], [0.0, 0.0, 1.0], [0.0, 1.0, 1.0]], [0.0, 0.0, 1.0]),
            south:  add_quad(Mesh::new(), [[0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0]], [0.0, 0.0, -1.0]),
            east:   add_quad(Mesh::new(), [[1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [1.0, 0.0, 1.0], [1.0, 1.0, 1.0]], [1.0, 0.0, 0.0]),
            west:   add_quad(Mesh::new(), [[0.0, 0.0, 1.0], [0.0, 1.0, 1.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]], [-1.0, 0.0, 0.0]),
            top:    add_quad(Mesh::new(), [[0.0, 1.0, 0.0], [0.0, 1.0, 1.0], [1.0, 1.0, 0.0], [1.0, 1.0, 1.0]], [0.0, 1.0, 0.0]),
            bottom: add_quad(Mesh::new(), [[0.0, 0.0, 1.0], [0.0, 0.0, 0.0], [1.0, 0.0, 1.0], [1.0, 0.0, 0.0]], [0.0, -1.0, 0.0]),
        };

        pub static ref SLAB: VoxelMesh = VoxelMesh {
            always: Mesh::new(),
            north:  add_quad(Mesh::new(), [[1.0, 0.0, 1.0], [1.0, 1.0, 1.0], [0.0, 0.0, 1.0], [0.0, 1.0, 1.0]], [0.0, 0.0, 1.0]),
            south:  add_quad(Mesh::new(), [[0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0]], [0.0, 0.0, -1.0]),
            east:   add_quad(Mesh::new(), [[1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [1.0, 0.0, 1.0], [1.0, 1.0, 1.0]], [1.0, 0.0, 0.0]),
            west:   add_quad(Mesh::new(), [[0.0, 0.0, 1.0], [0.0, 1.0, 1.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]], [-1.0, 0.0, 0.0]),
            top:    add_quad(Mesh::new(), [[0.0, 1.0, 0.0], [0.0, 1.0, 1.0], [1.0, 1.0, 0.0], [1.0, 1.0, 1.0]], [0.0, 1.0, 0.0]),
            bottom: add_quad(Mesh::new(), [[0.0, 0.0, 1.0], [0.0, 0.0, 0.0], [1.0, 0.0, 1.0], [1.0, 0.0, 0.0]], [0.0, -1.0, 0.0]),
        };
    }

    fn add_quad(mut mesh: Mesh, quad_verts: [[f32; 3]; 4], normal: [f32; 3]) -> Mesh {
        let index_offset = mesh.vertices.len() as u32;
        mesh.indices.append(&mut vec![
            index_offset,
            index_offset + 1,
            index_offset + 2,
            index_offset + 2,
            index_offset + 1,
            index_offset + 3,
        ]);
        mesh.vertices.reserve(4);

        let color = [0.8, 0.5, 0.3];

        // v0
        mesh.vertices.push(Vertex {
            position: [
                quad_verts[0][0],
                quad_verts[0][1],
                quad_verts[0][2],
            ],
            color: color,
            normal,
            uv: [0.0, 0.0],
        });

        // v1
        mesh.vertices.push(Vertex {
            position: [
                quad_verts[1][0],
                quad_verts[1][1],
                quad_verts[1][2],
            ],
            color: color,
            normal,
            uv: [1.0, 0.0],
        });

        // v2
        mesh.vertices.push(Vertex {
            position: [
                quad_verts[2][0],
                quad_verts[2][1],
                quad_verts[2][2],
            ],
            color: color,
            normal,
            uv: [0.0, 1.0],
        });

        // v3
        mesh.vertices.push(Vertex {
            position: [
                quad_verts[3][0],
                quad_verts[3][1],
                quad_verts[3][2],
            ],
            color: color,
            normal,
            uv: [1.0, 1.0],
        });

        mesh
    }
}

pub struct VoxelMesh {
    pub always: Mesh,
    pub north: Mesh,
    pub south: Mesh,
    pub east: Mesh,
    pub west: Mesh,
    pub top: Mesh,
    pub bottom: Mesh,
}

pub fn get_voxel_mesh(shape: VoxelShape) -> &'static VoxelMesh {
    &*voxel_meshes::CUBE_MESH
}
