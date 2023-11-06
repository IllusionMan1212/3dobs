use std::io::{Read, Seek, BufReader};

use log::info;

use crate::importer::Object;

const BINARY_FBX_MAGIC: &[u8; 21] = b"Kaydara FBX Binary  \x00";

#[derive(Debug)]
struct FbxProperty {
    data: String,
}

#[derive(Debug)]
struct FbxNode {
    name: String,
    properties: Vec<FbxProperty>,
    children: Vec<FbxNode>,
}

struct FbxRecordIterator<R: Read + Seek> {
    reader: BufReader<R>,
}

impl<R: Read + Seek> FbxRecordIterator<R> {
    fn new(reader: BufReader<R>) -> Self {
        FbxRecordIterator {
            reader,
        }
    }
}

impl<R: Read + Seek> Iterator for FbxRecordIterator<R> {
    type Item = FbxNode;

    fn next(&mut self) -> Option<Self::Item> {
        let mut children = Vec::new();

        // NOTE: these are 4 bytes only in versions <7.5. 7.5+ uses 8 bytes
        let mut end_offset = [0; 4];
        let mut num_properties = [0; 4];
        let mut property_list_len = [0; 4];
        let mut name_len = [0; 1];
        self.reader.read_exact(&mut end_offset).unwrap();
        self.reader.read_exact(&mut num_properties).unwrap();
        self.reader.read_exact(&mut property_list_len).unwrap();
        self.reader.read_exact(&mut name_len).unwrap();

        let end_offset = u32::from_le_bytes(end_offset);
        let num_properties = u32::from_le_bytes(num_properties);
        let property_list_len = u32::from_le_bytes(property_list_len);
        let name_len = u8::from_le_bytes(name_len);

        if name_len == 0 {
            return None;
        }

        let name = String::from_utf8(self.reader.by_ref().take(name_len as _).bytes().map(|b| b.unwrap()).collect()).unwrap();

        // println!("Name: {}", name);

        // self.reader.seek_relative(property_list_len as _).unwrap(); // skip the properties
        let properties = parse_properties(self.reader.by_ref(), num_properties, property_list_len);

        // if there's still data left before reaching the end of the record,
        // then that means there are child record nodes
        while self.reader.stream_position().unwrap() < end_offset as _ {
            if let Some(child) = Self::next(self) {
                children.push(child);
            }
        }

        Some(FbxNode{
            name,
            properties,
            children
        })
    }
}

enum TypeCode {
    Short,
    Int,
    Long,
    Float,
    Double,
    Bool,

    FloatArray,
    DoubleArray,
    LongArray,
    IntArray,
    BoolArray,

    String,
    Raw,
}

impl TypeCode {
    fn from_byte(byte: u8) -> Self {
        match byte {
            // Primitive types
            b'Y' => TypeCode::Short,
            b'C' => TypeCode::Bool,
            b'I' => TypeCode::Int,
            b'F' => TypeCode::Float,
            b'D' => TypeCode::Double,
            b'L' => TypeCode::Long,

            // Array types
            b'f' => TypeCode::FloatArray,
            b'd' => TypeCode::DoubleArray,
            b'l' => TypeCode::LongArray,
            b'i' => TypeCode::IntArray,
            b'b' => TypeCode::BoolArray,

            // Special types
            b'S' => TypeCode::String,
            b'R' => TypeCode::Raw,

            _ => { panic!("Invalid property TypeCode: {}", std::str::from_utf8(&[byte]).unwrap()) }
        }
    }
}

