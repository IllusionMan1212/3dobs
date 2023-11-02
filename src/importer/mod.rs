mod obj;
mod stl;

use std::path::PathBuf;

use crate::{mesh::Vertex, aabb::AABB, utils::{SupportedFileExtensions, self}};

#[derive(Debug, Clone)]
pub enum TextureType {
    Ambient,
    Diffuse,
    Specular,
    SpecularHighlight,
    Bump,
    Displacement,
    Decal,
    Reflection,
    Emissive
}

impl TextureType {
    pub fn from_material_str(token: &str) -> Option<Self> {
        match token {
            "map_Ka" => Some(TextureType::Ambient),
            "map_Kd" => Some(TextureType::Diffuse),
            "map_Ks" => Some(TextureType::Specular),
            "map_Ns" => Some(TextureType::SpecularHighlight),
            "map_bump" => Some(TextureType::Bump),
            "map_d" => Some(TextureType::Displacement),
            "decal" => Some(TextureType::Decal),
            "refl" => Some(TextureType::Reflection),
            "map_Ke" => Some(TextureType::Emissive),
            _ => None
        }
    }
}

#[derive(Debug, Clone)]
pub struct Material {
    pub name: String,
    pub ambient_color: glm::Vec3,
    pub diffuse_color: glm::Vec3,
    pub specular_color: glm::Vec3,
    pub specular_exponent: f32,
    pub opacity: f32,
    pub textures: Vec<Texture>
}

impl Material {
    fn new(name: String, ambient: glm::Vec3, diffuse: glm::Vec3, specular: glm::Vec3, shininess: f32, opacity: f32, textures: Vec<Texture>) -> Self {
        Self {
            name,
            ambient_color: ambient,
            diffuse_color: diffuse,
            specular_color: specular,
            specular_exponent: shininess,
            opacity,
            textures
        }
    }
}

impl Default for Material {
    fn default() -> Self {
        Self {
            name: "default_mat".to_string(),
            ambient_color: glm::vec3(0.4, 0.4, 0.4),
            diffuse_color: glm::vec3(0.7, 0.7, 0.7),
            specular_color: glm::vec3(0.1, 0.1, 0.1),
            specular_exponent: 32.0,
            opacity: 1.0,
            textures: Vec::new()
        }
    }
}

impl std::fmt::Display for Material {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Ambient: {:?}\nDiffuse: {:?}\nSpecular: {:?}\nShininess: {}\nOpacity: {}\nTextures: {:?}",
            self.ambient_color,
            self.diffuse_color,
            self.specular_color,
            self.specular_exponent,
            self.opacity,
            self.textures
            )
    }
}

#[derive(Clone, Debug)]
pub struct Texture {
    pub id: u32,
    pub typ: TextureType,
}

impl Texture {
    pub fn new(path: std::path::PathBuf, typ: TextureType) -> Result<Self, Box<dyn std::error::Error>> {
        let id = utils::load_texture(path)?;

        Ok(Texture {
            id,
            typ,
        })
    }
}


#[derive(Debug)]
pub struct ObjMesh {
    pub name: String,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub material: Option<Material>,
}

#[derive(Debug)]
pub struct Object {
    pub name: String,
    pub meshes: Vec<ObjMesh>,
    pub aabb: AABB,
}

pub fn load_from_file(path: &PathBuf) -> Result<Object, Box<dyn std::error::Error>> {
    let path_str = match path.to_str() {
        Some(s) => s,
        None => return Err("Failed to convert path to string".into())
    };

    let file = std::fs::File::open(path_str)?;
    // TODO: if no extension, then test for binary STL magic bytes 
    // if no magic bytes, then try to guess based on the first line of text in the file

    let obj = match SupportedFileExtensions::from_str(path.extension().unwrap().to_str().unwrap()) {
        Some(SupportedFileExtensions::STL) => stl::load_stl(file)?,
        Some(SupportedFileExtensions::OBJ) => obj::load_obj(path, file)?,
        _ => panic!("Unsupported file extension: {}", path_str),
    };

    Ok(obj)
}
