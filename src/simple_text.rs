use crate::gl;

macro_rules! cstr {
    ($e:expr) => {
        concat!($e, "\0").as_ptr() as *const std::os::raw::c_char
    };
}

// internal vertex
type Vertex = [f32;2];

// use 15 verts to hold all possible letters
// 0 - 1 - 2
// 3 - 4 - 5
// 6 - 7 - 8
// 9 - 10- 11
// 12- 13- 14
const VERTS: [Vertex;15] = [
    [0.0, 1.0], [0.25, 1.0], [0.5, 1.0],
    [0.0, 0.75], [0.25, 0.75], [0.5, 0.75],
    [0.0, 0.5], [0.25, 0.5], [0.5, 0.5],
    [0.0, 0.25], [0.25, 0.25], [0.5, 0.25],
    [0.0, 0.0], [0.25, 0.0], [0.5, 0.0],
];

// start/end char values
const START: usize = '!' as usize;
const END: usize = 'Z' as usize;

// use index arrays to create letters
const LETTERS: [[i32;15];END - START + 1] = [
    [4, 1, 7, 13, 14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],      // !
    [4, 1, 4, 2, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],        // "
    [8, 0, 12, 2, 14, 3, 5, 9, 11, 0, 0, 0, 0, 0, 0],     // #
    [8, 2, 3, 3, 11, 11, 12, 1, 13, 0, 0, 0, 0, 0, 0],    // $
    [14, 2, 12, 0, 3, 3, 1, 1, 0, 11, 13, 13, 14, 14, 11], // %
    [14, 14, 3, 3, 1, 1, 7, 7, 9, 9, 12, 12, 13, 13, 11], // &
    [2, 1, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],        // '
    [6, 1, 3, 3, 9, 9, 13, 0, 0, 0, 0, 0, 0, 0, 0],       // (
    [6, 1, 5, 5, 11, 11, 13, 0, 0, 0, 0, 0, 0, 0, 0],     // )
    [6, 0, 8, 6, 2, 1, 7, 0, 0, 0, 0, 0, 0, 0, 0],        // *
    [4, 6, 8, 4, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],       // +
    [2, 10, 12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],      // ,
    [2, 6, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],        // -
    [8, 12, 13, 13, 10, 10, 9, 9, 12, 0, 0, 0, 0, 0, 0],  // .
    [2, 2, 12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],       // /
    [10, 0, 2, 2, 14, 14, 12, 12, 0, 0, 14, 0, 0, 0, 0],  // 0
    [6, 3, 1, 1, 13, 12, 14, 0, 0, 0, 0, 0, 0, 0, 0],     // 1
    [10, 3, 1, 1, 5, 5, 8, 8, 12, 12, 14, 0, 0, 0, 0],    // 2
    [12, 0, 1, 1, 5, 5, 11, 11, 13, 13, 12, 8, 7, 0, 0],  // 3
    [6, 14, 2, 2, 6, 6, 8, 0, 0, 0, 0, 0, 0, 0, 0],       // 4
    [12, 2, 0, 0, 6, 6, 7, 7, 11, 11, 13, 13, 12, 0, 0],  // 5
    [14, 2, 1, 1, 3, 3, 12, 12, 13, 13, 11, 11, 7, 7, 6], // 6
    [4, 0, 2, 2, 12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],       // 7
    [12, 1, 3, 3, 11, 11, 13, 13, 9, 9, 5, 5, 1, 0, 0],   // 8
    [10, 2, 1, 1, 3, 3, 7, 7, 8, 2, 14, 0, 0, 0, 0],      // 9
    [4, 1, 4, 10, 13, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],      // :
    [4, 1, 4, 10, 12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],      // ;
    [4, 5, 6, 6, 11, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],       // <
    [4, 5, 3, 9, 11, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],       // =
    [4, 3, 8, 8, 9, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],        // >
    [8, 3, 1, 1, 5, 5, 7, 7, 10, 0, 0, 0, 0, 0, 0],       // ?
    [14, 4, 7, 7, 8, 8, 5, 5, 1, 1, 3, 3, 9, 9, 14],      // @
    [10, 1, 6, 1, 8, 6, 12, 8, 14, 6, 8, 0, 0, 0, 0],     // A
    [14, 0, 12, 0, 5, 5, 7, 7, 6, 7, 11, 11, 13, 13, 12], // B
    [10, 2, 1, 1, 3, 3, 9, 9, 13, 13, 14, 0, 0, 0, 0],    // C
    [12, 0, 1, 1, 5, 5, 11, 11, 13, 13, 12, 12, 0, 0, 0], // D
    [8, 0, 12, 0, 2, 6, 7, 12, 14, 0, 0, 0, 0, 0, 0],     // E
    [6, 0, 12, 0, 2, 6, 7, 0, 0, 0, 0, 0, 0, 0, 0],       // F
    [14, 2, 1, 1, 3, 3, 9, 9, 13, 13, 11, 11, 8, 8, 7],   // G
    [6, 0, 12, 6, 8, 2, 14, 0, 0, 0, 0, 0, 0, 0, 0],      // H
    [6, 0, 2, 1, 13, 12, 14, 0, 0, 0, 0, 0, 0, 0, 0],     // I
    [8, 1, 2, 2, 11, 11, 13, 13, 9, 0, 0, 0, 0, 0, 0],    // J
    [6, 0, 12, 6, 2, 6, 14, 0, 0, 0, 0, 0, 0, 0, 0],      // K
    [4, 0, 12, 12, 14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],     // L
    [8, 0, 12, 0, 7, 7, 2, 2, 14, 0, 0, 0, 0, 0, 0],      // M
    [6, 0, 12, 0, 14, 14, 2, 0, 0, 0, 0, 0, 0, 0, 0],     // N
    [12, 1, 5, 5, 11, 11, 13, 13, 9, 9, 3, 3, 1, 0, 0],   // O
    [10, 0, 12, 0, 1, 1, 5, 5, 7, 7, 6, 0, 0, 0, 0],      // P
    [12, 0, 12, 12, 13, 13, 11, 11, 2, 2, 0, 10, 14, 0, 0],  // Q
    [12, 0, 12, 0, 1, 1, 5, 5, 7, 7, 6, 7, 14, 0, 0],     // R
    [14, 2, 1, 1, 3, 3, 6, 6, 8, 8, 11, 11, 13, 13, 12],  // S
    [4, 0, 2, 1, 13, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],       // T
    [8, 0, 9, 9, 13, 13, 11, 11, 2, 0, 0, 0, 0, 0, 0],    // U
    [4, 0, 13, 13, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],      // V
    [8, 0, 12, 12, 1, 1, 14, 14, 2, 0, 0, 0, 0, 0, 0],    // W
    [4, 0, 14, 2, 12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],      // X
    [6, 0, 7, 7, 2, 7, 13, 0, 0, 0, 0, 0, 0, 0, 0],       // Y
    [6, 0, 2, 2, 12, 12, 14, 0, 0, 0, 0, 0, 0, 0, 0],      // Z
];

