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
    projection: glm::Mat4x4,
    transform: glm::Mat4x4,
    move_speed: f32
}

impl Camera {
    fn assign_camera_uniform(&self, program: &Program) {
        let camera_transform = self.projection * self.transform;
        if let Err(e) = program.set_uniform_matrix("camera", camera_transform.as_ptr(), gl::UniformMatrix4fv) {
            eprintln!("Error occured in camera::forward, e: {}", e);
        }
    }

    pub fn move_in_dir(&mut self, direction: VecDir, delta_time: f32, program: &Program) {
        let (indices, dir): ([usize; 3], f32) = match direction {
            VecDir::Forward     => ([8, 9, 10],  1.0),
            VecDir::Backward    => ([8, 9, 10], -1.0),
            VecDir::Left        => ([0, 1,  3],  1.0),
            VecDir::Right       => ([0, 1,  3], -1.0),
            VecDir::Up          => ([4, 5,  6], -1.0),
            VecDir::Down        => ([4, 5,  6],  1.0),
        };
        
        let modifier = self.move_speed * delta_time * dir;

        self.transform = glm::translate(
            &self.transform, 
            &glm::vec3(
                self.transform[indices[0]] * modifier,
                self.transform[indices[1]] * modifier,
                self.transform[indices[2]] * modifier
            )
        );

        self.assign_camera_uniform(program);
    }
}   

pub struct CameraBuilder {
    projection: Option<glm::Mat4x4>,
    transform: Option<glm::Mat4x4>,
    move_speed: Option<f32>
}

impl CameraBuilder {
    pub fn init() -> Self {
        Self {
            projection: None,
            transform: None,
            move_speed: None
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

    pub fn transform(mut self, start_pos: &glm::Vec3) -> Self {
        self.transform = Some(glm::translate(&glm::identity::<f32, glm::U4>(), &start_pos));
        
        self
    }

    pub fn move_speed(mut self, move_speed: f32) -> Self {
        self.move_speed = Some(move_speed);

        self
    }

    // TODO: Return Result<Camera, CustomError> 
    pub fn build_and_attach_to_program(self, program: &mut Program) -> Camera {
        let projection = self.projection.expect("CameraBuiler has no projection");
        let transform = self.transform.expect("CameraBuiler has no transform");
        let move_speed: f32 = self.move_speed.unwrap_or_else(|| {
            println!("Default move_speed for CameraBuilder not supplied, using default");
            1.0
        });

        let camera = Camera {
            projection,
            transform,
            move_speed
        };

        if let Err(e) = program.locate_uniform("camera") {
            eprint!("Failed to find camera, probably loading wrong shader. err: {}", e);
        };

        let camera_transform = camera.projection * camera.transform;
        if let Err(e) = program.set_uniform_matrix("camera", camera_transform.as_ptr(), gl::UniformMatrix4fv) {
            eprintln!("{}", e);
        };

        return camera;
    }
}