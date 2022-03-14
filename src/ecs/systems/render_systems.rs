use std::sync::{atomic::Ordering, Arc};

use dashmap::DashMap;
use glam::{Mat4, Quat, Vec3};
use legion::{IntoQuery, World};

use crate::{
    ecs::components::{rendering_components::MeshRenderer, transformation_components::Position},
    rendering::{
        material::Material,
        render_pass_data::{render_layers, RenderPassData},
    },
    state::State,
};

lazy_static! {
    static ref PASSES: DashMap<u64, Arc<RenderPassData<dyn Material>>> = DashMap::new();
}

pub fn construct_buffers(state: &State, world: &World) {
    // Loop through all mesh renderers and append their data to the pass buffers if their data is dirty
    let mut query = <(&MeshRenderer, &Position)>::query();
    query.iter(world).for_each(|(renderer, position)| {
        if !renderer.dirty.load(Ordering::Relaxed) {
            return;
        }

        let mesh_lock = renderer.mesh.read();
        if mesh_lock.vertex_count == 0 {
            return;
        }

        let layer = render_layers::get_layer_by_name(renderer.render_layer.to_string());
        let layer = match layer {
            Some(layer) => layer,
            None => return,
        };

        let mut layer_lock = layer.write();
        let pass = layer_lock.get_or_create_pass(state, Arc::clone(&renderer.material));

        let transform =
            Mat4::from_scale_rotation_translation(Vec3::ONE, Quat::IDENTITY, position.0);

        pass.write()
            .insert_mesh(&state, Arc::clone(&renderer.mesh), &transform);

        renderer.dirty.store(false, Ordering::Relaxed);
    });
}
