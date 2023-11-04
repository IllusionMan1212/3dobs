#version 330 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aNormal;
layout (location = 2) in vec2 aTexCoords;
layout (location = 3) in vec3 aTangent;
layout (location = 4) in vec3 aBitangent;
out vec3 fragNormals;
out vec3 fragPos;
out vec2 texCoords;
out mat3 fragTBN;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;
uniform mat3 normalMatrix;
uniform bool useNormalMatrix;

void main()
{
  gl_Position = projection * view * model * vec4(aPos, 1.0f);
  if (useNormalMatrix)
    fragNormals = normalMatrix * aNormal;
  else
    fragNormals = mat3(transpose(inverse(model))) * aNormal;

  vec3 T = normalize(vec3(model * vec4(aTangent, 0.0)));
  vec3 B = normalize(vec3(model * vec4(aBitangent, 0.0)));
  vec3 N = normalize(vec3(model * vec4(aNormal, 0.0)));
  mat3 TBN = mat3(T, B, N);

  fragPos = vec3(model * vec4(aPos, 1.0));
  texCoords = aTexCoords;
  fragTBN = TBN;
}

