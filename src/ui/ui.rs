use std::time::{SystemTime, UNIX_EPOCH};

use glad_gl::gl;
use log::{info, debug};
use serde::{Serialize, Deserialize};

use crate::{camera::Camera, model, imgui_glfw_support, imgui_opengl_renderer, mesh, ui, logger, utils};

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Settings {
    pub one_instance: bool,
}

pub struct State {
    pub active_model: Option<u32>,
    pub show_console: bool,
    pub show_help_menu_about: bool,
    pub show_settings: bool,
    pub show_keybinds: bool,
    pub is_cursor_captured: bool,
    pub can_capture_cursor: bool,
    pub draw_grid: bool,
    pub draw_aabb: bool,
    pub fov_zoom: bool,
    pub rotation_speed: f32,
    pub wireframe: bool,
    pub first_frame_drawn: bool,
    pub camera: Camera,
    pub objects: Vec<model::Model>,
    pub viewport_size: [f32; 2],
    pub logger: logger::WritableLog,
    pub settings: Settings,
    pub fps: f32,
    pub show_textures: bool,
    pub show_normal: bool,
    pub show_emission: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            active_model: None,
            show_console: false,
            show_help_menu_about: false,
            show_settings: false,
            show_keybinds: false,
            first_frame_drawn: false,
            is_cursor_captured: false,
            can_capture_cursor: false,
            draw_grid: false,
            draw_aabb: false,
            fov_zoom: true,
            rotation_speed: 1.0,
            wireframe: false,
            camera: Camera::new(),
            objects: vec![],
            viewport_size: [0.0, 0.0],
            logger: logger::WritableLog::default(),
            settings: Settings::default(),
            fps: 0.0,
            show_textures: true,
            show_normal: true,
            show_emission: true,
        }
    }
}

impl State {
    pub fn get_next_id(&self) -> u32 {
        let mut id = 0;
        if let Some(last_model) = self.objects.last() {
            id = last_model.id + 1;
        }

        return id;
    }
}

pub fn init_imgui(window: &mut glfw::Window) -> (imgui::Context, imgui_glfw_support::GlfwPlatform, imgui_opengl_renderer::Renderer) {
    let mut imgui = imgui::Context::create();
    imgui.set_ini_filename(None);
    imgui.io_mut().config_flags.insert(imgui::ConfigFlags::DOCKING_ENABLE);
    imgui.io_mut().config_flags.set(imgui::ConfigFlags::NAV_ENABLE_KEYBOARD, true);

    let mut glfw_platform = imgui_glfw_support::GlfwPlatform::init(&mut imgui);
    glfw_platform.attach_window(
        imgui.io_mut(),
        &window,
        imgui_glfw_support::HiDpiMode::Default
    );

    imgui
        .fonts()
        .add_font(&[imgui::FontSource::TtfData {
            data: include_bytes!("../../assets/fonts/Exo-Regular.ttf"),
            size_pixels: 20.0,
            config: Some(imgui::FontConfig {
                oversample_h: 3,
                pixel_snap_h: true,
                ..imgui::FontConfig::default()
            })
        }]);

    imgui.io_mut().font_global_scale = (1.0 / glfw_platform.hidpi_factor()) as f32;

    gl::load(|e| window.get_proc_address(e) as *const std::os::raw::c_void);

    let renderer = imgui_opengl_renderer::Renderer::new(&mut imgui);
    glfw_platform.set_clipboard_backend(&mut imgui, &window);

    (imgui, glfw_platform, renderer)
}

pub fn import_model(state: &mut State) {
    let models = match rfd::FileDialog::new()
        .set_title("Import Model(s)")
        .set_directory("./")
        .add_filter("All supported files", &["obj", "OBJ", "stl", "STL"])
        .add_filter("Wavefront OBJ (.obj)", &["obj", "OBJ"])
        .add_filter("STL (.stl)", &["stl", "STL"])
        .pick_files() {
            Some(m) => m,
            None => return,
        };
    utils::import_models_from_paths(&models, state);
}

