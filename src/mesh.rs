use glad_gl::gl;

use crate::{shader::Shader, utils, importer::{Material, TextureType}};

fn create_rotation_matrix(pitch: f32, yaw: f32, roll: f32, pivot: glm::Vec3) -> glm::Mat4 {
    let pitch = pitch.to_radians();
    let yaw = yaw.to_radians();
    let roll = roll.to_radians();

    let mat = utils::mat_ident();

    let rot_z = glm::ext::rotate(&mat, roll, glm::vec3(0.0, 0.0, 1.0));
    let rot_x = glm::ext::rotate(&mat, pitch, glm::vec3(1.0, 0.0, 0.0));
    let rot_y = glm::ext::rotate(&mat, yaw, glm::vec3(0.0, 1.0, 0.0));
    let trans_to_origin = glm::ext::translate(&mat, -glm::vec3(pivot.x, pivot.y, pivot.z));
    let trans_back_to_center = glm::ext::translate(&mat, glm::vec3(pivot.x, pivot.y, pivot.z));

    let combined = trans_back_to_center * rot_z * rot_y * rot_x * trans_to_origin;

    combined
}

pub fn apply_rotation(matrix: &glm::Mat4, rot: glm::Vec3, pivot: glm::Vec3) -> glm::Mat4 {
    let rot = create_rotation_matrix(rot.x, rot.y, rot.z, pivot);

    return rot * *matrix;
}

#[derive(Debug)]
pub struct Mesh {
    pub name: String,
    pub position: glm::Vec3,
    pub rotation: glm::Vec3,
    pub scale: glm::Vec3,
    pub pivot: glm::Vec3,

    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub material: Material,

    vao: u32,
    vbo: u32,
    ebo: u32,
}

impl Mesh {
    pub fn new(name: &str, vertices: Vec<Vertex>, indices: Vec<u32>, material: Option<Material>) -> Mesh {
        let mut vao = 0;
        let mut vbo = 0;
        let mut ebo = 0;

        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::GenBuffers(1, &mut ebo);

            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

            gl::BufferData(gl::ARRAY_BUFFER, (std::mem::size_of::<Vertex>() * vertices.len() as usize) as isize, vertices.as_ptr() as *const std::ffi::c_void, gl::STATIC_DRAW);

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, (std::mem::size_of::<u32>() * indices.len() as usize) as isize, indices.as_ptr() as *const std::ffi::c_void, gl::STATIC_DRAW);

