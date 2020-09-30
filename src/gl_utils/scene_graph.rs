extern crate nalgebra_glm as glm;

// Author: Michael H. Gimle

// TODO: Discard this and write a custom structure INSTEAD of scene graphing

use std::mem::ManuallyDrop;
use std::pin::Pin;

use super::{camera::Camera, geometric_object::GeometricObject};

// Used to crete an unholy abomination upon which you should not cast your gaze.
// This ended up being a necessity due to wanting to keep the code written by students as "straight forward" as possible
// It is very very double plus ungood Rust, and intentionally leaks memory like a sieve. But it works, and you're more than welcome to pretend it doesn't exist!
// In case you're curious about how it works; It allocates memory on the heap (Box), promises to prevent it from being moved or deallocated until dropped (Pin) 
// and finally prevents the compiler from dropping it automatically at all (ManuallyDrop). If that sounds like a janky solution, it's because it is.
// Prettier, Rustier and better solutions were tried numerous times, but were all found wanting of having what I arbitrarily decided to be the required level of
// simplicity of use.
type Node = ManuallyDrop<Pin<Box<SceneNode>>>;

pub struct SceneNode {
    pub position: glm::Vec3,
    /// In radians
    pub rotation: glm::Vec3,
    pub scale: glm::Vec3,
    pub reference_point: glm::Vec3,

    pub current_transformation_matrix: glm::Mat4,

    // my hack to integrate with existing code
    pub geometric_object: Option<GeometricObject>,

    pub children: Vec<*mut SceneNode>,
}

impl SceneNode {
    pub fn new() -> Node {
        ManuallyDrop::new(Pin::new(Box::new(SceneNode {
            position: glm::zero(),
            rotation: glm::zero(),
            scale: glm::vec3(1.0, 1.0, 1.0),
            reference_point: glm::zero(),
            current_transformation_matrix: glm::identity(),
            geometric_object: None,
            children: vec![],
        })))
    }

    pub fn from_vao(geometric_object: GeometricObject) -> Node {
        ManuallyDrop::new(Pin::new(Box::new(SceneNode {
            position: glm::zero(),
            rotation: glm::zero(),
            scale: glm::vec3(1.0, 1.0, 1.0),
            reference_point: glm::zero(),
            current_transformation_matrix: glm::identity(),
            geometric_object: Some(geometric_object),
            children: vec![],
        })))
    }
    pub fn add_child(&mut self, child: &SceneNode) {
        self.children.push(child as *const SceneNode as *mut SceneNode)
    }

    // TODO: impl Display instead
    pub fn print(&self) {
        let m = self.current_transformation_matrix;
        let matrix_string = format!(
"
      {:.2}  {:.2}  {:.2}  {:.2}
      {:.2}  {:.2}  {:.2}  {:.2}
      {:.2}  {:.2}  {:.2}  {:.2}
      {:.2}  {:.2}  {:.2}  {:.2}
",
            m[0],m[4],m[8],m[12],
            m[1],m[5],m[9],m[13],
            m[2],m[6],m[10],m[14],
            m[3],m[7],m[11],m[15],
        );

        let (vao, indices) = match &self.geometric_object {
            Some(g) => (g.id, g.indices_count),
            None => (0, -1)
        }; 
        println!(
"SceneNode {{
    VAO:       {}
    Indices:   {}
    Children:  {}
    Position:  [{:.2}, {:.2}, {:.2}]
    Rotation:  [{:.2}, {:.2}, {:.2}]
    Reference: [{:.2}, {:.2}, {:.2}]
    Current Transformation Matrix: {}
}}",
            vao,
            indices,
            self.children.len(),
            self.position.x,
            self.position.y,
            self.position.z,
            self.rotation.x,
            self.rotation.y,
            self.rotation.z,
            self.reference_point.x,
            self.reference_point.y,
            self.reference_point.z,
            matrix_string,
        );
    } 
}

// author: Aksel Hjerpbakk
impl SceneNode {    
    pub fn set_reference_point(&mut self, point: glm::Vec3) {
        self.reference_point = point;
    }

    // Again, no consideration of speed, we construct a lot of discarded vectors in this function.
    // Which is even worse in a recursive context! Also using euler rotation here might bite me later ...
    pub fn update_node_transformations(&mut self, transformation_so_far: &glm::Mat4) {
        unsafe {
            // Construct the correct transformation matrix
            let transformation = {
                let mut self_mat = glm::scale(&glm::Mat4::identity(), &self.scale);

                self_mat = glm::translate(&self_mat, &self.reference_point);
                self_mat = glm::rotate(&self_mat, self.rotation.x, &glm::vec3(1.0, 0.0, 0.0));
                self_mat = glm::rotate(&self_mat, self.rotation.y, &glm::vec3(0.0, 1.0, 0.0));
                self_mat = glm::rotate(&self_mat, self.rotation.z, &glm::vec3(0.0, 0.0, 1.0));
                self_mat = glm::translate(&self_mat, &glm::vec3(-self.reference_point.x, -self.reference_point.y, -self.reference_point.z));
                
                glm::translate(&self_mat, &self.position)
            };

            // Update the node's transformation matrix
            self.current_transformation_matrix = transformation_so_far * transformation;

            // Recurse
            for &child in &self.children {
                (*child).update_node_transformations(&self.current_transformation_matrix);
            }
        }
    }

    // This is just to complete the assignment, this drawing 
    // function ignores instancing
    pub fn draw(&self, camera: &Camera) {
        unsafe {
            match &self.geometric_object {
                Some(g) => {
                    // Again, we don't care about perfomance and always update transform
                    g.update_transform(0, &self.current_transformation_matrix);
                    g.draw_all();
                },
                None => ()
            };

            // Check if node is drawable, set uniforms, draw
            for &child in &self.children {
                (*child).draw(&camera);
            }
        }
    }
}