fn parse_properties<R: Read + Seek>(reader: &mut BufReader<R>, num_properties: u32, properties_size: u32) -> Vec<FbxProperty> {
    if num_properties == 0 {
        return Vec::new();
    }

    let mut properties = Vec::with_capacity(num_properties as _);

    for _ in 0..num_properties {
        let mut type_code = [0; 1];

        reader.read(&mut type_code).unwrap();

        let type_code = u8::from_le_bytes(type_code);

        // TODO: needs a lot more work
        let value = match TypeCode::from_byte(type_code) {
            TypeCode::Short => {
                let mut value = [0; 2];
                reader.read_exact(&mut value).unwrap();
                let value = i16::from_le_bytes(value);
                // println!("Short: {}", value);
            },
            TypeCode::Bool => {
                let mut value = [0; 1];
                reader.read_exact(&mut value).unwrap();
                let value = (u8::from_le_bytes(value) & 1) != 0;
                // println!("Bool: {}", value);
            },
            TypeCode::Int => {
                let mut value = [0; 4];
                reader.read_exact(&mut value).unwrap();
                let value = i32::from_le_bytes(value);
                // println!("Int: {}", value);
            },
            TypeCode::Float => {
                let mut value = [0; 4];
                reader.read_exact(&mut value).unwrap();
                let value = f32::from_le_bytes(value);
                // println!("Float: {}", value);
            },
            TypeCode::Double => {
                let mut value = [0; 8];
                reader.read_exact(&mut value).unwrap();
                let value = f64::from_le_bytes(value);
                // println!("Double: {}", value);
            },
            TypeCode::Long => {
                let mut value = [0; 8];
                reader.read_exact(&mut value).unwrap();
                let value = i64::from_le_bytes(value);
                // println!("Long: {}", value);
            },

            // TODO: these are more complicated and could be compressed
            TypeCode::FloatArray => {
                let mut len = [0; 4];
                let mut encoding = [0; 4];
                let mut compressed_len = [0; 4];
                reader.read_exact(&mut len).unwrap();
                reader.read_exact(&mut encoding).unwrap();
                reader.read_exact(&mut compressed_len).unwrap();
                let len = i32::from_le_bytes(len);
                let encoding = i32::from_le_bytes(encoding);
                let compressed_len = i32::from_le_bytes(compressed_len);

                if encoding == 0 {
                    let mut value = vec![0; len as usize * std::mem::size_of::<f32>()];
                    reader.read_exact(&mut value).unwrap();
                    // println!("FloatArray: {:?}", value);
                } else {
                    let mut value = vec![0; compressed_len as _];
                    reader.read_exact(&mut value).unwrap();
                    // println!("Compressed FloatArray: {:?}", value);
                }
            },
            TypeCode::DoubleArray => {
                let mut len = [0; 4];
                let mut encoding = [0; 4];
                let mut compressed_len = [0; 4];
                reader.read_exact(&mut len).unwrap();
                reader.read_exact(&mut encoding).unwrap();
                reader.read_exact(&mut compressed_len).unwrap();
                let len = i32::from_le_bytes(len);
                let encoding = i32::from_le_bytes(encoding);
                let compressed_len = i32::from_le_bytes(compressed_len);

                if encoding == 0 {
                    let mut value = vec![0; len as usize * std::mem::size_of::<f64>()];
                    reader.read_exact(&mut value).unwrap();
                    // println!("DoubleArray: {:?}", value);
                } else {
                    let mut value = vec![0; compressed_len as _];
                    reader.read_exact(&mut value).unwrap();
                    // println!("Compressed DoubleArray: {:?}", value);
                }
            },
            TypeCode::LongArray => {
                let mut len = [0; 4];
                let mut encoding = [0; 4];
                let mut compressed_len = [0; 4];
                reader.read_exact(&mut len).unwrap();
                reader.read_exact(&mut encoding).unwrap();
                reader.read_exact(&mut compressed_len).unwrap();
                let len = i32::from_le_bytes(len);
                let encoding = i32::from_le_bytes(encoding);
                let compressed_len = i32::from_le_bytes(compressed_len);

                if encoding == 0 {
                    let mut value = vec![0; len as usize * std::mem::size_of::<i64>()];
                    reader.read_exact(&mut value).unwrap();
                    // println!("LongArray: {:?}", value);
                } else {
                    let mut value = vec![0; compressed_len as _];
                    reader.read_exact(&mut value).unwrap();
                    // println!("Compressed LongArray: {:?}", value);
                }
            },
            TypeCode::IntArray => {
                let mut len = [0; 4];
                let mut encoding = [0; 4];
                let mut compressed_len = [0; 4];
                reader.read_exact(&mut len).unwrap();
                reader.read_exact(&mut encoding).unwrap();
                reader.read_exact(&mut compressed_len).unwrap();
                let len = i32::from_le_bytes(len);
                let encoding = i32::from_le_bytes(encoding);
                let compressed_len = i32::from_le_bytes(compressed_len);

                if encoding == 0 {
                    let mut value = vec![0; len as usize * std::mem::size_of::<i32>()];
                    reader.read_exact(&mut value).unwrap();
                    // println!("IntArray: {:?}", value);
                } else {
                    let mut value = vec![0; compressed_len as _];
                    reader.read_exact(&mut value).unwrap();
                    // println!("Compressed IntArray: {:?}", value);
                }
            },
            TypeCode::BoolArray => {
                let mut len = [0; 4];
                let mut encoding = [0; 4];
                let mut compressed_len = [0; 4];
                reader.read_exact(&mut len).unwrap();
                reader.read_exact(&mut encoding).unwrap();
                reader.read_exact(&mut compressed_len).unwrap();
                let len = i32::from_le_bytes(len);
                let encoding = i32::from_le_bytes(encoding);
                let compressed_len = i32::from_le_bytes(compressed_len);

                if encoding == 0 {
                    let mut value = vec![0; len as usize * std::mem::size_of::<u8>()];
                    reader.read_exact(&mut value).unwrap();
                    // println!("BoolArray: {:?}", value);
                } else {
                    let mut value = vec![0; compressed_len as _];
                    reader.read_exact(&mut value).unwrap();
                    // println!("Compressed BoolArray: {:?}", value);
                }
            },

            TypeCode::String => {
                let mut len = [0; 4];
                reader.read_exact(&mut len).unwrap();
                let len = i32::from_le_bytes(len);

                let mut value = vec![0; len as _];
                reader.read_exact(&mut value).unwrap();
                let value = String::from_utf8(value).unwrap();

                // println!("String: {}", value);
            },
            TypeCode::Raw => {
                let mut len = [0; 4];
                reader.read_exact(&mut len).unwrap();
                let len = i32::from_le_bytes(len);

                let mut value = vec![0; len as _];
                reader.read_exact(&mut value).unwrap();

                // println!("Raw: {:?}", value);
            },
        };

        // TODO:
        properties.push(FbxProperty {
            data: "".to_string()
        });
    }

    properties
}

