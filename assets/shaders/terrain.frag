#version 430 core

in vec3 vert_normals;
in vec4 vert_colors;

out vec4 color;

// SOURCE: https://learnopengl.com/Advanced-OpenGL/Depth-testing
float near = 0.01;
float far = 40;

void main()
{
    float ndc = gl_FragCoord.z * 2.0 - 1.0; 
    float linearDepth = (2.0 * near * far) / (far + near - ndc * (far - near));
    float inverseDepth =  (1 - linearDepth);
    vec3 depthColor = vec3(vert_colors.x * inverseDepth, vert_colors.y * inverseDepth, vert_colors.z * inverseDepth);	
    color = vec4(depthColor, 1);
}