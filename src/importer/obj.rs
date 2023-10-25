use std::{io::{BufReader, BufRead}, collections::HashMap, path::PathBuf};

use log::{error, warn, trace};

use crate::{mesh::Vertex, aabb::AABB, importer::ObjMesh, importer::Object, importer::Material};

const BUF_CAP: usize = 1024 * 128; // 128 Kilobytes

enum ObjToken {
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

impl ObjToken {
    fn from_str<'a>(s: &'a str) -> Option<ObjToken> {
        match s {
            "o" => Some(ObjToken::Object),
            "g" => Some(ObjToken::Group),
            "v" => Some(ObjToken::Vertex),
            "vn" => Some(ObjToken::Normal),
            "vt" => Some(ObjToken::TexCoord),
            "f" => Some(ObjToken::Face),
            "p" => Some(ObjToken::Point),
            "l" => Some(ObjToken::Line),
            "s" => Some(ObjToken::SmoothShading),
            "mtllib" => Some(ObjToken::MaterialLib),
            "usemtl" => Some(ObjToken::MaterialUsage),
            _ => None,
        }
    }
}

enum MtlToken {
    NewMaterial,
    AmbientColor,
    DiffuseColor,
    SpecularColor,
    Emissive,
    SpecularExponent,
    Refraction,
    Opacity,
    AmbientTexture,
    DiffuseTexture,
    SpecularTexture,
    SpecularHighlightTexture,
    EmissiveTexture,
    BumpTexture,
    DisplacementTexture,
    DecalTexture,
    ReflectionTexture,
}

impl MtlToken {
    fn from_str(s: &str) -> Option<MtlToken> {
        match s {
            "newmtl" => Some(MtlToken::NewMaterial),
            "Ka" => Some(MtlToken::AmbientColor),
            "Kd" => Some(MtlToken::DiffuseColor),
            "Ks" => Some(MtlToken::SpecularColor),
            "Ke" => Some(MtlToken::Emissive),
            "Ns" => Some(MtlToken::SpecularExponent),
            "Ni" => Some(MtlToken::Refraction),
            "d" => Some(MtlToken::Opacity),
            "map_Ka" => Some(MtlToken::AmbientTexture),
            "map_Kd" => Some(MtlToken::DiffuseTexture),
            "map_Ks" => Some(MtlToken::SpecularTexture),
            "map_Ns" => Some(MtlToken::SpecularHighlightTexture),
            "map_Ke" => Some(MtlToken::EmissiveTexture),
            "map_bump" => Some(MtlToken::BumpTexture),
            "map_d" => Some(MtlToken::DisplacementTexture),
            "decal" => Some(MtlToken::DecalTexture),
            "refl" => Some(MtlToken::ReflectionTexture),
            _ => None,
        }
    }
}

