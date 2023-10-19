mod obj;

use std::path::PathBuf;

use crate::{mesh::Vertex, aabb::AABB};

#[derive(Debug)]
pub struct ObjMesh {
    pub name: String,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

#[derive(Debug)]
pub struct Object {
    pub name: String,
    pub meshes: Vec<ObjMesh>,
    pub aabb: AABB
}

pub fn load_from_file(path: &PathBuf) -> Result<Object, Box<dyn std::error::Error>> {
    // FIXME: don't explode if to_str() fails
    let path_str = path.to_str().unwrap();
    let file = std::fs::File::open(path_str)?;
    // TODO: support both stl and obj based on file extension
    // if no extension, then test for binary STL magic bytes 
    // if no magic bytes, then try to guess based on the first line of text in the file

    let obj = obj::load_obj(file)?;
    Ok(obj)
}
