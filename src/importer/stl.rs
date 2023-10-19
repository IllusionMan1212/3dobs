struct STLHeader {
    header: [u8; 80],
    num_triangles: u32,
}

struct Vertex {
    x: f32,
    y: f32,
    z: f32,
}

struct Triangle {
    normal: [f32; 3],
    vertices: [Vertex; 3],
    attribute_byte_count: u16,
}

struct STL {
    header: STLHeader,
    triangles: Vec<Triangle>,
}

impl STL {
    pub fn new(stl: Vec<u8>) -> () {
        // TODO:
    }
}
