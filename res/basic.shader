#shader vertex
#version 430 core
layout (location = 0) in vec2 pos;
void main() {
    gl_Position = vec4(pos, 0.0, 1.0);
}

#shader fragment
#version 430 core
out vec4 outColor;
void main() {
    outColor = vec4(0.8, 0.4, 0.0, 1.0);
}
