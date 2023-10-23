use crate::{mesh::{Mesh, apply_rotation}, shader::Shader, utils, ui::ui, aabb, importer};

const SCALING_FACTOR: f32 = 8.0;

#[derive(Debug)]
pub struct Model {
    pub id: u32,
    pub name: String,
    pub meshes: Vec<Mesh>,
    pub aabb: aabb::AABB,
    pub scaling_factor: f32,
    pub mem_usage: usize,
}

// TODO: texture support, kept for reference
// fn load_material_textures(
//     mat: &russimp::material::Material,
//     dir: &std::path::PathBuf,
//     loaded_textures: &mut Vec<Texture>
// ) -> (Vec<Texture>, Vec<Box<dyn std::error::Error>>) {

//     let mut textures = vec![];
//     let mut errors = vec![];

//     for (typ, tex) in mat.textures.iter() {
//         if SUPPORTED_TEXTURE_TYPES.contains(typ) {
//             let texture = tex.borrow();
//             let mut skip = false;
//             let tex_filename = &texture.filename;
//             // HACK: fix this
//             if tex_filename.is_empty() {
//                 continue;
//             }
//             // println!("texture filename: {}", tex_filename);
//             let path = dir.join(tex_filename);

//             for loaded_tex in &mut *loaded_textures {
//                 if loaded_tex.path == path {
//                     textures.push(loaded_tex.clone());
//                     skip = true;
//                     break;
//                 }
//             }

//             if !skip {
//                 match Texture::new(path, *typ) {
//                     Ok(texture) => {
//                         loaded_textures.push(texture.clone());
//                         textures.push(texture);
//                     },
//                     Err(e) => {
//                         let err = anyhow!("Error loading texture: {}", e);
//                         println!("{}", err);
//                         errors.push(err.into());
//                     },
//                 }
//             }
//         }
//     }

//     return (textures, errors);
// }

impl Model {
    pub fn new(obj: importer::Object, state: &mut ui::State) -> Model {
        let mut meshes = Vec::new();

        let scale_factor_x = SCALING_FACTOR / (obj.aabb.max.x - obj.aabb.min.x);
        let scale_factor_y = SCALING_FACTOR / (obj.aabb.max.y - obj.aabb.min.y);
        let scale_factor_z = SCALING_FACTOR / (obj.aabb.max.z - obj.aabb.min.z);

        // Use the minimum scaling factor to maintain proportions
        let scale = scale_factor_x.min(scale_factor_y).min(scale_factor_z);

        for mesh in obj.meshes.into_iter() {
            meshes.push(Mesh::new(&mesh.name, mesh.vertices, mesh.indices, mesh.material));
        }

        let mut model = Model {
            id: state.get_next_id(),
            name: obj.name.to_owned(),
            aabb: obj.aabb,
            scaling_factor: scale,
            meshes,
            mem_usage: 0,
        };

        model.set_mem_usage();

        return model;
    }

    pub fn draw(&self, shader: &Shader, draw_aabb: bool) {
        let center_x = ((self.aabb.max.x / 2.0) + (self.aabb.min.x / 2.0)) * self.scaling_factor;
        let center_y = ((self.aabb.max.y / 2.0) + (self.aabb.min.y / 2.0)) * self.scaling_factor;
        let center_z = ((self.aabb.max.z / 2.0) + (self.aabb.min.z / 2.0)) * self.scaling_factor;
        let pivot = glm::vec3(center_x, center_y, center_z);

        let model_mat = glm::ext::scale(&utils::mat_ident(), glm::vec3(self.scaling_factor, self.scaling_factor, self.scaling_factor));
        let model_mat = apply_rotation(&model_mat, self.meshes[0].rotation, pivot);
        let model_mat = glm::ext::translate(&model_mat, glm::vec3(self.meshes[0].position.x, self.meshes[0].position.y, self.meshes[0].position.z));


        for mesh in &self.meshes {
            mesh.draw(shader, self.scaling_factor, pivot);
        }

        if draw_aabb {
            self.aabb.draw(shader, &model_mat);
        }
    }

    pub fn rotate(&mut self, xoffset: f32, yoffset: f32) -> &mut Self {
        let rotation = glm::vec3(-yoffset, xoffset, 0.0);
        for mesh in &mut self.meshes {
            mesh.rotate(rotation);
        }
        self
    }

    pub fn reset_rotation(&mut self) -> &mut Self {
        for mesh in &mut self.meshes {
            mesh.reset_rotation();
        }
        self
    }

    fn set_mem_usage(&mut self) {
        let mut size: usize = 0;

        size += std::mem::size_of_val(self);
        for mesh in &self.meshes {
            size += std::mem::size_of_val(mesh);
            size += std::mem::size_of::<importer::Material>();

            for texture in &mesh.material.textures {
                size += std::mem::size_of_val(&texture);
            }
            for vertex in &mesh.vertices {
                size += std::mem::size_of_val(vertex);
            }
            for index in &mesh.indices {
                size += std::mem::size_of_val(index);
            }
        }

        self.mem_usage = size;
    }
}