fn parse_mtl(path: &PathBuf) -> Result<HashMap<String, Material>, Box<dyn std::error::Error>> {
    let file = std::fs::File::open(path)?;
    let reader = BufReader::with_capacity(BUF_CAP, file);
    let mut material_name = String::new();
    let mut materials: HashMap<String, Material> = HashMap::new();
    let mut material;
    let mut ambient = glm::vec3(0.0, 0.0, 0.0);
    let mut diffuse = glm::vec3(0.0, 0.0, 0.0);
    let mut specular = glm::vec3(0.0, 0.0, 0.0);
    let mut shininess = 32.0;
    let mut opacity = 1.0;

    for line in reader.lines() {
        let line = line?;
        // skip empty lines and comments
        if line.is_empty() || line.chars().nth(0).is_some_and(|c| c == '#') {
            continue;
        }

        let mut iter = line.split_ascii_whitespace();
        let first = iter.next();
        if let Some(token) = first {
            match MtlToken::from_str(token) {
                Some(MtlToken::NewMaterial) => {
                    if !material_name.is_empty() {
                        material = Material::new(material_name.clone(), ambient, diffuse, specular, shininess, opacity, HashMap::new());
                        materials.insert(material_name, material);
                    }

                    material_name = iter.next().unwrap().to_string();
                }
                Some(MtlToken::AmbientColor) => {
                    let r = iter.next().unwrap().parse::<f32>().unwrap();
                    let g = iter.next().unwrap().parse::<f32>().unwrap();
                    let b = iter.next().unwrap().parse::<f32>().unwrap();
                    ambient = glm::vec3(r, g, b);
                }
                Some(MtlToken::DiffuseColor) => {
                    let r = iter.next().unwrap().parse::<f32>().unwrap();
                    let g = iter.next().unwrap().parse::<f32>().unwrap();
                    let b = iter.next().unwrap().parse::<f32>().unwrap();
                    diffuse = glm::vec3(r, g, b);
                }
                Some(MtlToken::SpecularColor) => {
                    let r = iter.next().unwrap().parse::<f32>().unwrap();
                    let g = iter.next().unwrap().parse::<f32>().unwrap();
                    let b = iter.next().unwrap().parse::<f32>().unwrap();
                    specular = glm::vec3(r, g, b);
                }
                Some(MtlToken::SpecularExponent) => {
                    shininess = iter.next().unwrap().parse::<f32>().unwrap();
                }
                Some(MtlToken::Opacity) => {
                    opacity = iter.next().unwrap().parse::<f32>().unwrap();
                }
                Some(MtlToken::DiffuseTexture) => {
                    // TODO: textures
                }
                _ => { warn!("Unhandled material token: {}", token) },
            }
        }
    }

    material = Material::new(material_name.clone(), ambient, diffuse, specular, shininess, opacity, HashMap::new());

    materials.insert(material_name, material);

    Ok(materials)
}

