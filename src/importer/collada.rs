use std::{io::BufReader, collections::HashMap};

use crate::{importer::{Object, ObjMesh}, aabb::AABB, mesh::{Mesh, Vertex}};

use log::{info, warn};
use xml::reader::{EventReader, XmlEvent};

enum ColladaElement {
    // Asset,
    // LibraryAnimations,
    // LibraryAnimationClips,
    // LibraryCameras,
    // LibraryControllers,
    // LibraryForceFields,
    // LibraryImages,
    // LibraryLights,
    // LibraryNodes,
    // LibraryPhysicsMaterials,
    // LibraryPhysicsModels,
    // LibraryPhysicsScenes,
    // LibraryVisualScenes,
    // Scene,
}

#[derive(Debug)]
enum SupportedElement {
    None,

    Unit,
    UpAxis,

    LibraryMaterials,
    Material,
    InstanceEffect,

    LibraryEffects,
    Effect,
    Technique, // might remove this
    Diffuse, // vec4
    Shininess,

    LibraryGeometries,
    Geometry,
    Mesh,
    Source,
    Input,
    FloatArray,
    Polylist,
    VCount,
    Triangles,
    P,
}

impl SupportedElement {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "unit" => Some(Self::Unit),
            "up_axis" => Some(Self::UpAxis),

            "library_materials" => Some(Self::LibraryMaterials),
            "material" => Some(Self::Material),
            "instance_effect" => Some(Self::InstanceEffect),

            "library_effects" => Some(Self::LibraryEffects),
            "effect" => Some(Self::Effect),
            "technique" => Some(Self::Technique),
            "diffuse" => Some(Self::Diffuse),
            "shininess" => Some(Self::Shininess),

            "library_geometries" => Some(Self::LibraryGeometries),
            "geometry" => Some(Self::Geometry),
            "mesh" => Some(Self::Mesh),
            "source" => Some(Self::Source),
            "input" => Some(Self::Input),
            "float_array" => Some(Self::FloatArray),
            "polylist" => Some(Self::Polylist),
            "vcount" => Some(Self::VCount),
            "triangles" => Some(Self::Triangles),
            "p" => Some(Self::P),

            _ => None,
        }
    }
}