            // vertex positions
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, std::mem::size_of::<Vertex>() as i32, std::ptr::null());

            // vertex normals
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, std::mem::size_of::<Vertex>() as i32, (3 * std::mem::size_of::<f32>()) as *const std::ffi::c_void);

            // vertex texture coords
            gl::EnableVertexAttribArray(2);
            gl::VertexAttribPointer(2, 2, gl::FLOAT, gl::FALSE, std::mem::size_of::<Vertex>() as i32, (6 * std::mem::size_of::<f32>()) as *const std::ffi::c_void);

            // vertex tangent
            gl::EnableVertexAttribArray(3);
            gl::VertexAttribPointer(3, 3, gl::FLOAT, gl::FALSE, std::mem::size_of::<Vertex>() as i32, (8 * std::mem::size_of::<f32>()) as *const std::ffi::c_void);

            // vertex bitangent
            gl::EnableVertexAttribArray(4);
            gl::VertexAttribPointer(4, 3, gl::FLOAT, gl::FALSE, std::mem::size_of::<Vertex>() as i32, (11 * std::mem::size_of::<f32>()) as *const std::ffi::c_void);

            gl::BindVertexArray(0);
        }

        Mesh {
            name: name.to_string(),
            vertices,
            indices,
            material: material.unwrap_or_default(),
            vbo,
            vao,
            ebo,
            position: glm::vec3(0.0, 0.0, 0.0),
            rotation: glm::vec3(0.0, 0.0, 0.0),
            scale: glm::vec3(1.0, 1.0, 1.0),
            pivot: glm::vec3(0.0, 0.0, 0.0),
        }
    }

    pub fn draw(&self, shader: &Shader, scale: f32, pivot: glm::Vec3, show_textures: bool, use_normal: bool, use_emissive: bool) {
        shader.use_shader();

        let model_mat = glm::ext::scale(&utils::mat_ident(), glm::vec3(scale, scale, scale));
        let model_mat = apply_rotation(&model_mat, self.rotation, pivot);
        let model_mat = glm::ext::translate(&model_mat, glm::vec3(self.position.x, self.position.y, self.position.z));
        shader.set_mat4fv("model", &model_mat);

        if glm::ext::is_invertible(&model_mat) {
            let normal_mat = glm::transpose(&glm::inverse(&model_mat));
            let normal_mat = glm::mat3(
                normal_mat[0][0], normal_mat[0][1], normal_mat[0][2],
                normal_mat[1][0], normal_mat[1][1], normal_mat[1][2],
                normal_mat[2][0], normal_mat[2][1], normal_mat[2][2],
            );
            shader.set_mat3fv("normalMatrix", &normal_mat);
            shader.set_bool("useNormalMatrix", true);
        } else {
            shader.set_bool("useNormalMatrix", false);
        }

        let mut polygon_mode = 0;
        unsafe {
            gl::GetIntegerv(gl::POLYGON_MODE, &mut polygon_mode);
        }
        let is_wireframe = polygon_mode as u32 == gl::LINE;

        if !is_wireframe {
            // TODO: these can be missing in the (.obj) material, maybe we should set them
            // to 1.0 as fallback. shininess too
            shader.set_3fv("material.ambient", self.material.ambient_color);
            shader.set_3fv("material.diffuse", self.material.diffuse_color);
            shader.set_3fv("material.specular", self.material.specular_color);
            shader.set_float("material.shininess", self.material.specular_exponent);
            shader.set_float("material.opacity", self.material.opacity);
        } else {
            shader.set_3fv("material.ambient", glm::vec3(0.0, 0.0, 0.0));
            shader.set_3fv("material.diffuse", glm::vec3(0.0, 0.0, 0.0));
        }

        if show_textures {
            shader.set_bool("useTextures", self.material.textures.len() > 0);
            shader.set_bool("hasNormalTexture", false);
            shader.set_bool("hasEmissionTexture", false);
            for i in 0..self.material.textures.len() {
                unsafe {
                    gl::ActiveTexture(gl::TEXTURE0 + i as u32);
                    gl::BindTexture(gl::TEXTURE_2D, self.material.textures[i].id);
                    match self.material.textures[i].typ {
                        TextureType::Ambient => {
                            shader.set_int("material.texture_ambient", i as i32);
                        },
                        TextureType::Diffuse => {
                            shader.set_int("material.texture_diffuse", i as i32);
                        },
                        TextureType::Specular => {
                            shader.set_int("material.texture_specular", i as i32);
                        },
                        TextureType::Emissive => {
                            if use_emissive {
                                shader.set_bool("hasEmissionTexture", true);
                            }
                            shader.set_int("material.texture_emission", i as i32);
                        },
                        TextureType::Bump => {
                            if use_normal {
                                shader.set_bool("hasNormalTexture", true);
                            }
                            shader.set_int("material.texture_normal", i as i32);
                        },
                        _ => { continue; },
                    }

                }
            }
        } else {
            shader.set_bool("useTextures", false);
        }

        unsafe {
            // draw Mesh
            gl::BindVertexArray(self.vao);
            gl::DrawElements(gl::TRIANGLES, self.indices.len() as i32, gl::UNSIGNED_INT, std::ptr::null());

            // reset stuff to default
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindVertexArray(0);
        }
    }

    pub fn rotate(&mut self, rotation: glm::Vec3) {
        self.rotation = self.rotation + rotation;
    }

    pub fn reset_rotation(&mut self) {
        self.rotation = glm::vec3(0.0, 0.0, 0.0);
    }
}

impl Drop for Mesh {
    fn drop(&mut self) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindVertexArray(0);
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteBuffers(1, &self.ebo);
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteTextures(self.material.textures.len() as i32, self.material.textures.iter().map(|t| t.id).collect::<Vec<u32>>().as_ptr());
        }
    }
}

#[derive(Clone, Debug)]
#[repr(packed(2))]
pub struct Vertex {
    pub position: glm::Vec3,
    pub normal: glm::Vec3,
    pub tex_coords: glm::Vec2,
    pub tangent: glm::Vec3,
    pub bitangent: glm::Vec3,
}

impl Vertex {
    pub fn new(position: glm::Vec3, normal: glm::Vec3, tex_coords: glm::Vec2, tangent: glm::Vec3, bitangent: glm::Vec3) -> Self {
        Vertex {
            position,
            normal,
            tex_coords,
            tangent,
            bitangent,
        }
    }
}

