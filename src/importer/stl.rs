use std::io::{Read, Seek, BufReader, BufRead};

use crate::{importer::ObjMesh, importer::Object, importer::Material, aabb::AABB, mesh::Vertex};

const STL_HEADER_SIZE: u64 = 80;
const STL_TRIANGLE_SIZE: usize = 50;

#[repr(packed(2))]
#[derive(Debug)]
struct STLTriangle {
    normal: glm::Vec3,
    verts: [glm::Vec3; 3],
    attribute_byte_count: u16
}

struct FacetIterator<R: Read> {
    reader: BufReader<R>,
}

impl<R: Read> FacetIterator<R> {
    fn new(reader: R) -> Self {
        FacetIterator {
            reader: BufReader::new(reader),
        }
    }
}

impl<R: Read> Iterator for FacetIterator<R> {
    type Item = STLTriangle;

    fn next(&mut self) -> Option<STLTriangle> {
        let mut line = String::with_capacity(0x50);
        let mut normal = glm::vec3(0.0, 0.0, 0.0);
        let mut vertices = Vec::with_capacity(3);

        loop {
            match self.reader.read_line(&mut line) {
                Ok(0) => return None, // Reached EOF
                Ok(_) => {
                    if line.contains("normal") {
                        let mut norm_iter = line.split_whitespace().skip(2).map(|s| s.parse::<f32>().unwrap());
                        normal.x = norm_iter.next().unwrap();
                        normal.y = norm_iter.next().unwrap();
                        normal.z = norm_iter.next().unwrap();
                    } else if line.contains("vertex") {
                        let mut vertex_iter = line.split_whitespace().skip(1).map(|s| s.parse::<f32>().unwrap());
                        vertices.push(glm::vec3(vertex_iter.next().unwrap(), vertex_iter.next().unwrap(), vertex_iter.next().unwrap()));
                    } else if line.contains("endfacet") {
                        break;
                    }
                }
                // TODO: might need to panic and log here in the future
                Err(_) => return None, // Error reading
            }
            line.clear();
        }

        let mut verts_iter = vertices.iter();

        Some(STLTriangle{
            normal,
            verts: [*verts_iter.next().unwrap(), *verts_iter.next().unwrap(), *verts_iter.next().unwrap()],
            attribute_byte_count: 0
        })
    }
}

fn parse_ascii_stl(file: std::fs::File) -> Result<Object, Box<dyn std::error::Error>> {
    let mut reader = BufReader::new(file);
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut min_aabb = glm::vec3(f32::MAX, f32::MAX, f32::MAX);
    let mut max_aabb = glm::vec3(f32::MIN, f32::MIN, f32::MIN);

    reader.read_line(&mut String::new())?; // Skip the first line (solid name)
    let facet_iter = FacetIterator::new(reader);

    let tex_coords = glm::vec2(0.0, 0.0);

    for (i, triangle) in facet_iter.enumerate() {
        for vert in triangle.verts {
            vertices.push(Vertex{
                position: glm::vec3(vert.x, vert.y, vert.z),
                normal: glm::vec3(triangle.normal.x, triangle.normal.y, triangle.normal.z),
                tex_coords,
            });

            min_aabb = glm::min(min_aabb, vert);
            max_aabb = glm::max(max_aabb, vert);
        }

        indices.push(i as u32 * 3);
        indices.push(i as u32 * 3 + 1);
        indices.push(i as u32 * 3 + 2);
    }

    let aabb = AABB::new(min_aabb, max_aabb);

    Ok(Object{
        name: "default_object".to_string(),
        meshes: vec![ObjMesh{
            name: "default_mesh".to_string(),
            vertices,
            indices,
            material: Some(Material::default())
        }],
        aabb,
    })
} 

#[derive(Debug)]
struct TrianglesIter<R: Read> {
    reader: R,
    buf: Vec<u8>,
    triangles_to_read: usize,
    triangles_read: usize,
}

impl<R: Read> TrianglesIter<R> {
    fn new(reader: R, triangles_to_read: usize) -> Self {
        TrianglesIter {
            reader,
            buf: vec![0u8; STL_TRIANGLE_SIZE],
            triangles_to_read,
            triangles_read: 0,
        }
    }
}

impl<R: Read> Iterator for TrianglesIter<R> {
    type Item = STLTriangle;

    fn next(&mut self) -> Option<STLTriangle> {
        if self.triangles_read >= self.triangles_to_read {
            None
        } else {
            let _ = self.reader.read_exact(&mut self.buf);

            self.triangles_read += 1;

            unsafe {
                let normal = (self.buf[0..12].as_ptr() as *const glm::Vec3).read();
                let verts = [
                    (self.buf[12..24].as_ptr() as *const glm::Vec3).read(),
                    (self.buf[24..36].as_ptr() as *const glm::Vec3).read(),
                    (self.buf[36..48].as_ptr() as *const glm::Vec3).read(),
                ];
                let abc = u16::from_le_bytes([self.buf[48], self.buf[49]]);

                Some(STLTriangle{
                    verts,
                    normal,
                    attribute_byte_count: abc
                })
            }
        }
    }
}

fn parse_binary_stl(mut file: std::fs::File) -> Result<Object, Box<dyn std::error::Error>> {
    // skip header for now
    file.seek(std::io::SeekFrom::Start(STL_HEADER_SIZE))?;

    let mut buf: [u8; 4] = [0; 4];
    file.read(&mut buf)?;
    let tri_count: u32 = u32::from_le_bytes(buf);

    let mut min_aabb = glm::vec3(f32::MAX, f32::MAX, f32::MAX);
    let mut max_aabb = glm::vec3(f32::MIN, f32::MIN, f32::MIN);
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let reader = BufReader::new(file);

    let triangles_reader = TrianglesIter::new(reader, tri_count as usize);

    let tex_coords = glm::vec2(0.0, 0.0);
    for (i, triangle) in triangles_reader.enumerate() {
        for vert in triangle.verts {
            vertices.push(Vertex{
                position: glm::vec3(vert.x, vert.y, vert.z),
                normal: glm::vec3(triangle.normal.x, triangle.normal.y, triangle.normal.z),
                tex_coords,
            });

            min_aabb = glm::min(min_aabb, vert);
            max_aabb = glm::max(max_aabb, vert);
        }

        indices.push(i as u32*3);
        indices.push(i as u32*3 + 1);
        indices.push(i as u32*3 + 2);
    }

    let aabb = AABB::new(min_aabb, max_aabb);

    Ok(Object{
        name: "default_object".to_string(),
        meshes: vec![ObjMesh{
            name: "default_mesh".to_string(),
            vertices,
            indices,
            material: Some(Material::default())
        }],
        aabb,
    })
} 

fn is_ascii(buf: &[u8]) -> bool {
    for b in buf {
        if *b > 127 {
            return false;
        }
    }

    true
}

pub fn load_stl(mut file: std::fs::File) -> Result<Object, Box<dyn std::error::Error>> {
    let mut buf: [u8; 512] = [0; 512];
    let _ = file.read_exact(&mut buf);
    _ = file.seek(std::io::SeekFrom::Start(0));
    let is_ascii = is_ascii(&buf);

    let now = std::time::Instant::now();
    let obj = if is_ascii {
        parse_ascii_stl(file)?
    } else {
        parse_binary_stl(file)?
    };
    let elapsed = now.elapsed();
    println!("Loaded in {} ms", elapsed.as_millis());

    Ok(obj)
}