pub fn load_dae(file: std::fs::File) -> Result<Object, Box<dyn std::error::Error>> {
    let reader = BufReader::new(file);
    let parser = EventReader::new(reader);

    let mut chunk_p_by = 0;
    let mut current_face_count = 0;

    let mut current_source_id = String::new();
    let mut current_element = SupportedElement::None;
    let mut sources = HashMap::<String, Vec<f32>>::new();
    let mut vcount = Vec::new();
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut tex_coords = Vec::new();

    let mut vertices: Vec<Vertex> = Vec::new();
    let mut indices = Vec::new();
    // let meshes = Vec::new();
    // let materials = Vec::new();

    let mut min_aabb = glm::vec3(f32::MAX, f32::MAX, f32::MAX);
    let mut max_aabb = glm::vec3(f32::MIN, f32::MIN, f32::MIN);

    let now = std::time::Instant::now();
    for e in parser {
        match e {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                current_element = match SupportedElement::from_str(&name.local_name) {
                    Some(element) => element,
                    None => {
                        // warn!("Unsupported element: {}", name.local_name);
                        SupportedElement::None
                    }
                };

                match current_element {
                    SupportedElement::Unit => {},
                    SupportedElement::UpAxis => {},
                    SupportedElement::LibraryMaterials => {},
                    SupportedElement::Source => {
                        if let Some(source_id) = attributes.into_iter().find(|a| a.name.local_name == "id") {
                            current_source_id = source_id.value
                        }
                    },
                    SupportedElement::Input => {
                        let semantic = &attributes.iter().find(|a| a.name.local_name == "semantic").unwrap().value;
                        // strip the # at the beginning of the source id
                        let source_id = &attributes.iter().find(|a| a.name.local_name == "source").unwrap().value[1..];

                        match semantic.as_str() {
                            "POSITION" => {
                                positions = sources.get(source_id)
                                    .unwrap()
                                    .chunks(3)
                                    .map(|v| {
                                        let vertex = glm::vec3(v[0], v[1], v[2]);

                                        min_aabb = glm::vec3(
                                            min_aabb.x.min(vertex.x),
                                            min_aabb.y.min(vertex.y),
                                            min_aabb.z.min(vertex.z),
                                        );

                                        max_aabb = glm::vec3(
                                            max_aabb.x.max(vertex.x),
                                            max_aabb.y.max(vertex.y),
                                            max_aabb.z.max(vertex.z),
                                        );

                                        vertex
                                    })
                                    .collect::<Vec<_>>();
                            }
                            "NORMAL" => {
                                chunk_p_by += 1;

                                normals = sources.get(source_id)
                                    .unwrap()
                                    .chunks(3)
                                    .map(|v| glm::vec3(v[0], v[1], v[2]))
                                    .collect::<Vec<_>>();
                            },
                            "TEXCOORD" => {
                                chunk_p_by += 1;

                                tex_coords = sources.get(source_id)
                                    .unwrap()
                                    .chunks(2)
                                    .map(|v| glm::vec2(v[0], v[1]))
                                    .collect::<Vec<_>>();
                            }
                            // ignore this since it's used to refer to the vertices
                            // tag and we already parse POSITION from that
                            "VERTEX" => {
                                chunk_p_by += 1;
                            }, 
                            _ => { panic!("Unknown <input> semantic: {}", semantic) }
                        }
                    },
                    // This element is used to indicate that the vcount
                    // aka vertex count for every face is 3 without having to
                    // explicity write the vcount element
                    SupportedElement::Triangles => {
                        current_face_count = attributes.into_iter().find(|a| a.name.local_name == "count").unwrap().value.parse::<usize>().unwrap();

                        // preemptively allocate the exact amount of vertices we want
                        vertices = Vec::with_capacity(current_face_count * 3);

                        // fill vcount with all 3s for as many faces as we got
                        // so we can use it later when parsing <p>
                        vcount = vec![3; current_face_count]
                    },
                    // This element can have a varying amount of vertices
                    // per face and the vcount element is required to be present (i think)
                    SupportedElement::Polylist => {
                        current_face_count = attributes.into_iter().find(|a| a.name.local_name == "count").unwrap().value.parse::<usize>().unwrap();

                        vcount = Vec::with_capacity(current_face_count);
                    }
                    SupportedElement::Geometry => {
                        // TODO: name attribute for the object? name
                    },
                    _ => {} // Other elements we don't care about (should probably only be None
                            // here and we can nuke the elements that we don't use from the enum)
                }
            }
            Ok(XmlEvent::EndElement { name }) => {
                match SupportedElement::from_str(&name.local_name) {
                    Some(SupportedElement::P) => {
                        // reset chunk_by to 0 so we know we're done with the
                        // current polylist
                        chunk_p_by = 0;
                    },
                    _ => { /*warn!("Unsupported element: {}", name.local_name)*/ },
                }
                // TODO: each polylist has a material. meaning we split the mesh
                // into multiple meshes according to the polylists.
                // Some models don't have polylist and instead have a triangles

                // TODO: if we encounter mesh, we push it to the vector

                // Mesh::new();
                // println!("End: {}", name);
            }
            Ok(XmlEvent::Characters(data)) => {
                match current_element {
                    SupportedElement::FloatArray => {
                        sources.insert(
                            current_source_id.clone(),
                            data
                            .trim()
                            .lines()
                            .map(|l| l
                                .split_ascii_whitespace()
                                .map(|v| v.parse::<f32>().unwrap())
                                .collect::<Vec<_>>())
                            .flatten()
                            .collect::<Vec<_>>()
                        );
                    },
                    SupportedElement::P => {
                        // TODO: get current_v_count by looping over the faces??
                        // current_v_count * chunk_p_by that's 1 v count
                        // we also need to triangulate anything with >3 v_count
                        // vertices = data
                        //     .trim()
                        //     .lines()
                        //     .map(|l| l
                        //         .split_ascii_whitespace()
                        //         .map(|i| i.parse::<usize>().unwrap())
                        //         .collect::<Vec<_>>())
                        //     .flatten()
                        //     .collect::<Vec<_>>()
                        //     .chunks(chunk_p_by)
                        //     .map(|i| Vertex::new(positions[i[0]], normals[i[1]], tex_coords[i[2]]))
                        // .collect::<Vec<_>>();

                        // println!("P: {:?}", vertices.len());

                        // println!("data: {}", data);
                        let data = data
                            .trim()
                            .split_ascii_whitespace()
                            .map(|i| i.parse::<usize>().unwrap())
                            .collect::<Vec<_>>();

                        for i in 0..current_face_count {
                            let current_v_count = vcount.get(i).unwrap();
                            // TODO: triangulate face if vcount >3
                            // TODO: get vertices and indices in one go

                            let face = data
                                .chunks(chunk_p_by * current_v_count)
                                .skip(i)
                                .take(1)
                                .flatten()
                                .collect::<Vec<_>>();

                            // NOTE: no idea if this is fine and doesn't break things
                            if face.len() == 0 { continue; }


                            // HACK: this assumes that a face only has vertices
                            // for faces with more, we need to triangulate and then
                            // push the indices properly (dk how yet)
                            indices.push(i as u32 * 3);
                            indices.push(i as u32 * 3 + 1);
                            indices.push(i as u32 * 3 + 2);

                            // normals for this face
                            let normals = {
                                // TODO: if we don't have normals. we should generate them
                                // also take v_count into account here so we triangulate
                                // the normals immediate as well ???
                                // the lternative is to do this twice in each branch
                                // of the coming if statement. idk
                            };

                            // tex_coords for this face
                            let tex_coords = {
                                // TODO: if we don't have tex_coords. return 0.0, 0.0
                            };

                            if *current_v_count > 3 {
                                // triangulate
                            } else {
                                // vertices.push(Vertex::new());
                            }

                            vertices.extend(face
                                .chunks(chunk_p_by)
                                // TODO: v[0], v[1], v[2] is bad cuz im not using chunk_p_by
                                .map(|v| Vertex::new(positions[*v[0]], normals[*v[1]], tex_coords[*v[2]]))
                                .collect::<Vec<_>>()
                            );
                        }
                    },
                    SupportedElement::VCount => {
                        vcount = data
                            .trim()
                            .lines()
                            .map(|l| l
                                .split_ascii_whitespace()
                                .map(|c| c.parse::<usize>().unwrap())
                                .collect::<Vec<_>>())
                            .flatten()
                            .collect::<Vec<_>>();

                        // println!("vcount: {:?}", vcount);
                    },
                    _ => {
                        // warn!("Unsupported Characters event for element: {:?}", current_element);
                    }
                }
            }
            Err(e) => {
                return Err(Box::new(e));
            }
            _ => {}
        }
    }

    // println!("vcount: {:?}", vcount);
    println!("Vertices: {:?}", vertices);
    println!("Indices: {:?}", indices);
    // println!("Normals: {:?}", normals);
    // println!("TexCoords: {:?}", tex_coords);
    let elapsed = now.elapsed();
    info!("Loaded in {}ms",  elapsed.as_millis());

    todo!();

    let aabb = AABB::new(min_aabb, max_aabb);

    // Ok(Object {
    //     name: "test".to_string(),
    //     meshes: vec![ObjMesh{
    //         name: "test".to_string(),
    //         vertices,
    //         indices,
    //         material: None
    //     }],
    //     aabb
    // })
}