pub fn draw_main_menu_bar(ui: &imgui::Ui, state: &mut State, window: &mut glfw::Window) {
    ui.main_menu_bar(|| {
        ui.menu("File", || {
            if ui.menu_item_config("Import Model(s)").shortcut("Ctrl+O").build() {
                import_model(state);
            }
            if ui.menu_item_config("Settings").build() {
                state.show_settings = !state.show_settings;
            }
            if ui.menu_item_config("Quit").shortcut("Ctrl+Q").build() {
                window.set_should_close(true);
            }
        });
        ui.menu("View", || {
            if ui.menu_item_config("Show Grid").selected(state.draw_grid).build() {
                state.draw_grid = !state.draw_grid;
            }
            if ui.menu_item_config("Draw Bounding Box").selected(state.draw_aabb).build() {
                state.draw_aabb = !state.draw_aabb;
            }
        });
        ui.menu("Help", || {
            if ui.menu_item_config("Keybinds").selected(state.show_keybinds).build() {
                state.show_keybinds = !state.show_keybinds;
            }
            if ui.menu_item_config("About").selected(state.show_help_menu_about).build() {
                state.show_help_menu_about = !state.show_help_menu_about;
            }
        });
        let mem = state.objects.iter().fold(0 as usize, |acc, m| acc + m.mem_usage) as f32;
        let mem_fps = format!("Mem: {:.1}MB | FPS: {:.1}", mem / (1024.0 * 1024.0), state.fps);
        let avail_size = [*ui.content_region_avail().get(0).unwrap() - ui.calc_text_size(&mem_fps)[0], 0.0];
        ui.dummy(avail_size);
        ui.text(&mem_fps);
    });
}

fn draw_about_window(ui: &imgui::Ui, state: &mut State) {
    if !state.show_help_menu_about {
        return;
    }
    let display_size = ui.io().display_size;

    ui.window("About")
        .resizable(false)
        .movable(false)
        .opened(&mut state.show_help_menu_about)
        .position([display_size[0] / 2.0, display_size[1] / 2.0], imgui::Condition::Always)
        .position_pivot([0.5, 0.5])
        .build(|| {
            ui.text("3dobs - 3D Object Browser");
            ui.text(format!("Version: {}-{}", env!("CARGO_PKG_VERSION"), env!("GIT_HASH")));
            ui.text(format!("{}", env!("CARGO_PKG_DESCRIPTION")));
            ui.spacing();
            ui.spacing();
            ui.text(format!("Made by: {}", env!("CARGO_PKG_AUTHORS")));
        });
}

pub fn draw_settings_window(ui: &imgui::Ui, state: &mut State) {
    if !state.show_settings {
        return;
    }
    let display_size = ui.io().display_size;

    ui.window("Settings")
        .opened(&mut state.show_settings)
        .movable(false)
        .position([display_size[0] / 2.0, display_size[1] / 2.0], imgui::Condition::Always)
        .position_pivot([0.5, 0.5])
        .build(|| {
            if ui.checkbox("Only allow one program instance (Reboot required when enabling)", &mut state.settings.one_instance) {
                confy::store("3dobs", "settings", state.settings.clone()).unwrap();
            }
        });
}

fn draw_keybinds_window(ui: &imgui::Ui, state: &mut State) {
    if !state.show_keybinds {
        return;
    }
    let display_size = ui.io().display_size;

    ui.window("Keybinds")
        .opened(&mut state.show_keybinds)
        .resizable(false)
        .movable(false)
        .position([display_size[0] / 2.0, display_size[1] / 2.0], imgui::Condition::Always)
        .position_pivot([0.5, 0.5])
        .build(|| {
            if let Some(..) = ui.begin_table_with_sizing("Keybinds Table", 2, imgui::TableFlags::SIZING_STRETCH_SAME, [0.0, 0.0], 0.0) {
                ui.table_next_column();
                ui.text_colored([0.7, 0.7, 0.6, 1.0], "Key");
                ui.table_next_column();
                ui.text_colored([0.7, 0.7, 0.6, 1.0], "Action");

                ui.table_next_column();
                ui.text("Ctrl + O | Drag & Drop");
                ui.table_next_column();
                ui.text("Import Model(s)");

                ui.table_next_column();
                ui.text("Ctrl + Q");
                ui.table_next_column();
                ui.text("Quit");
                
                ui.table_next_column();
                ui.text("Left Mouse Button");
                ui.table_next_column();
                ui.text("Rotate object");

                ui.table_next_column();
                ui.text("Scroll");
                ui.table_next_column();
                ui.text("Zoom camera");

                ui.table_next_column();
                ui.text("Left Shift");
                ui.table_next_column();
                ui.text("Pan camera");

                ui.table_next_column();
                ui.text("Left Ctrl");
                ui.table_next_column();
                ui.text("Increase camera movement speed");
            }
        });
}

