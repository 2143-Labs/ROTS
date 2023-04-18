use bevy::{prelude::*, render::render_resource::PrimitiveTopology, sprite::TextureAtlas};

fn generate_mesh_from_sprite(texture_atlas: Handle<TextureAtlas>) -> PbrBundle {
    // iterate over each pixel in the texture atlas
    // if the pixel is not transparent, add a cube to the mesh
    // return the mesh
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut index = 0;
    // get the texture from the texture atlas handle
    let texture = texture_atlas.get().textures[0].get();

    let texture_size = texture.size();
    let texture_data = texture.data.as_ref().unwrap();
    for x in 0..texture_size.width {
        for y in 0..texture_size.height {
            let pixel = texture_data.get_pixel(x, y);
            // if not transparent
            if pixel[3] != 0 {
                let cube = Mesh::from(shape::Cube { size: 1. });
                let mut cube_vertices = cube.attribute(Mesh::ATTRIBUTE_POSITION).unwrap().into();
                let mut cube_indices = cube.indices().unwrap().into();
                // move the vertices to the correct position
                for i in 0..cube_vertices.len() {
                    cube_vertices[i] += Vec3::new(x as f32, y as f32, 0.);
                }
                // add the index offset to the indices
                for i in 0..cube_indices.len() {
                    cube_indices[i] += index;
                }
                index += cube_vertices.len();
                vertices.append(&mut cube_vertices);
                indices.append(&mut cube_indices);
            }
        }
    }
    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.set_indices(Some(Indices::U32(indices)));
    PbrBundle {
        mesh: mesh,
        material: texture_atlas.get().textures[0],
        ..default()
    }
}
