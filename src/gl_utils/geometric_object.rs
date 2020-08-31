use gl;
use gl::types::{GLuint, GLsizei};

use super::{
    helpers,
    bindable::Bindable
};

pub struct GeometricObject {
    id: GLuint,
    vbo_ids: [GLuint; 2],
    pub count: GLsizei
}


impl Drop for GeometricObject {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(2, self.vbo_ids.as_ptr());
            gl::DeleteVertexArrays(1, self.id as *const GLuint);
        }
    }
}

impl Bindable for GeometricObject {
    fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.id);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo_ids[GeometricObject::VERT_INDX]);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.vbo_ids[GeometricObject::INDC_INDX]);
        }
    }

    fn unbind(&self) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }
    }
}

// TODO: errors
impl GeometricObject {
    pub const VERT_INDX: usize = 0;
    pub const INDC_INDX: usize = 1;

    pub fn init(vertices: &Vec<f32>, indices: &Vec<u32>) -> GeometricObject  {
        let mut id: GLuint = 0;
        let mut vbo_ids: [GLuint; 2] = [0; 2];

        unsafe {
            gl::GenVertexArrays(1, &mut id);
            gl::BindVertexArray(id);

            gl::GenBuffers(2, vbo_ids.as_mut_ptr());

            // instantiate vertices buffer
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo_ids[GeometricObject::VERT_INDX]);
            gl::BufferData(
                gl::ARRAY_BUFFER, 
                helpers::byte_size_of_array(vertices),
                helpers::array_to_c_void(vertices),
                gl::STATIC_DRAW
            );
            
            // instantiate indices buffer
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, vbo_ids[GeometricObject::INDC_INDX]);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER, 
                helpers::byte_size_of_array(indices),
                helpers::array_to_c_void(indices),
                gl::STATIC_DRAW
            );
            
            // Vertex attributes
            gl::EnableVertexAttribArray(0);
            let components = 3;
            let stride = components * helpers::size_of::<f32>();
            gl::VertexAttribPointer(
                0,                      // index of the generic vertex attribute ("layout (location = 0)")
                components,             // the number of components per generic vertex attribute
                gl::FLOAT,              // data type
                gl::FALSE,              // normalized (int-to-float conversion)
                stride,                 // stride (byte offset between consecutive attributes)
                std::ptr::null()        // offset of the first component
            );
            
            // Better safe than sorry :) 
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }

        GeometricObject {
            id,
            vbo_ids,
            count: indices.len() as i32
        }
    }
}