fn parse_ascii_fbx() -> Result<Object, Box<dyn std::error::Error>> {
    todo!();
}
fn parse_binary_fbx(mut file: std::fs::File) -> Result<Object, Box<dyn std::error::Error>> {
    // TODO: we might need to write code that handles the different versions of the FBX format
    // which sucks big time. for now let's just parse the newer versions.
    // changes start from version 7.5

    file.seek(std::io::SeekFrom::Current(6))?; // skip the rest of the header

    let now = std::time::Instant::now();
    let fbx_nodes = FbxRecordIterator::new(BufReader::new(file));

    // skip the first record node because it's the header extension node
    // not sure if it's always there
    for node in fbx_nodes {
        // println!("node: {:?}", node);
    }
    let elapsed = now.elapsed();
    info!("Loaded in {} ms", elapsed.as_millis());

    todo!();
    // Ok(Object {
    //     name: "".to_string(),
    //     aabb: aabb::AABB::new(glm::vec3(0.0, 0.0, 0.0), glm::vec3(0.0, 0.0, 0.0)),
    //     meshes: vec![],
    // })
}

fn is_binary(magic: &[u8]) -> bool {
    magic == BINARY_FBX_MAGIC
}

pub fn load_fbx(mut file: std::fs::File) -> Result<Object, Box<dyn std::error::Error>> {
    let mut magic: [u8; 21] = [0; 21];
    let _ = file.read_exact(&mut magic);

    let obj = if is_binary(&magic) {
        parse_binary_fbx(file)?
    } else {
        file.seek(std::io::SeekFrom::Start(0))?;
        parse_ascii_fbx()?
    };

    Ok(obj)
}
