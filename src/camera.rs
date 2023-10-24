use glm;

use crate::model;

pub struct Camera {
    pub position: glm::Vec3,
    pub front: glm::Vec3,
    pub up: glm::Vec3,
    pub pitch: f32,
    pub yaw: f32,
    pub speed: f32,
    _speed: f32,
    pub sensitivity: f32,
    pub fov: f32,
}

impl Camera {
    pub fn new() -> Camera {
        Camera {
            position: glm::vec3(0.0, 3.0, 3.0),
            front: glm::vec3(0.0, 0.0, -1.0),
            up: glm::vec3(0.0, 1.0, 0.0),
            pitch: 0.0,
            yaw: -90.0,
            _speed: 0.0,
            speed: 10.0,
            sensitivity: 0.05,
            fov: 45.0,
        }
    }

    pub fn handle_mouse_scroll(&mut self, yoffset: f32, can_capture_cursor: bool, fov_zoom: bool) {
        if !can_capture_cursor { return }
        if fov_zoom {
            self.fov -= yoffset;

            if self.fov <= 0.5 {
                self.fov = 0.5;
            }
            if self.fov >= 85.0 {
                self.fov = 85.0;
            }
        } else {
            self.position = self.position + (self.front * self._speed) + glm::vec3(0.0, 0.0, -yoffset);
        }
    }

    pub fn move_camera(&mut self, xoffset: f32, yoffset: f32) {
        let new_x = xoffset * self.sensitivity * self._speed;
        let new_y = yoffset * self.sensitivity * self._speed;

        self.position = self.position + glm::vec3(new_x, new_y, 0.0);
    }

    pub fn update_speed(&mut self, delta_time: f32) {
        self._speed = self.speed * delta_time;
    }

    pub fn focus_on_selected_model(&mut self, active_model: Option<u32>, objects: &Vec<model::Model>) {
        if let Some(id) = active_model {
            for obj in objects {
                if obj.id == id {
                    // we scale the center of the object since the model (and therefore the AABB) is scaled
                    let center_x = ((obj.aabb.max.x / 2.0) + (obj.aabb.min.x / 2.0)) * obj.scaling_factor;
                    let center_y = ((obj.aabb.max.y / 2.0) + (obj.aabb.min.y / 2.0)) * obj.scaling_factor;
                    let z = obj.aabb.max.z * obj.scaling_factor + 10.0;
                    self.position = glm::vec3(center_x, center_y, z);
                    self.front = glm::vec3(0.0, 0.0, -1.0);
                    break;
                }
            }
        }
    }
}

