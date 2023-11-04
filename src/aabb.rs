use glad_gl::gl;

use crate::{mesh::Vertex, shader::Shader};

#[derive(Debug)]
pub struct AABB {
    pub min: glm::Vec3,
    pub max: glm::Vec3,
    indices_len: u32,
    vao: u32,
    vbo: u32,
    ebo: u32,
}

impl AABB {
    pub fn new(min: glm::Vec3, max: glm::Vec3) -> AABB {
        let mut vao = 0;
        let mut vbo = 0;
        let mut ebo = 0;

        let vertices = vec![
            Vertex {
                position: glm::vec3(min.x, min.y, min.z),
                tex_coords: glm::vec2(0.0, 0.0),
                normal: glm::vec3(0.0, 0.0, 0.0),
                tangent: glm::vec3(0.0, 0.0, 0.0),
                bitangent: glm::vec3(0.0, 0.0, 0.0),
            },
            Vertex {
                position: glm::vec3(max.x, min.y, min.z),
                tex_coords: glm::vec2(1.0, 0.0),
                normal: glm::vec3(0.0, 0.0, 0.0),
                tangent: glm::vec3(0.0, 0.0, 0.0),
                bitangent: glm::vec3(0.0, 0.0, 0.0),
            },
            Vertex {
                position: glm::vec3(max.x, max.y, min.z),
                tex_coords: glm::vec2(1.0, 1.0),
                normal: glm::vec3(0.0, 0.0, 0.0),
                tangent: glm::vec3(0.0, 0.0, 0.0),
                bitangent: glm::vec3(0.0, 0.0, 0.0),
            },
            Vertex {
                position: glm::vec3(min.x, max.y, min.z),
                tex_coords: glm::vec2(0.0, 1.0),
                normal: glm::vec3(0.0, 0.0, 0.0),
                tangent: glm::vec3(0.0, 0.0, 0.0),
                bitangent: glm::vec3(0.0, 0.0, 0.0),
            },
            Vertex {
                position: glm::vec3(min.x, min.y, max.z),
                tex_coords: glm::vec2(0.0, 0.0),
                normal: glm::vec3(0.0, 0.0, 0.0),
                tangent: glm::vec3(0.0, 0.0, 0.0),
                bitangent: glm::vec3(0.0, 0.0, 0.0),
            },
            Vertex {
                position: glm::vec3(max.x, min.y, max.z),
                tex_coords: glm::vec2(1.0, 0.0),
                normal: glm::vec3(0.0, 0.0, 0.0),
                tangent: glm::vec3(0.0, 0.0, 0.0),
                bitangent: glm::vec3(0.0, 0.0, 0.0),
            },
            Vertex {
                position: glm::vec3(max.x, max.y, max.z),
                tex_coords: glm::vec2(1.0, 1.0),
                normal: glm::vec3(0.0, 0.0, 0.0),
                tangent: glm::vec3(0.0, 0.0, 0.0),
                bitangent: glm::vec3(0.0, 0.0, 0.0),
            },
            Vertex {
                position: glm::vec3(min.x, max.y, max.z),
                tex_coords: glm::vec2(0.0, 1.0),
                normal: glm::vec3(0.0, 0.0, 0.0),
                tangent: glm::vec3(0.0, 0.0, 0.0),
                bitangent: glm::vec3(0.0, 0.0, 0.0),
            },
        ];

        let indices = vec![
            0, 1, 2, 2, 3, 0, // front
            1, 5, 6, 6, 2, 1, // right
            5, 4, 7, 7, 6, 5, // back
            4, 0, 3, 3, 7, 4, // left
            3, 2, 6, 6, 7, 3, // top
            4, 5, 1, 1, 0, 4, // bottom
        ];

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

            gl::BindVertexArray(0);
        }

        AABB {
            min,
            max,
            indices_len: indices.len() as u32,
            vao,
            vbo,
            ebo
        }
    }

    pub fn draw(&self, shader: &Shader, model_mat: &glm::Mat4) {
        shader.use_shader();

        shader.set_mat4fv("model", &model_mat);
        shader.set_3fv("material.ambient", glm::vec3(1.0, 0.627, 0.157));
        shader.set_3fv("material.diffuse", glm::vec3(1.0, 0.627, 0.157));

        unsafe {
            // draw Mesh
            gl::BindVertexArray(self.vao);
            gl::LineWidth(5.0);
            gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
            gl::DrawElements(gl::TRIANGLES, self.indices_len as i32, gl::UNSIGNED_INT, std::ptr::null());

            // reset stuff to default
            gl::BindVertexArray(0);
            gl::LineWidth(1.0);
        }
    }
}

impl Drop for AABB {
    fn drop(&mut self) {
        unsafe {
            gl::BindVertexArray(0);
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteBuffers(1, &self.ebo);
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}