const VERTEX_SHADER_CODE: &str = r"
#version 430 core
layout (location=0) in vec2 position;
uniform vec2 offset;
uniform vec2 scale;
void main() {
    float x = position.x * scale.x + offset.x;
    float y = position.y * scale.y + offset.y;
    gl_Position = vec4(x, y, 0.0, 1.0);
}
";

const FRAGMENT_SHADER_CODE: &str = r"
#version 430 core
out vec4 f_color;
void main() {
    f_color = vec4(1.0, 1.0, 1.0, 1.0);
}
";

// Draws a string of length <len> at an <x>/<y> position on the screen, optionally with a <shadow>
//  The screen is set up to be 0,0 at the lower left and 150,150 at the upper right for positioning
//  by default.
pub fn draw_text(string: &str, x: f32, y: f32, xscale: f32, yscale: f32) {
    if string.len() <= 0 {
        return;
    }

    let program = {
        let vcode = VERTEX_SHADER_CODE;
        let fcode = FRAGMENT_SHADER_CODE;
        gl_helpers::ProgramBuilder::new()
            .vertex_shader_code(vcode)
            .fragment_shader_code(fcode)
            .build()
            .unwrap()
    };

    let offset_uniform_location = gl_helpers::get_uniform_location(program, cstr!("offset")).unwrap();
    let scale_uniform_location = gl_helpers::get_uniform_location(program, cstr!("scale")).unwrap();

    unsafe {
        gl::UseProgram(program);
        gl::Uniform2f(scale_uniform_location, xscale, yscale);
    }

    let mut x = x;
    for c in string.chars() {
        let temp = c.to_ascii_uppercase();
        let temp = temp as usize;

        if temp >= START && temp <= END {
            let vcount = LETTERS[temp - START][0] as usize;
            let vertices: Vec<f32> = LETTERS[temp - START][1..].iter().take(vcount).flat_map(
                |index| VERTS[*index as usize].iter().copied()
            ).collect();

            let buffer = gl_helpers::create_buffer(&vertices, gl_helpers::BufferUsage::StreamDraw).unwrap();
            let vao = gl_helpers::create_single_buffer_vertex_array(buffer, &[2]).unwrap();

            unsafe {
                gl::Uniform2f(offset_uniform_location, x, y);
                gl::BindVertexArray(vao);
                gl::DrawArrays(gl::LINES, 0, vcount as i32);

                gl::DeleteVertexArrays(1, &vao);
                gl::DeleteBuffers(1, &buffer);
            }
        }

        x += xscale * 0.675;
    }

    unsafe {
        gl::DeleteProgram(program);
    }
}