fn draw_transformations(ui: &imgui::Ui, mesh: &mut mesh::Mesh) {
    imgui::Drag::new("###XPos")
        .range(f32::NEG_INFINITY, f32::INFINITY)
        .speed(0.1)
        .display_format("X: %.3f")
        .build(ui, &mut mesh.position.x);
    imgui::Drag::new("###YPos")
        .range(f32::NEG_INFINITY, f32::INFINITY)
        .speed(0.1)
        .display_format("Y: %.3f")
        .build(ui, &mut mesh.position.y);
    imgui::Drag::new("###ZPos")
        .range(f32::NEG_INFINITY, f32::INFINITY)
        .speed(0.1)
        .display_format("Z: %.3f")
        .build(ui, &mut mesh.position.z);
}

fn draw_mesh_hierarchy(ui: &imgui::Ui, mesh: &mut mesh::Mesh, i: usize) {
    ui.tree_node_config(format!("{}###{}", mesh.name.as_str(), i)).build(|| {
        ui.text(format!("Vertices: {}", mesh.vertices.len()));
        ui.text(format!("Triangles: {}", mesh.indices.len() / 3));
        ui.tree_node_config(mesh.material.name.as_str()).build(|| {
            ui.text(format!("{}", mesh.material));
        });
        ui.tree_node_config("Transformations").build(|| {
            draw_transformations(ui, mesh);
        })
    });
}

fn draw_object_hierarchy(ui: &imgui::Ui, state: &mut State, idx: usize) -> bool {
    ui.table_next_column();
    if ui.checkbox(format!("###{}", state.objects[idx].id), &mut (Some(state.objects[idx].id) == state.active_model)) {
        state.objects[idx].reset_rotation();
        state.active_model = Some(state.objects[idx].id);
        state.camera.focus_on_selected_model(state.active_model, &state.objects);
    }

    ui.table_next_column();
    ui.tree_node_config(format!("{} ({:.1}MB)###{}", state.objects[idx].name.as_str(), state.objects[idx].mem_usage as f32 / (1024.0 * 1024.0), idx))
        .build(|| {
            for (j, mesh) in &mut state.objects[idx].meshes.iter_mut().enumerate() {
                draw_mesh_hierarchy(ui, mesh, j);
            }
        });

    ui.table_next_column();
    if ui.small_button(format!("X###{}-{}", state.objects[idx].name.as_str(), idx)) {
        info!("Removing object {}", state.objects[idx].name);
        return true;
    }

    return false;
}

fn draw_objects_window(ui: &imgui::Ui, state: &mut State) {
    ui.window("Objects")
        .size([500.0, 200.0], imgui::Condition::FirstUseEver)
        .build(|| {
            let mut i = 0;

            if let Some(..) = ui.begin_table_with_sizing("Objects Table", 3, imgui::TableFlags::SIZING_FIXED_FIT, [0.0, 0.0], 0.0) {
                ui.table_setup_column_with(imgui::TableColumnSetup {
                    name: "",
                    flags: imgui::TableColumnFlags::empty(),
                    init_width_or_weight: 30.0,
                    user_id: imgui::Id::default(),
                });
                ui.table_setup_column_with(imgui::TableColumnSetup {
                    name: "",
                    flags: imgui::TableColumnFlags::WIDTH_STRETCH,
                    init_width_or_weight: 0.0,
                    user_id: imgui::Id::default(),
                });
                ui.table_setup_column_with(imgui::TableColumnSetup {
                    name: "",
                    flags: imgui::TableColumnFlags::empty(),
                    init_width_or_weight: 20.0,
                    user_id: imgui::Id::default(),
                });

                while i < state.objects.len() {
                    if draw_object_hierarchy(ui, state, i) {
                        let selected_obj_id = state.objects[i].id;
                        state.objects.remove(i);
                        if state.active_model == Some(selected_obj_id) {
                            let model = state.objects.last_mut().map(|m| m.reset_rotation());
                            state.active_model = model.and_then(|o| Some(o.id));
                            state.camera.focus_on_selected_model(state.active_model, &state.objects);
                        }
                        continue;
                    }

                    i = i + 1;
                }
            }
        });
}

