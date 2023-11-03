use std::{io::{BufReader, BufRead}, collections::HashMap, path::PathBuf};

use log::{error, warn, trace};

use crate::{mesh::Vertex, aabb::AABB, importer::{ObjMesh, Object, Material, Texture, TextureType}};

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
    Transparency,
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
            "Tr" => Some(MtlToken::Transparency),
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

fn parse_mtl(path: &PathBuf, obj_textures: &mut HashMap<String, Texture>) -> Result<HashMap<String, Material>, Box<dyn std::error::Error>> {
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
    let mut mat_textures: Vec<Texture> = Vec::new();

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
                        material = Material::new(material_name.clone(), ambient, diffuse, specular, shininess, opacity, mat_textures.clone());
                        materials.insert(material_name, material);

                        mat_textures.clear();
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
                Some(MtlToken::Transparency) => {
                    // it's just opposite of opacity so we subtract it from 1.0
                    opacity = 1.0 - iter.next().unwrap().parse::<f32>().unwrap();
                }
                Some(MtlToken::DiffuseTexture)
                | Some(MtlToken::AmbientTexture)
                | Some(MtlToken::SpecularTexture)
                | Some(MtlToken::EmissiveTexture) => {
                    let tex_type = TextureType::from_material_str(token).unwrap();

                    let name = iter.next().unwrap().to_string();
                    let tex = if obj_textures.contains_key(&name) {
                        let mut tex = obj_textures.get(&name).unwrap().clone();

                        tex.typ = tex_type;
                        tex
                    } else {
                        let path = path.parent().unwrap().join(&name);
                        let tex = match Texture::new(path, tex_type) {
                            Ok(v) => v,
                            Err(e) => {
                                error!("Failed to load texture: {}", e);
                                continue;
                            }
                        };

                        obj_textures.insert(name, tex.clone());
                        tex
                    };

                    mat_textures.push(mat);
                }
                _ => { warn!("Unhandled material token: {}", token) },
            }
        }
    }

    material = Material::new(material_name.clone(), ambient, diffuse, specular, shininess, opacity, mat_textures);

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
    let mut textures = HashMap::new();

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
                    let vec = iter.collect::<Vec<_>>();
                    if vec.len() < 3 {
                        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Incomplete vertex data")));
                    }

                    let mut iter = vec.iter()
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
                    let vec = iter.collect::<Vec<_>>();
                    if vec.len() < 3 {
                        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Incomplete vertex normal data")));
                    }

                    let mut iter = vec.iter()
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
                    // vertically flip the texcoords because flipping the texture is expensive
                    tex_coords.push(glm::vec2(u, 1.0 - v));
                }
                Some(ObjToken::Face) => {
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
                            let mut vert = it.next().unwrap().parse::<i32>().unwrap();
                            if vert < 0 {
                                vert = temp_vertices.len() as i32 + vert;
                            } else {
                                vert -= 1;
                            }
                            let mut normal = it.next().unwrap().parse::<i32>().unwrap();
                            if normal < 0 {
                                normal = normals.len() as i32 + normal;
                            } else {
                                normal -= 1;
                            }
                            vertices.push(Vertex{
                                position: *temp_vertices.get(vert as usize).unwrap(),
                                normal: *normals.get(normal as usize).unwrap(),
                                tex_coords: glm::vec2(0.0, 0.0)
                            });
                        } else if vert.matches("/").count() == 2 {
                            let mut it = vert.split("/");
                            let mut vertex = it.next().unwrap().parse::<i32>().unwrap();
                            if vertex < 0 {
                                vertex = temp_vertices.len() as i32 + vertex;
                            } else {
                                vertex -= 1;
                            }
                            let mut t_coords = it.next().unwrap().parse::<i32>().unwrap();
                            if t_coords < 0 {
                                t_coords = tex_coords.len() as i32 + t_coords;
                            } else {
                                t_coords -= 1;
                            }
                            let mut normal = it.next().unwrap().parse::<i32>().unwrap();
                            if normal < 0 {
                                normal = normals.len() as i32 + normal;
                            } else {
                                normal -= 1;
                            }
                            vertices.push(Vertex{
                                position: *temp_vertices.get(vertex as usize).unwrap(),
                                normal: *normals.get(normal as usize).unwrap(),
                                tex_coords: *tex_coords.get(t_coords as usize).unwrap()
                            });
                        } else if vert.matches("/").count() == 1 {
                            let mut it = vert.split("/");
                            let mut vertex = it.next().unwrap().parse::<i32>().unwrap();
                            if vertex < 0 {
                                vertex = temp_vertices.len() as i32 + vertex;
                            } else {
                                vertex -= 1;
                            }
                            let mut t_coords = it.next().unwrap().parse::<i32>().unwrap();
                            if t_coords < 0 {
                                t_coords = tex_coords.len() as i32 + t_coords;
                            } else {
                                t_coords -= 1;
                            }
                            vertices.push(Vertex{
                                position: *temp_vertices.get(vertex as usize).unwrap(),
                                normal: calculated_normal,
                                tex_coords: *tex_coords.get(t_coords as usize).unwrap()
                            });
                        } else {
                            let mut vert = vert.parse::<i32>().unwrap();
                            if vert < 0 {
                                vert = temp_vertices.len() as i32 + vert;
                            } else {
                                vert -= 1;
                            }
                            vertices.push(Vertex{
                                position: *temp_vertices.get(vert as usize).unwrap(),
                                normal: calculated_normal,
                                tex_coords: glm::vec2(0.0, 0.0),
                            });
                        }

                        // Triangulate faces. 2 triangles per face
                        if i < face.len() - 2 {
                            indices.push(indices_counter);
                            indices.push(indices_counter + i as u32 + 1);
                            indices.push(indices_counter + i as u32 + 2);
                        }
                    }

                    indices_counter += face.len() as u32;
                }
                Some(ObjToken::MaterialLib) => {
                    for matlib in iter {
                        let material_path = obj_path.parent().unwrap().join(matlib);
                        let new_materials = parse_mtl(&material_path, &mut textures);
                        match new_materials {
                            Ok(m) => {
                                materials.extend(m);
                            },
                            Err(e) => {
                                error!("Failed to parse mtl file {:?}: {}", material_path, e);
                            }
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
                // Things we ignore have a statement to not clutter the log
                Some(ObjToken::Line) | Some(ObjToken::Point) => {
                    // we don't handle lines or points
                }
                Some(ObjToken::SmoothShading) => {
                    // idc about this
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
