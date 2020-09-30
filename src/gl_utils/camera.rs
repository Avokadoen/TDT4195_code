use crate::gl_utils::shaders::program::Program;
use glm;

// Move this
pub enum VecDir {
    Forward,
    Left,
    Backward,
    Right,
    Up,
    Down
}

pub struct Camera {
    pub projection: glm::Mat4x4,
    translation: glm::Mat4x4,
    orientation: glm::Quat,
    pitch: f32,
    yaw: f32,
    move_speed: f32,
    turn_sensitivity: f32,
    binded_programs: Vec<Program>
}

impl Camera {
    fn assign_camera_uniform(&self) {
        for program in &self.binded_programs {
            let camera_transform = self.projection * glm::quat_to_mat4(&self.orientation) * self.translation;
            if let Err(e) = program.set_uniform_matrix("camera", camera_transform.as_ptr(), gl::UniformMatrix4fv) {
                eprintln!("Error occured while assigning camera, e: {}", e);
            }
        }
    }

    pub fn move_in_dir(&mut self, direction: VecDir, delta_time: f32) {
        let local_direction = {
            let global_direction = match direction {
                VecDir::Forward     => glm::vec3(0.0, 0.0, 1.0),
                VecDir::Backward    => glm::vec3(0.0, 0.0, -1.0),
                VecDir::Left        => glm::vec3(1.0, 0.0, 0.0),
                VecDir::Right       => glm::vec3(-1.0, 0.0, 0.0),
                VecDir::Up          => glm::vec3(0.0, -1.0, 0.0),
                VecDir::Down        => glm::vec3(0.0, 1.0, 0.0),
            };

            glm::quat_rotate_vec3(&glm::quat_inverse(&self.orientation), &global_direction)
        };
        
        let stride = self.move_speed * delta_time;

        let offset = local_direction * stride;
        self.translation = glm::translate(
            &self.translation, 
            &offset
        );

        self.assign_camera_uniform();
    }

    pub fn turn(&mut self, turn_vector: (f64, f64), delta_time: f32) {
        self.pitch += turn_vector.1 as f32 * delta_time * self.turn_sensitivity;
        self.orientation = glm::quat_rotate(&glm::quat_identity(), self.pitch, &glm::vec3(1.0, 0.0, 0.0));
        
        self.yaw += turn_vector.0 as f32 * delta_time * self.turn_sensitivity; 
        self.orientation = glm::quat_rotate(&self.orientation, self.yaw, &glm::vec3(0.0, 1.0, 0.0));

        // Avoid float rounding errors
        let one_rotation = 2.0 * std::f32::consts::PI;
        self.yaw = self.yaw % one_rotation;
        self.pitch = self.pitch % one_rotation;

        self.assign_camera_uniform();
    }

    pub fn position(&self) -> glm::Vec3 {
        glm::vec3(self.translation[12], self.translation[13], self.translation[14])
    }
}   

pub struct CameraBuilder {
    projection: Option<glm::Mat4x4>,
    translation: Option<glm::Mat4x4>,
    pitch: Option<f32>,
    yaw: Option<f32>,
    move_speed: Option<f32>,
    turn_sensitivity: Option<f32>
}

impl CameraBuilder {
    pub fn init() -> Self {
        Self {
            projection: None,
            translation: None,
            pitch: None,
            yaw: None,
            move_speed: None,
            turn_sensitivity: None
        }
    }

    pub fn projection(mut self, aspect: f32, fovy: f32, near: f32, far: f32) -> Self {
        self.projection = Some(glm::perspective::<f32>(
            aspect, 
            fovy, 
            near, 
            far
        ));

        self
    }

    pub fn translation(mut self, start_pos: &glm::Vec3) -> Self {
        self.translation = Some(glm::translate(&glm::identity::<f32, glm::U4>(), &start_pos));
        
        self
    }

    pub fn pitch(mut self, pitch: f32) -> Self {
        self.pitch = Some(pitch);

        self
    }
    
    pub fn yaw(mut self, yaw: f32) -> Self {
        self.yaw = Some(yaw);

        self
    }

    pub fn move_speed(mut self, move_speed: f32) -> Self {
        self.move_speed = Some(move_speed);

        self
    }

    pub fn turn_sensitivity(mut self, turn_sensitivity: f32) -> Self {
        self.turn_sensitivity = Some(turn_sensitivity);

        self
    }

    // TODO: Return Result<Camera, CustomError> 
    #[must_use = "Camera can only be built using the build_and_attach_to_programs() function"]
    pub fn build_and_attach_to_programs(self, mut programs: Vec<Program>) -> Camera {
        let projection = self.projection.expect("CameraBuiler has no projection");

        let translation = self.translation.unwrap_or_else(|| {
            println!("Translation for CameraBuilder not supplied, using default");
            glm::identity()
        });

        let pitch = self.pitch.unwrap_or_else(|| {
            println!("Pitch for CameraBuilder not supplied, using default");
            0.0
        });

        let yaw = self.yaw.unwrap_or_else(|| {
            println!("Yaw for CameraBuilder not supplied, using default");
            0.0
        });

        let move_speed: f32 = self.move_speed.unwrap_or_else(|| {
            println!("Move_speed for CameraBuilder not supplied, using default");
            1.0
        });

        let turn_sensitivity: f32 = self.turn_sensitivity.unwrap_or_else(|| {
            println!("Turn_sensitivity for CameraBuilder not supplied, using default");
            1.0
        });

        for program in &mut programs {
            // find uniform for camera
            if let Err(e) = program.locate_uniform("camera") {
                eprint!("Failed to find camera, probably loading wrong shader. err: {}", e);
            };
        }

        let mut camera = Camera {
            projection,
            translation,
            orientation: glm::quat_identity(),
            pitch,
            yaw,
            move_speed,
            turn_sensitivity,
            binded_programs: programs
        };

        // TODO: HACK: Used to assign camera uniforms, resolve hack
        camera.turn((1.0, 0.0), 0.0);

        camera
    }
}