fn draw_console(ui: &imgui::Ui, state: &mut State) {
    ui.window("Console")
        .size([500.0, 200.0], imgui::Condition::FirstUseEver)
        .build(|| {
            ui.child_window("###ConsoleHistory")
                .size([0.0, -35.0])
                .build(|| {
                    for line in state.logger.arc.read().unwrap().history.iter() {
                        let style = ui.push_style_color(imgui::StyleColor::Text, line.level);

                        ui.text_wrapped(line.message.clone());
                        style.pop();
                    }
                    if ui.scroll_y() >= ui.scroll_max_y() {
                        ui.set_scroll_here_y_with_ratio(1.0);
                    }
                });

            ui.separator();
            if ui.button("Clear") {
                let mut logger = state.logger.arc.write().unwrap();
                logger.clear();
            }
        });
}

fn create_initial_docking(ui: &imgui::Ui, state: &mut State) {
    let flags =
        // No borders etc for top-level window
        imgui::WindowFlags::NO_DECORATION | imgui::WindowFlags::NO_MOVE
        // Show menu bar
        | imgui::WindowFlags::MENU_BAR
        // Don't raise window on focus (as it'll clobber floating windows)
        | imgui::WindowFlags::NO_BRING_TO_FRONT_ON_FOCUS | imgui::WindowFlags::NO_NAV_FOCUS
        // Don't want the dock area's parent to be dockable!
        | imgui::WindowFlags::NO_DOCKING
        ;

    let padding = ui.push_style_var(imgui::StyleVar::WindowPadding([0.0, 0.0]));
    let rounding = ui.push_style_var(imgui::StyleVar::WindowRounding(0.0));

    ui.window("Main Window")
        .flags(flags)
        .position([0.0, 0.0], imgui::Condition::Always)
        .size(ui.io().display_size, imgui::Condition::Always)
        .build(|| {
            // Create top-level docking area, needs to be made early (before docked windows)
            let ui_d = ui::docking::UiDocking {};
            let space = ui_d.dockspace("MainDockArea");

            // Set up splits, docking windows. This can be done conditionally,
            // or calling it every time is also mostly fine
            if !state.first_frame_drawn {
                space.split(
                    imgui::Direction::Right,
                    300.0 / ui.io().display_size[0],
                    |right| {
                        right.split(
                            imgui::Direction::Up,
                            0.6,
                            |up| {
                                up.dock_window("Objects");
                            },
                            |down| {
                                down.dock_window("Console");
                            }
                        )
                    },
                    |left| {
                        left.dock_window("Viewer");
                    }
                )
            }
        });

    padding.pop();
    rounding.pop();
}

