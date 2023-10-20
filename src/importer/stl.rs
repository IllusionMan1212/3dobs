use std::io::{Read, Seek, BufReader};

use crate::{importer::ObjMesh, importer::Object, importer::Material, aabb::AABB, mesh::Vertex};

const STL_HEADER_SIZE: u64 = 80;
// might not be enough since it's not guaranteed. we'll see
const ASCII_STL_MAGIC: [u8; 11] = [0x73, 0x6F, 0x6C, 0x69, 0x64, 0x20, 0x61, 0x73, 0x63, 0x69, 0x69]; // "solid ascii"

#[repr(packed(2))]
struct STLTriangle {
    normal: glm::Vec3,
    verts: [glm::Vec3; 3],
    attribute_byte_count: u16
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
            buf: vec![0u8; 50],
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

fn parse_ascii_stl(file: std::fs::File) -> Result<Object, Box<dyn std::error::Error>> {
    // TODO:
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

            if vert.x < min_aabb.x {
                min_aabb.x = vert.x;
            }

            if vert.y < min_aabb.y {
                min_aabb.y = vert.y;
            }

            if vert.z < min_aabb.z {
                min_aabb.z = vert.z;
            }

            if vert.x > max_aabb.x {
                max_aabb.x = vert.x;
            }

            if vert.y > max_aabb.y {
                max_aabb.y = vert.y;
            }

            if vert.z > max_aabb.z {
                max_aabb.z = vert.z;
            }
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

pub fn load_stl(mut file: std::fs::File) -> Result<Object, Box<dyn std::error::Error>> {
    let mut buf: [u8; 11] = [0; 11];
    let _ = file.read_exact(&mut buf);
    _ = file.seek(std::io::SeekFrom::Start(0));
    let is_ascii = buf == ASCII_STL_MAGIC;

    let obj = if is_ascii {
        parse_ascii_stl(file)?
    } else {
        parse_binary_stl(file)?
    };
    let now = std::time::Instant::now();
    let obj = parse_binary_stl(file)?;
    let elapsed = now.elapsed();
    println!("STL file loaded in {} ms", elapsed.as_millis());

    Ok(obj)
}
