use std::sync::Arc;

use legion::{IntoQuery, World};

use crate::{
    ecs::components::rendering_components::MeshRenderer,
    rendering::render_pass_data::{create_render_pass, render_layers},
    state::State,
};

pub fn construct_buffers(state: &State, world: &World) {
    // Clear passes
    render_layers::RENDER_LAYERS.iter().for_each(|layer| {
        let mut layer_lock = layer.write();
        layer_lock.passes.clear();
    });

    // Loop through all mesh renderers and append their data to the pass buffers
    // TODO: It is worth noting that this should only be done when there is a change
    let mut query = <(&MeshRenderer)>::query();
    query.iter(world).for_each(|mesh| {
        let layer = render_layers::get_layer_by_name(mesh.render_layer.to_string());
        let layer = match layer {
            Some(layer) => layer,
            None => return,
        };

        let mut pass = create_render_pass(state, Arc::clone(&mesh.material));
        let mesh_lock = mesh.mesh.read();
        if mesh_lock.vertices.len() == 0 {
            return;
        }

        pass.set_indices(&state.device, &mesh_lock.indices);
        pass.set_vertices(&state.device, &mesh_lock.vertices);
        let mut layer_lock = layer.write();
        layer_lock.push_pass(pass);
    });
}
