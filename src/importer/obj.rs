use std::io::{BufReader, BufRead};

use crate::{mesh::Vertex, aabb::AABB, importer::ObjMesh, importer::Object};

const BUF_CAP: usize = 1024 * 128; // 128 Kilobytes

enum Token {
    Object,
    Group,
    Vertex,
    Normal,
    TexCoord,
    Face,
    Point,
    Line,
    SmoothShading,
    MaterialLib,
    MaterialUsage,
}

impl Token {
    fn from_str<'a>(s: &'a str) -> Option<Token> {
        match s {
            "o" => Some(Token::Object),
            "g" => Some(Token::Group),
            "v" => Some(Token::Vertex),
            "vn" => Some(Token::Normal),
            "vt" => Some(Token::TexCoord),
            "f" => Some(Token::Face),
            "p" => Some(Token::Point),
            "l" => Some(Token::Line),
            "s" => Some(Token::SmoothShading),
            "mtllib" => Some(Token::MaterialLib),
            "usemtl" => Some(Token::MaterialUsage),
            _ => None,
        }
    }
}

pub fn load_obj(file: std::fs::File) -> Result<Object, Box<dyn std::error::Error>> {
    let now = std::time::Instant::now();
    let reader = BufReader::with_capacity(BUF_CAP, file);
    let mut object_name = String::new();
    let mut current_mesh_name = String::new();
    let mut temp_vertices = Vec::new();
    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut indices_counter: u32 = 0;
    let mut indices = Vec::new();
    let mut tex_coords = Vec::new();
    let mut meshes = Vec::new();
    let mut min_aabb = glm::vec3(std::f32::MAX, std::f32::MAX, std::f32::MAX);
    let mut max_aabb = glm::vec3(std::f32::MIN, std::f32::MIN, std::f32::MIN);

    for line in reader.lines() {
        let line = line?;
        // skip empty lines and comments
        if line.is_empty() || line.chars().nth(0).is_some_and(|c| c == '#') {
            continue;
        }

        let mut iter = line.split_ascii_whitespace();
        let first = iter.next();
        if let Some(token) = first {
            match Token::from_str(token) {
                Some(Token::Object) => {
                    let name = {
                        if current_mesh_name.is_empty() {
                            object_name.clone()
                        } else {
                            current_mesh_name.clone()
                        }
                    };
                    if !vertices.is_empty() {
                        meshes.push(ObjMesh{
                            name,
                            vertices: vertices.clone(),
                            indices: indices.clone(),
                        });
                    }
                    vertices.clear();
                    indices.clear();
                    indices_counter = 0;

                    object_name = iter.next().unwrap_or("").to_string();
                }
                Some(Token::Group) => {
                    let name = {
                        if current_mesh_name.is_empty() {
                            object_name.clone()
                        } else {
                            current_mesh_name.clone()
                        }
                    };
                    if !vertices.is_empty() {
                        meshes.push(ObjMesh{
                            name,
                            vertices: vertices.clone(),
                            indices: indices.clone(),
                        });
                    }
                    vertices.clear();
                    indices.clear();
                    indices_counter = 0;

                    current_mesh_name = iter.next().unwrap_or("default_mesh").to_string();
                }
                Some(Token::Vertex) => {
                    let mut iter = iter
                        .take(3)
                        .map(|i| i.parse::<f32>().unwrap());
                    let x = iter.next().unwrap();
                    let y = iter.next().unwrap();
                    let z = iter.next().unwrap();
                    temp_vertices.push(glm::vec3(x, y, z));
                    if x < min_aabb.x {
                        min_aabb.x = x;
                    }

                    if y < min_aabb.y {
                        min_aabb.y = y;
                    }

                    if z < min_aabb.z {
                        min_aabb.z = z;
                    }

                    if x > max_aabb.x {
                        max_aabb.x = x;
                    }

                    if y > max_aabb.y {
                        max_aabb.y = y;
                    }

                    if z > max_aabb.z {
                        max_aabb.z = z;
                    }

                }
                Some(Token::Normal) => {
                    let mut iter = iter
                        .take(3)
                        .map(|i| i.parse::<f32>().unwrap());
                    let x = iter.next().unwrap();
                    let y = iter.next().unwrap();
                    let z = iter.next().unwrap();
                    normals.push(glm::vec3(x, y, z));
                }
                Some(Token::TexCoord) => {
                    let mut iter = iter
                        .take(2)
                        .map(|i| i.parse::<f32>().unwrap());
                    let u = iter.next().unwrap();
                    let v = iter.next().unwrap();
                    tex_coords.push(glm::vec2(u, v));
                }
                Some(Token::Face) => {
                    // TODO: vertex indices can be negative

                    let face = iter.collect::<Vec<_>>();
                    let mut calculated_normal = glm::vec3(0.0, 0.0, 0.0);

                    if normals.is_empty() {
                        calculated_normal = glm::normalize(glm::cross(
                            temp_vertices[face[1].parse::<i32>().unwrap() as usize - 1] - temp_vertices[face[0].parse::<i32>().unwrap() as usize - 1],
                            temp_vertices[face[2].parse::<i32>().unwrap() as usize - 1] - temp_vertices[face[0].parse::<i32>().unwrap() as usize - 1]
                        ));
                    }

                    for (i, vert) in face.iter().enumerate() {
                        if vert.contains("//") {
                            let mut it = vert.split("//");
                            let vert = it.next().unwrap().parse::<i32>().unwrap() - 1;
                            let normal = it.next().unwrap().parse::<i32>().unwrap() - 1;
                            vertices.push(Vertex{
                                position: *temp_vertices.get(vert as usize).unwrap(),
                                normal: *normals.get(normal as usize).unwrap(),
                                tex_coords: glm::vec2(0.0, 0.0)
                            });
                        } else if vert.contains("/") {
                            let mut it = vert.split("/");
                            let vertex = it.next().unwrap().parse::<i32>().unwrap() - 1;
                            let t_coords = it.next().unwrap().parse::<i32>().unwrap() - 1;
                            let normal = it.next().unwrap().parse::<i32>().unwrap() - 1;
                            vertices.push(Vertex{
                                position: *temp_vertices.get(vertex as usize).unwrap(),
                                normal: *normals.get(normal as usize).unwrap(),
                                tex_coords: *tex_coords.get(t_coords as usize).unwrap()
                            });
                        } else {
                            let vert = vert.parse::<i32>().unwrap() - 1;
                            vertices.push(Vertex{
                                position: *temp_vertices.get(vert as usize).unwrap(),
                                normal: calculated_normal,
                                tex_coords: glm::vec2(0.0, 0.0),
                            });
                        }

                        // 2 triangles per face
                        if i < face.len() - 2 {
                            indices.push(indices_counter);
                            indices.push(indices_counter + i as u32 + 1);
                            indices.push(indices_counter + i as u32 + 2);
                        }
                    }

                    indices_counter += face.len() as u32;
                }
                Some(Token::MaterialLib) => {
                    // TODO: handle material file
                }
                Some(Token::MaterialUsage) => {
                    // TODO: handle material usage
                    // this can be used multiple times before f is mentioned
                    // to apply the material to those faces after it
                }
                _ => {},
            }
        }
    }
    let elapsed = now.elapsed();
    println!("Loaded in {}ms",  elapsed.as_millis());

    let mesh_name = {
        if current_mesh_name.is_empty() && !object_name.is_empty() {
            object_name.clone()
        } else if !current_mesh_name.is_empty() {
            current_mesh_name.clone()
        } else {
            "default_mesh".to_string()
        }
    };

    meshes.push(ObjMesh{
        name: mesh_name,
        vertices: vertices.clone(),
        indices: indices.clone(),
    });

    let aabb = AABB::new(min_aabb, max_aabb);

    Ok(Object{
        name: object_name,
        meshes,
        aabb
    })
}
