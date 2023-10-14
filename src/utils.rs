use glad_gl::gl;
use glm;
use anyhow::{Result, Context};

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

