use std::path::PathBuf;

use glad_gl::gl;
use glm;
use anyhow::{Result, Context};

use crate::{log, model, ui, importer};

pub enum SupportedFileExtensions {
    OBJ,
    STL
}

impl SupportedFileExtensions {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "obj" => Some(Self::OBJ),
            "stl" => Some(Self::STL),
            _ => None
        }
    }
}

pub fn load_texture(path: &str) -> Result<u32> {
    let tex = image::io::Reader::open(path)
        .with_context(|| format!("Failed to open texture file: {}", path))?
        .decode()
        .with_context(|| format!("Failed to decode texture: {}", path))?;

    let mut texture_id: u32 = 0;
    let format = match tex.color().channel_count() {
        1 => gl::RED,
        2 => gl::RG,
        3 => gl::RGB,
        4 => gl::RGBA,
        _ => panic!("Unknown image format")
    };

    unsafe {
        gl::GenTextures(1, &mut texture_id);
        gl::BindTexture(gl::TEXTURE_2D, texture_id);

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);

        gl::TexImage2D(gl::TEXTURE_2D, 0, format as i32, tex.width() as i32, tex.height() as i32, 0, format, gl::UNSIGNED_BYTE, tex.as_bytes().as_ptr() as *const std::ffi::c_void);
        gl::GenerateMipmap(gl::TEXTURE_2D);
    }

    Ok(texture_id)
}

pub fn mat_ident() -> glm::Mat4 {
    glm::mat4(
        1., 0., 0., 0.,
        0., 1., 0., 0.,
        0., 0., 1., 0.,
        0., 0., 0., 1.
    )
}

pub fn import_models_from_paths(paths: &Vec<PathBuf>, state: &mut ui::ui::State) {
    for model_path in paths {
        let filename = model_path.file_name();
        if model_path.is_dir() {
            state.logger.log(&format!("Skipping directory \"{}\"", filename.unwrap().to_str().unwrap()), log::LogLevel::Info);
            continue;
        }
        match model_path.extension() {
            Some(ext) => {
                if SupportedFileExtensions::from_str(ext.to_str().unwrap()).is_none() {
                    state.logger.log(&format!("Skipping file \"{}\" because it is not an OBJ or STL file", filename.unwrap().to_str().unwrap()), log::LogLevel::Info);
                    continue;
                }
            },
            None => {
                state.logger.log(&format!("Skipping file \"{}\" because it has no extension", filename.unwrap().to_str().unwrap()), log::LogLevel::Info);
                continue;
            }
        }
        let obj_result = importer::load_from_file(model_path);
        match obj_result {
            Ok(obj) => {
                let mut m = model::Model::new(obj, state);

                state.active_model = Some(m.id);
                if let Some(model_name) = filename {
                    state.logger.log(&format!("Loaded model \"{}\"", model_name.to_str().unwrap()), log::LogLevel::Info);
                    m.name = model_name.to_str().unwrap().to_string();
                }
                state.objects.push(m);
                state.camera.update_position(state.active_model, &state.objects);
            },
            Err(e) => {
                let error = format!("Error loading model \"{}\": {}", model_path.to_str().unwrap(), e);
                println!("{}", error);

                state.logger.log(&error, log::LogLevel::Error);
            }
        }
    }
}
