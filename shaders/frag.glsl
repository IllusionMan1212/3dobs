#version 330 core
in vec3 fragNormals;
in vec3 fragPos;
in vec2 texCoords;

out vec4 FragColor;

struct Material {
  sampler2D texture_diffuse;
  sampler2D texture_specular;
  sampler2D texture_ambient;

  vec3 ambient;
  vec3 diffuse;
  vec3 specular;
  float shininess;
  float opacity;
};

struct DirLight {
  vec3 direction;

  vec3 ambient;
  vec3 diffuse;
  vec3 specular;
};

struct PointLight {
  vec3 position;

  vec3 ambient;
  vec3 diffuse;
  vec3 specular;

  float constant;
  float linear;
  float quadratic;
};

struct SpotLight {
  vec3 position;
  vec3 direction;
  float cutOff;
  float outerCutOff;

  vec3 ambient;
  vec3 diffuse;
  vec3 specular;

  float constant;
  float linear;
  float quadratic;
};

#define NR_POINT_LIGHTS 4
#define blinn true

uniform vec3 viewPos;
uniform Material material;
uniform DirLight dirLight;
uniform PointLight pointLights[NR_POINT_LIGHTS];
uniform SpotLight spotLight;
uniform bool useTextures;

vec3 CalculateDirLight(DirLight light, vec3 normal, vec3 viewDir) {
  vec3 lightDir = normalize(-light.direction);

  float diff = max(dot(lightDir, normal), 0.0);
  float spec = 0.0f;

  if (blinn) {
    vec3 halfwayDir = normalize(lightDir + viewDir);
    spec = pow(max(dot(normal, halfwayDir), 0.0), material.shininess);
  } else {
    vec3 reflectDir = reflect(-lightDir, normal);
    spec = pow(max(dot(viewDir, reflectDir), 0.0), material.shininess);
  }

  vec3 ambient = vec3(0.0, 0.0, 0.0);
  vec3 diffuse = vec3(0.0, 0.0, 0.0);
  vec3 specular = vec3(0.0, 0.0, 0.0);

  if (useTextures) {
    ambient = light.ambient * vec3(texture(material.texture_ambient, texCoords)) * material.ambient;
    diffuse = light.diffuse * diff * vec3(texture(material.texture_diffuse, texCoords)) * material.diffuse;
    specular = light.specular * spec * vec3(texture(material.texture_specular, texCoords)) * material.specular;
  } else {
    ambient = light.ambient * material.ambient;
    diffuse = light.diffuse * (diff * material.diffuse);
    specular = light.specular * (spec * material.specular);
  }

  return (ambient + diffuse + specular);
}

vec3 CalculatePointLight(PointLight light, vec3 normal, vec3 fragPos, vec3 viewDir) {
  vec3 lightDir = normalize(light.position - fragPos);

  float diff = max(dot(lightDir, normal), 0.0);
  float spec = 0.0f;

  if (blinn) {
    vec3 halfwayDir = normalize(lightDir + viewDir);
    spec = pow(max(dot(normal, halfwayDir), 0.0), material.shininess);
  } else {
    vec3 reflectDir = reflect(-lightDir, normal);
    spec = pow(max(dot(viewDir, reflectDir), 0.0), material.shininess);
  }

  // attenuation
  float distance = length(light.position - fragPos);
  float attenuation = 1.0 / (light.constant + light.linear * distance + light.quadratic * (distance + distance));

  vec3 ambient = vec3(0.0, 0.0, 0.0);
  vec3 diffuse = vec3(0.0, 0.0, 0.0);
  vec3 specular = vec3(0.0, 0.0, 0.0);

  if (useTextures) {
    ambient = light.ambient * vec3(texture(material.texture_ambient, texCoords)) * material.ambient;
    diffuse = light.diffuse * diff * vec3(texture(material.texture_diffuse, texCoords)) * material.diffuse;
    specular = light.specular * spec * vec3(texture(material.texture_specular, texCoords)) * material.specular;
  } else {
    ambient = light.ambient * material.ambient;
    diffuse = light.diffuse * (diff * material.diffuse);
    specular = light.specular * (spec * material.specular);
  }

  ambient *= attenuation;
  diffuse *= attenuation;
  specular *= attenuation;

  return (ambient + diffuse + specular);
}

vec3 CalculateSpotLight(SpotLight light, vec3 normal, vec3 fragPos, vec3 viewDir) {
  vec3 lightDir = normalize(light.position - fragPos);

  float diff = max(dot(lightDir, normal), 0.0);
  float spec = 0.0f;

  if (blinn) {
    vec3 halfwayDir = normalize(lightDir + viewDir);
    spec = pow(max(dot(normal, halfwayDir), 0.0), material.shininess);
  } else {
    vec3 reflectDir = reflect(-lightDir, normal);
    spec = pow(max(dot(viewDir, reflectDir), 0.0), material.shininess);
  }

  // attenuation
  float distance = length(light.position - fragPos);
  float attenuation = 1.0 / (light.constant + light.linear * distance + light.quadratic * (distance + distance));

  // spotlight
  float theta = dot(lightDir, normalize(-light.direction));
  float epsilon = light.cutOff - light.outerCutOff;
  float intensity = clamp((theta - light.outerCutOff) / epsilon, 0.0, 1.0);

  vec3 ambient = vec3(0.0, 0.0, 0.0);
  vec3 diffuse = vec3(0.0, 0.0, 0.0);
  vec3 specular = vec3(0.0, 0.0, 0.0);

  if (useTextures) {
    ambient = light.ambient * vec3(texture(material.texture_ambient, texCoords)) * material.ambient;
    diffuse = light.diffuse * diff * vec3(texture(material.texture_diffuse, texCoords)) * material.diffuse;
    specular = light.specular * spec * vec3(texture(material.texture_specular, texCoords)) * material.specular;
  } else {
    ambient = light.ambient * material.ambient;
    diffuse = light.diffuse * (diff * material.diffuse);
    specular = light.specular * (spec * material.specular);
  }

  diffuse *= intensity;
  specular *= intensity;

  ambient *= attenuation;
  diffuse *= attenuation;
  specular *= attenuation;

  return (ambient + diffuse + specular);
}

void main()
{
  vec3 norm = normalize(fragNormals);
  vec3 viewDir = normalize(viewPos - fragPos);

  // direction light
  vec3 result = CalculateDirLight(dirLight, norm, viewDir);

  // point lights
  for (int i = 0; i < NR_POINT_LIGHTS; i++) {
    result += CalculatePointLight(pointLights[i], norm, fragPos, viewDir);
  }

  result += CalculateSpotLight(spotLight, norm, fragPos, viewDir);

  FragColor = vec4(result, material.opacity);
}

