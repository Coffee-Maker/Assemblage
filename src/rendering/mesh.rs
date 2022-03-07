use super::vertex::Vertex;

#[derive(Clone)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn new() -> Mesh {
        Mesh {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn add_custom(
        mut self,
        vertices: Vec<[f32; 3]>,
        indices: Vec<u32>,
        normal: [f32; 3],
    ) -> Mesh {
        let index_offset = self.vertices.len() as u32;
        self.indices.reserve(indices.len());
        indices.iter().for_each(|i| {
            self.indices.push(index_offset + i);
        });

        self.vertices.reserve(vertices.len());

        let color = [0.5, 0.3, 0.2];

        vertices.iter().for_each(|position| {
            self.vertices.push(Vertex {
                position: *position,
                color,
                normal,
                uv: [0.0, 0.0],
            }) // TODO: Add UVs
        });

        self
    }

    pub fn add_quad(mut self, quad_verts: [[f32; 3]; 4], normal: [f32; 3]) -> Mesh {
        let index_offset = self.vertices.len() as u32;
        self.indices.append(&mut vec![
            index_offset,
            index_offset + 2,
            index_offset + 1,
            index_offset + 2,
            index_offset + 3,
            index_offset + 1,
        ]);
        self.vertices.reserve(4);

        let color = [0.5, 0.3, 0.2];

        // v0
        self.vertices.push(Vertex {
            position: [quad_verts[0][0], quad_verts[0][1], quad_verts[0][2]],
            color: color,
            normal,
            uv: [0.0, 0.0],
        });

        // v1
        self.vertices.push(Vertex {
            position: [quad_verts[1][0], quad_verts[1][1], quad_verts[1][2]],
            color: color,
            normal,
            uv: [1.0, 0.0],
        });

        // v2
        self.vertices.push(Vertex {
            position: [quad_verts[2][0], quad_verts[2][1], quad_verts[2][2]],
            color: color,
            normal,
            uv: [0.0, 1.0],
        });

        // v3
        self.vertices.push(Vertex {
            position: [quad_verts[3][0], quad_verts[3][1], quad_verts[3][2]],
            color: color,
            normal,
            uv: [1.0, 1.0],
        });

        self
    }

    pub fn add_tri(mut self, quad_verts: [[f32; 3]; 3], normal: [f32; 3]) -> Mesh {
        let index_offset = self.vertices.len() as u32;
        self.indices
            .append(&mut vec![index_offset, index_offset + 2, index_offset + 1]);
        self.vertices.reserve(4);

        let color = [0.8, 0.5, 0.3];

        // v0
        self.vertices.push(Vertex {
            position: [quad_verts[0][0], quad_verts[0][1], quad_verts[0][2]],
            color: color,
            normal,
            uv: [0.0, 0.0],
        });

        // v1
        self.vertices.push(Vertex {
            position: [quad_verts[1][0], quad_verts[1][1], quad_verts[1][2]],
            color: color,
            normal,
            uv: [1.0, 0.0],
        });

        // v2
        self.vertices.push(Vertex {
            position: [quad_verts[2][0], quad_verts[2][1], quad_verts[2][2]],
            color: color,
            normal,
            uv: [0.0, 1.0],
        });

        self
    }
}
