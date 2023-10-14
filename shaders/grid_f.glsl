#version 330 core
in vec3 nearPoint;
in vec3 farPoint;

out vec4 FragColor;

uniform mat4 projection;
uniform mat4 view;
uniform float near;
uniform float far;

vec4 grid(vec3 fragPos3D, float scale, float lineWidth, float lineOpacity) {
    vec2 coord = fragPos3D.xz * scale; // use the scale variable to set the distance between the lines
    vec2 derivative = fwidth(coord);
    vec2 grid = abs(fract(coord - 0.5) - 0.5) / derivative;
    float line = min(grid.x, grid.y);
    float minimumz = min(derivative.y, 1);
    float minimumx = min(derivative.x, 1);
    vec4 color = vec4(0.2, 0.2, 0.2, lineOpacity * (1.0 - min(line, 1.0)));
    // z axis
    if(abs(fragPos3D.x) < lineWidth * minimumx)
        color.z = 1.0;
    // x axis
    if(abs(fragPos3D.z) < lineWidth * minimumz)
        color.x = 1.0;
    return color;
}

float computeDepth(vec3 pos) {
    vec4 clip_space_pos = projection * view * vec4(pos.xyz, 1.0);
    return (clip_space_pos.z / clip_space_pos.w);
}

float computeLinearDepth(vec3 pos) {
    vec4 clip_space_pos = projection * view * vec4(pos.xyz, 1.0);
    float clip_space_depth = (clip_space_pos.z / clip_space_pos.w) * 2.0 - 1.0; // put back between -1 and 1
    float linearDepth = (2.0 * near * far) / (far + near - clip_space_depth * (far - near)); // get linear value between 0.01 and 100
    return linearDepth / far; // normalize
}

void main()
{
    float t = -nearPoint.y / (farPoint.y - nearPoint.y);
    vec3 fragPos3D = nearPoint + t * (farPoint - nearPoint);
    //gl_FragDepth = computeDepth(fragPos3D);
    // above depth calculation is buggy
    // see https://stackoverflow.com/questions/72791713/issues-with-infinite-grid-in-opengl-4-5-with-glsl
    // for solution
    gl_FragDepth = ((gl_DepthRange.diff * computeDepth(fragPos3D)) +
                gl_DepthRange.near + gl_DepthRange.far) / 2.0;

    float linearDepth = computeLinearDepth(fragPos3D);
    float fading = max(0, (0.5 - linearDepth));

    float lineWidth = 0.1; // width of the grid lines in world space units
    float lineOpacity = 0.7; // opacity of the grid lines

    // Compute the grid colors at different resolutions and blend them together
    FragColor = (grid(fragPos3D, 5, lineWidth, lineOpacity) + grid(fragPos3D, 0.5, lineWidth, lineOpacity)) * float(t > 0);

    // Fade the grid out as it gets farther away
    FragColor.a *= fading;
}
