use std::fs::File;
use std::env;
use std::path::PathBuf;

use glfw::{Action, Context, Key, Modifiers};
use glad_gl::gl;
use anyhow;
use simplelog::*;

use threedobs::{shader, ui::ui, utils, ipc};

fn main() -> anyhow::Result<(), Box<dyn std::error::Error>> {
    let logger = threedobs::logger::WritableLog::default();

    let log_conf = ConfigBuilder::default()
        .set_target_level(LevelFilter::Error)
        .set_thread_level(LevelFilter::Off)
        .set_level_color(Level::Error, Some(Color::Red))
        .set_level_color(Level::Debug, Some(Color::Rgb(128, 128, 255)))
        .set_level_color(Level::Warn, Some(Color::Rgb(255, 163, 0)))
        .set_level_color(Level::Info, Some(Color::Rgb(128, 128, 128)))
        .build();
    let in_program_log_conf = ConfigBuilder::default()
        .set_target_level(LevelFilter::Off)
        .set_time_level(LevelFilter::Off)
        .set_thread_level(LevelFilter::Off)
        .set_level_color(Level::Error, Some(Color::Red))
        .set_level_color(Level::Debug, Some(Color::Rgb(128, 128, 255)))
        .set_level_color(Level::Warn, Some(Color::Rgb(255, 255, 0)))
        .set_level_color(Level::Info, Some(Color::Rgb(128, 128, 128)))
        .build();

    let log_level = if cfg!(debug_assertions) {
        LevelFilter::Trace
    } else {
        LevelFilter::Info
    };

    CombinedLogger::init(
        vec![
        TermLogger::new(log_level, log_conf.clone(), TerminalMode::Mixed, ColorChoice::Auto),
        WriteLogger::new(log_level, log_conf, File::create("3dobs.log")?),
        WriteLogger::new(log_level, in_program_log_conf, logger.clone())
        ]
    ).unwrap();

    let settings: ui::Settings = confy::load("3dobs", "settings")?;

    let args: Vec<String> = env::args().collect();
    let args_paths: Vec<PathBuf> = args
        .iter()
        .skip(1)
        .map(|arg| std::fs::canonicalize(PathBuf::from(arg)).unwrap())
        .collect();

    let lock_file_name = "3dobs.lock";
    let lock_file_path = std::env::temp_dir().join(lock_file_name);
    let lock_file = File::create(&lock_file_path)?;
    let ipc_rx = ipc::init(&lock_file, args_paths.clone(), settings.one_instance);

    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS)?;

    glfw::WindowHint::ContextVersion(3, 3);
    glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core);
    glfw::WindowHint::OpenGlForwardCompat(true);

    let (mut window, events) = glfw.create_window(1200, 800, "3dobs", glfw::WindowMode::Windowed).expect("Failed to create GLFW window");

    window.set_all_polling(true);
    window.set_cursor_mode(glfw::CursorMode::Disabled);
    window.make_current();

    let mut state = ui::State{
        settings,
        logger,
        ..Default::default()
    };

    glfw.set_swap_interval(glfw::SwapInterval::Sync(1));

    let (mut imgui, glfw_platform, renderer) = ui::init_imgui(&mut window);

    let mesh_shader = shader::Shader::new(
        &mut shader::ShaderSource{
            name: "vertex.glsl".to_string(),
            source: include_str!("../shaders/vertex.glsl").to_string(),
        },
        &mut shader::ShaderSource{
            name: "frag.glsl".to_string(),
            source: include_str!("../shaders/frag.glsl").to_string(),
        },
        )?;
    let grid_shader = shader::Shader::new(
        &mut shader::ShaderSource{
            name: "grid_v.glsl".to_string(),
            source: include_str!("../shaders/grid_v.glsl").to_string(),
        },
        &mut shader::ShaderSource{
            name: "grid_f.glsl".to_string(),
            source: include_str!("../shaders/grid_f.glsl").to_string(),
        },
        )?;

    let points_lights: [glm::Vec3; 4] = [
        glm::vec3(0.7, 0.2, 2.0),
        glm::vec3(2.3, -3.3, -4.0),
        glm::vec3(-4.0, 2.0, -12.0),
        glm::vec3(0.0, 0.0, -3.0),
    ];

    let mut delta_time: f32 = 0.0;
    let mut last_frame: f32 = 0.0;
    let mut last_cursor = None;

    let (w, h) = window.get_size();
    let mut last_x: f32 = w as f32 / 2.0;
    let mut last_y: f32 = h as f32 / 2.0;
    let mut first_mouse: bool = true;

    unsafe {
        grid_shader.use_shader();
        grid_shader.set_float("near", 0.01);
        grid_shader.set_float("far", 200.0);

        mesh_shader.use_shader();

        // set light uniforms
        for i in 0..points_lights.len() {
            mesh_shader.set_3fv(&format!("pointLights[{}].position", i), points_lights[i]);

            mesh_shader.set_float(&format!("pointLights[{}].constant", i), 1.0);
            mesh_shader.set_float(&format!("pointLights[{}].linear", i), 0.09);
            mesh_shader.set_float(&format!("pointLights[{}].quadratic", i), 0.032);

            mesh_shader.set_3fv(&format!("pointLights[{}].ambient", i), glm::vec3(0.1, 0.1, 0.1));
            mesh_shader.set_3fv(&format!("pointLights[{}].diffuse", i), glm::vec3(0.7, 0.7, 0.7));
            mesh_shader.set_3fv(&format!("pointLights[{}].specular", i), glm::vec3(1.0, 1.0, 1.0));
        }
        mesh_shader.set_float("spotLight.cutOff", glm::cos(glm::radians(12.5)));
        mesh_shader.set_float("spotLight.outerCutOff", glm::cos(glm::radians(15.0)));
        mesh_shader.set_3fv("spotLight.ambient", glm::vec3(0.2, 0.2, 0.2));
        mesh_shader.set_3fv("spotLight.diffuse", glm::vec3(0.5, 0.5, 0.5));
        mesh_shader.set_3fv("spotLight.specular", glm::vec3(1.0, 1.0, 1.0));
        mesh_shader.set_float("spotLight.constant", 1.0);
        mesh_shader.set_float("spotLight.linear", 0.09);
        mesh_shader.set_float("spotLight.quadratic", 0.032);

        mesh_shader.set_3fv("dirLight.direction", glm::vec3(-0.2, -1.0, -0.3));
        mesh_shader.set_3fv("dirLight.ambient", glm::vec3(0.1, 0.1, 0.1));
        mesh_shader.set_3fv("dirLight.diffuse", glm::vec3(0.5, 0.5, 0.5));
        mesh_shader.set_3fv("dirLight.specular", glm::vec3(1.0, 1.0, 1.0));

        let scene_fb = create_scene_framebuffer();

        if args.len() > 1 {
            utils::import_models_from_paths(&args_paths, &mut state);
        }

        let mut time_since_last_frame_acc = 0.0;

        // main loop
        while !window.should_close() {
            let current_frame = glfw.get_time() as f32;
            delta_time = current_frame - last_frame;
            last_frame = current_frame;

            imgui.io_mut().update_delta_time(std::time::Duration::from_secs_f32(delta_time));

            state.camera.update_speed(delta_time);

            time_since_last_frame_acc += delta_time;

            if time_since_last_frame_acc >= 0.1 {
                state.fps = 1.0 / delta_time;
                time_since_last_frame_acc = 0.0;
            }

            // camera matrices
            let view_mat = glm::ext::look_at(state.camera.position, state.camera.position + state.camera.front, state.camera.up);
            let projection_mat = glm::ext::perspective(glm::radians(state.camera.fov), state.viewport_size[0] / state.viewport_size[1], 0.01, 200.0);

            if let Some(rx) = &ipc_rx {
                match rx.try_recv() {
                    Ok(paths) => {
                        window.focus();
                        utils::import_models_from_paths(&paths, &mut state);
                    },
                    Err(e) => {
                        match e {
                            std::sync::mpsc::TryRecvError::Empty => {},
                            std::sync::mpsc::TryRecvError::Disconnected => {
                                panic!("Error: IPC thread channel disconnected");
                            }
                        }
                    }
                }
            }
            

            for (_, event) in glfw::flush_messages(&events) {
                // order of handling events is important here
                // we need to handle window events first to have an updated
                // is_cursor_captured
                handle_window_event(&mut window, &event, &mut state);
                if !state.is_cursor_captured {
                    glfw_platform.handle_event(imgui.io_mut(), &window, &event);
                }

                match event {
                    glfw::WindowEvent::CursorPos(xpos, ypos) => {
                        if first_mouse {
                            last_x = xpos as f32;
                            last_y = ypos as f32;
                            first_mouse = false;
                        }

                        let xoffset = xpos as f32 - last_x;
                        let yoffset = last_y - ypos as f32;
                        last_x = xpos as f32;
                        last_y = ypos as f32;

                        if state.can_capture_cursor && window.get_mouse_button(glfw::MouseButtonLeft) == Action::Press {
                            if window.get_key(glfw::Key::LeftShift) == Action::Press {
                                state.camera.move_camera(-xoffset, -yoffset);
                            } else {
                                if let Some(active_model) = state.active_model {
                                    let x_rotation = xoffset * state.camera.sensitivity * state.rotation_speed;
                                    let y_rotation = yoffset * state.camera.sensitivity * state.rotation_speed;
                                    let model = state.objects.iter_mut().find(|m| m.id == active_model).unwrap();
                                    // let x_rotation = glm::quat_angle_axis(xoffset * state.camera.sensitivity, &state.camera.up);
                                    model.rotate(x_rotation, y_rotation);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }

            //
            // draw scene to framebuffer
            //
            let (scene_texture, rbo) = create_scene_texture_and_renderbuffer(&window, scene_fb);

            gl::BindFramebuffer(gl::FRAMEBUFFER, scene_fb);
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::ClearColor(0.2, 0.2, 0.2, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            mesh_shader.use_shader();

            mesh_shader.set_mat4fv("view", &view_mat);
            mesh_shader.set_mat4fv("projection", &projection_mat);

            mesh_shader.set_3fv("spotLight.position", state.camera.position);
            mesh_shader.set_3fv("spotLight.direction", state.camera.front);
            mesh_shader.set_3fv("viewPos", state.camera.position);

            for obj in &state.objects {
                if state.wireframe {
                    gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
                } else {
                    gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
                }
                if Some(obj.id) == state.active_model {obj.draw(&mesh_shader, state.draw_aabb, state.show_textures, state.show_normal, state.show_emission);}
            }
            gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);

            // draw grid
            if state.draw_grid {draw_grid(&grid_shader, &view_mat, &projection_mat);}

            //
            // draw ui
            //
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            gl::ClearColor(0.1, 0.1, 0.1, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::Disable(gl::DEPTH_TEST);
            gl::Disable(gl::BLEND);
            ui::draw_ui(&mut imgui, &renderer, &glfw_platform, &mut window, &mut state, &mut last_cursor, scene_texture);

            glfw.poll_events();
            window.swap_buffers();

            gl::DeleteTextures(1, &scene_texture);
            gl::DeleteRenderbuffers(1, &rbo);
        }

        gl::DeleteFramebuffers(1, &scene_fb);
    }

    Ok(())
}

fn draw_grid(shader: &threedobs::shader::Shader, view_mat: &glm::Mat4, projection_mat: &glm::Mat4) {
    shader.use_shader();
    shader.set_mat4fv("view", &view_mat);
    shader.set_mat4fv("projection", &projection_mat);
    unsafe {
        gl::DrawArrays(gl::TRIANGLES, 0, 6);
    }
}

fn handle_window_event(window: &mut glfw::Window, event: &glfw::WindowEvent, state: &mut ui::State) {
    match event {
        glfw::WindowEvent::Key(Key::O, _, Action::Press, Modifiers::Control) => {
            ui::import_model(state);
        }
        glfw::WindowEvent::Key(Key::Q, _, Action::Press, Modifiers::Control) => {
            window.set_should_close(true);
        }
        glfw::WindowEvent::Key(Key::LeftControl, _, Action::Press, _) => {
            state.camera.speed *= 5.0;
        }
        glfw::WindowEvent::Key(Key::LeftControl, _, Action::Release, _) => {
            state.camera.speed /= 5.0;
        }
        glfw::WindowEvent::MouseButton(glfw::MouseButtonLeft, Action::Press, _) => {
            if !state.can_capture_cursor { return; }
            state.is_cursor_captured = true;
            window.set_cursor_mode(glfw::CursorMode::Disabled);
        }
        glfw::WindowEvent::MouseButton(glfw::MouseButtonLeft, Action::Release, _) => {
            if !state.can_capture_cursor { return; }
            state.is_cursor_captured = false;
            window.set_cursor_mode(glfw::CursorMode::Normal);
        }
        glfw::WindowEvent::Scroll(_, yoff) => {
            state.camera.handle_mouse_scroll(*yoff as f32, state.can_capture_cursor, state.fov_zoom);
        }
        glfw::WindowEvent::FileDrop(paths) => {
            utils::import_models_from_paths(paths, state);
        }
        glfw::WindowEvent::FramebufferSize(w, h) => {
            unsafe {
                gl::Viewport(0, 0, *w, *h);
            }
        }
        _ => {}
    }
}

fn create_scene_framebuffer() -> u32 {
    let mut fb: u32 = 0;

    unsafe {
        gl::GenFramebuffers(1, &mut fb);
        gl::BindFramebuffer(gl::FRAMEBUFFER, fb);
    }

    return fb;
}

fn create_scene_texture_and_renderbuffer(window: &glfw::Window, fbo: u32) -> (u32, u32) {
    let mut fb_texture: u32 = 0;
    let mut rbo: u32 = 0;

    let (w, h) = window.get_size();

    unsafe {
        gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);
        // texture
        gl::GenTextures(1, &mut fb_texture);
        gl::BindTexture(gl::TEXTURE_2D, fb_texture);

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as i32, w, h, 0, gl::RGB, gl::UNSIGNED_BYTE, std::ptr::null());

        gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, fb_texture, 0);

        // renderbuffer for depth
        gl::GenRenderbuffers(1, &mut rbo);
        gl::BindRenderbuffer(gl::RENDERBUFFER, rbo);
        gl::RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH24_STENCIL8, w, h);
        gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::DEPTH_STENCIL_ATTACHMENT, gl::RENDERBUFFER, rbo);

        if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
            panic!("ERROR::FRAMEBUFFER:: Framebuffer is not complete!");
        }
    }

    return (fb_texture, rbo);
}