fn draw_viewport(ui: &imgui::Ui, state: &mut State, texture: u32) {
    ui.window("Viewer")
        .size(ui.content_region_avail(), imgui::Condition::FirstUseEver)
        .no_decoration()
        .scrollable(!state.can_capture_cursor)
        .resizable(true)
        .build(|| {
            let mut tex_size = ui.content_region_avail();
            tex_size[1] -= 25.0;
            state.viewport_size = tex_size;

            if ui.button("Reset Camera") {
                state.camera.focus_on_selected_model(state.active_model, &state.objects);
            }
            ui.same_line();
            if ui.button("Capture Scene") {
                let now = std::time::Instant::now();
                let mut w = 0;
                let mut h = 0;

                unsafe {
                    gl::GetTextureLevelParameteriv(texture, 0, gl::TEXTURE_WIDTH, &mut w);
                    gl::GetTextureLevelParameteriv(texture, 0, gl::TEXTURE_HEIGHT, &mut h);
                }

                let mut pixels = vec![0u8; (w * h * 4) as usize];

                unsafe {
                    gl::GetTextureImage(texture, 0, gl::RGBA, gl::UNSIGNED_BYTE, (w * h * 4) as i32, pixels.as_mut_ptr() as *mut std::ffi::c_void);
                }

                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Current time to not be before the UNIX epoch");
                let file_name = format!("capture-{}.png", timestamp.as_secs());
                let save_path = std::path::Path::new(file_name.as_str());
                let capture = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(w as u32, h as u32, pixels).unwrap();
                let capture = image::DynamicImage::ImageRgba8(capture);
                let capture = capture.flipv();
                let capture = capture.resize_exact(tex_size[0] as u32, tex_size[1] as u32, image::imageops::FilterType::Gaussian);
                let _ = capture.save(save_path);
                let elapsed = now.elapsed();

                info!("Scene capture saved to: {} successfully", save_path
                    .canonicalize()
                    .expect("Capture path to be canonicalized")
                    .to_str()
                    .expect("Capture path to be valid unicode"));

                debug!("Scene capture took: {}ms", elapsed.as_millis());

                unsafe {
                    gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
                }
            }
            ui.same_line();
            ui.checkbox("Wireframe", &mut state.wireframe);
            ui.same_line();
            ui.checkbox("FOV zoom", &mut state.fov_zoom);
            ui.same_line();
            ui.checkbox("Show Textures", &mut state.show_textures);
            ui.same_line();
            ui.checkbox("Use Normal", &mut state.show_normal);
            ui.same_line();
            ui.checkbox("Use Emissive", &mut state.show_emission);
            ui.same_line();
            ui.set_next_item_width(150.0);
            imgui::Drag::new("Camera Speed")
                .range(1.0, 10000.0)
                .speed(1.0)
                .display_format("%.3f")
                .build(ui, &mut state.camera.speed);
            ui.same_line();
            ui.set_next_item_width(150.0);
            imgui::Drag::new("Rotation Speed")
                .range(0.1, 100.0)
                .speed(0.5)
                .display_format("%.3f")
                .build(ui, &mut state.rotation_speed);
            imgui::Image::new(imgui::TextureId::new(texture.try_into().unwrap()), tex_size)
                // flip the image vertically
                .uv0([0.0, 1.0])
                .uv1([1.0, 0.0])
                .build(ui);

            // only allow capturing the cursor if the mouse is over the viewport
            state.can_capture_cursor = ui.is_item_hovered();
        });
}

pub fn draw_ui(
    imgui: &mut imgui::Context,
    renderer: &imgui_opengl_renderer::Renderer,
    glfw_platform: &imgui_glfw_support::GlfwPlatform,
    window: &mut glfw::Window,
    state: &mut State,
    last_cursor: &mut Option<imgui::MouseCursor>,
    scene_fb_texture: u32,
) {
    glfw_platform.prepare_frame(imgui.io_mut(), window).expect("Failed to prepare imgui frame");

    let ui = imgui.new_frame();
    create_initial_docking(ui, state);

    draw_main_menu_bar(ui, state, window);

    draw_viewport(ui, state, scene_fb_texture);
    draw_objects_window(ui, state);
    draw_console(ui, state);
    draw_about_window(ui, state);
    draw_keybinds_window(ui, state);
    draw_settings_window(ui, state);

    ui.end_frame_early();

    if !state.can_capture_cursor {
        let cursor = ui.mouse_cursor();
        if *last_cursor != cursor {
            *last_cursor = cursor;
            glfw_platform.prepare_render(&ui, window);
        }
    }

    imgui.update_platform_windows();

    renderer.render(imgui);
    state.first_frame_drawn = true;
}
