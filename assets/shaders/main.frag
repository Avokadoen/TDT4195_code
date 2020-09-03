#version 430 core

out vec4 color;

void main()
{
    float depth = gl_FragCoord.z / gl_FragCoord.w;
    color = vec4(vec3(1, 1, 1) / depth, 1);
}