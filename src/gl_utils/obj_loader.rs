use std::str::FromStr;
use std::num::ParseFloatError;
use crate::gl_utils::geometric_object::GeometricObject;
use std::{fmt, path::Path};

pub enum ObjParseError {
    PathReadError(std::io::Error)
}

impl fmt::Display for ObjParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ObjParseError::PathReadError(e) => e.fmt(f)
        }
    }
}

// TODO: evaluate if we can stream data directly to GPU somehow
pub struct ParsedObj {
    pub vertices: Vec<f32>,
    pub faces: Vec<u32> // TODO: remember this starts with 1 for some reason
}

pub fn load_and_parse_obj(obj_path: &str) -> Result<ParsedObj, ObjParseError> {
    let path = Path::new(obj_path);

    let obj_data = {
        let file_content = std::fs::read_to_string(path);
        let data = match file_content {
            Ok(s) => s,
            Err(e) => return Err(ObjParseError::PathReadError(e))
        };
        data
    };

    parse_obj(&obj_data)
}

pub fn parse_obj(obj_data: &str) -> Result<ParsedObj, ObjParseError> { 
    // TODO: Allow callee to adjust pre-allocation or do some smart analysis of the file
    let mut vertices = Vec::<f32>::with_capacity(1000);
    let mut faces = Vec::<u32>::with_capacity(100);

    let mut valuesf32: [Option<f32>; 4] = [None; 4];
    let mut valuesu32: [Option<u32>; 4] = [None; 4];
    // TODO: better error handling
    for line in obj_data.lines() {
        let line = line.trim_start();
        // TODO: g
        if line.starts_with("#") {
            continue;
        } else if line.starts_with("vp") {
            todo!("space vertices");
        } else if line.starts_with("vn") {
            // todo!("vertex normals");
        } else if line.starts_with("vt") {
            todo!("uv mapping");
        } else if line.starts_with("g") {
            // todo!("groups");
        }else if line.starts_with("v") {
            parse_line::<f32>(&mut valuesf32, line);
            for i in 0..valuesf32.len() {
                match valuesf32[i] {
                    Some(v) => vertices.push(v),
                    None => {
                        // If w component
                        if i == 3 {
                            vertices.push(1.0);
                        }
                    }
                }

                valuesf32[i] = None;
            }
        } else if line.starts_with("f") {
            parse_line::<u32>(&mut valuesu32, line);
            for i in 0..valuesu32.len() {
                if i == 3 {
                    break;
                }

                match valuesu32[i] {
                    Some(v) => faces.push(v - 1),
                    None => { eprintln!("error parsing faces") } // TODO
                 }

                valuesu32[i] = None;
            }
        }

    }
                
    return Ok(
        ParsedObj {
            vertices,
            faces
        }
    );
}

// Internal function, so we can do ugly stuff like out variables (gotta go fast)
// Use of array is to keep data on the stack
fn parse_line<T: FromStr>(target: &mut [Option<T>; 4], line: &str) {
    let mut values = line.trim_start().split_whitespace();
    values.next(); // first is always irrelevant
    let values = values;
    
    let mut i = 0;
    for value in values {
        if i > 3 {
            break;
        }

        target[i] = match value.parse::<T>() {
            Ok(v) => Some(v),
            Err(_) => None
        };

        i += 1;
    }
} 

#[test]
fn parse_v_simple_works() {
    // Source: https://people.sc.fsu.edu/~jburkardt/data/obj/tetrahedron.obj
    let tetrahedon = String::from(
        "# tetrahedron.obj created by hand.
        #
        
        g tetrahedron
        
        v 1.00 1.00 1.00
        v 2.00 1.00 1.00
        v 1.00 2.00 1.00
        v 1.00 1.00 2.00
        
        f 1 3 2
        f 1 4 3
        f 1 2 4
        f 2 3 4"
    );

    match parse_obj(&tetrahedon) {
        Ok(o) => {
            let exp_vertices = vec![
                1.0, 1.0, 1.0, 1.0,
                2.0, 1.0, 1.0, 1.0,
                1.0, 2.0, 1.0, 1.0,
                1.0, 1.0, 2.0, 1.0
            ];
            assert_eq!(o.vertices, exp_vertices);

            let exp_faces = vec![
                1, 3, 2,
                1, 4, 3,
                1, 2, 4,
                2, 3, 4
            ];
            assert_eq!(o.faces, exp_faces);
        }
        Err(_) => {
            panic!("parse_v_simple_works failed");
        }
    }
}
// TODO: Tests