pub fn load_obj(obj_path: &PathBuf, file: std::fs::File) -> Result<Object, Box<dyn std::error::Error>> {
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
    let mut materials: HashMap<String, Material> = HashMap::new();
    let mut current_material: Option<Material> = None;
    let mut min_aabb = glm::vec3(f32::MAX, f32::MAX, f32::MAX);
    let mut max_aabb = glm::vec3(f32::MIN, f32::MIN, f32::MIN);

    for line in reader.lines() {
        let line = line?;
        // skip empty lines and comments
        if line.is_empty() || line.chars().nth(0).is_some_and(|c| c == '#') {
            continue;
        }

        let mut iter = line.split_ascii_whitespace();
        let first = iter.next();
        if let Some(token) = first {
            match ObjToken::from_str(token) {
                Some(ObjToken::Object) => {
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
                            material: current_material.clone()
                        });
                    }
                    vertices.clear();
                    indices.clear();
                    indices_counter = 0;

                    object_name = iter.next().unwrap_or("").to_string();
                }
                Some(ObjToken::Group) => {
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
                            material: current_material.clone()
                        });
                    }
                    vertices.clear();
                    indices.clear();
                    indices_counter = 0;

                    current_mesh_name = iter.next().unwrap_or("default_mesh").to_string();
                }
                Some(ObjToken::Vertex) => {
                    let mut iter = iter
                        .take(3)
                        .map(|i| i.parse::<f32>().unwrap());
                    let x = iter.next().unwrap();
                    let y = iter.next().unwrap();
                    let z = iter.next().unwrap();
                    temp_vertices.push(glm::vec3(x, y, z));

                    min_aabb = glm::vec3(min_aabb.x.min(x), min_aabb.y.min(y), min_aabb.z.min(z));
                    max_aabb = glm::vec3(max_aabb.x.max(x), max_aabb.y.max(y), max_aabb.z.max(z));

                }
                Some(ObjToken::Normal) => {
                    let mut iter = iter
                        .take(3)
                        .map(|i| i.parse::<f32>().unwrap());
                    let x = iter.next().unwrap();
                    let y = iter.next().unwrap();
                    let z = iter.next().unwrap();
                    normals.push(glm::vec3(x, y, z));
                }
                Some(ObjToken::TexCoord) => {
                    let mut iter = iter
                        .take(2)
                        .map(|i| i.parse::<f32>().unwrap());
                    let u = iter.next().unwrap();
                    let v = iter.next().unwrap();
                    tex_coords.push(glm::vec2(u, v));
                }
                Some(ObjToken::Face) => {
                    // TODO: vertex indices can be negative

                    let face = iter.collect::<Vec<_>>();
                    let mut calculated_normal = glm::vec3(0.0, 0.0, 0.0);

                    if normals.is_empty() {
                        let part0 = face[0].split("/").next().unwrap();
                        let part1 = face[1].split("/").next().unwrap();
                        let part2 = face[2].split("/").next().unwrap();

                        calculated_normal = glm::normalize(glm::cross(
                            temp_vertices[part1.parse::<i32>().unwrap() as usize - 1] - temp_vertices[part0.parse::<i32>().unwrap() as usize - 1],
                            temp_vertices[part2.parse::<i32>().unwrap() as usize - 1] - temp_vertices[part0.parse::<i32>().unwrap() as usize - 1]
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
                        } else if vert.matches("/").count() == 2 {
                            let mut it = vert.split("/");
                            let vertex = it.next().unwrap().parse::<i32>().unwrap() - 1;
                            let t_coords = it.next().unwrap().parse::<i32>().unwrap() - 1;
                            let normal = it.next().unwrap().parse::<i32>().unwrap() - 1;
                            vertices.push(Vertex{
                                position: *temp_vertices.get(vertex as usize).unwrap(),
                                normal: *normals.get(normal as usize).unwrap(),
                                tex_coords: *tex_coords.get(t_coords as usize).unwrap()
                            });
                        } else if vert.matches("/").count() == 1 {
                            let mut it = vert.split("/");
                            let vertex = it.next().unwrap().parse::<i32>().unwrap() - 1;
                            let t_coords = it.next().unwrap().parse::<i32>().unwrap() - 1;
                            vertices.push(Vertex{
                                position: *temp_vertices.get(vertex as usize).unwrap(),
                                normal: calculated_normal,
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
                Some(ObjToken::MaterialLib) => {
                    let material_path = obj_path.parent().unwrap().join(iter.next().unwrap_or(""));
                    let new_materials = parse_mtl(&material_path);
                    match new_materials {
                        Ok(m) => {
                            materials.extend(m);
                        },
                        Err(e) => {
                            error!("Failed to parse mtl file {:?}: {}", material_path, e);
                        }
                    }
                }
                Some(ObjToken::MaterialUsage) => {
                    // Split into meshes by material usage
                    let name = {
                        if current_mesh_name.is_empty() && !object_name.is_empty() {
                            object_name.clone()
                        } else if !current_mesh_name.is_empty() {
                            current_mesh_name.clone()
                        } else {
                            "default_mesh".to_string()
                        }
                    };
                    if !vertices.is_empty() {
                        meshes.push(ObjMesh{
                            name,
                            vertices: vertices.clone(),
                            indices: indices.clone(),
                            material: current_material.clone()
                        });
                    }
                    vertices.clear();
                    indices.clear();
                    indices_counter = 0;

                    let mat_name = iter.next().unwrap_or("").to_string();
                    current_material = materials.get(&mat_name).cloned();
                }
                Some(ObjToken::Line) | Some(ObjToken::Point) => {
                    // we don't handle lines or points
                }
                _ => { warn!("Unhandled obj token: {}", token) },
            }
        }
    }
    let elapsed = now.elapsed();
    trace!("Loaded in {}ms",  elapsed.as_millis());

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
        material: current_material
    });

    let aabb = AABB::new(min_aabb, max_aabb);

    Ok(Object{
        name: object_name,
        meshes,
        aabb,
    })
}
