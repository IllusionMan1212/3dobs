#![allow(non_camel_case_types)]

use std::collections::HashMap;
use std::io::Read;

use hard_xml::XmlRead;
use log::info;

use crate::aabb::AABB;
use crate::importer::ObjMesh;
use crate::mesh::Vertex;

use super::Object;

#[derive(Default, Debug)]
enum UpAxis {
    X_UP,
    #[default]
    Y_UP,
    Z_UP,
}

impl std::str::FromStr for UpAxis {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "X_UP" => Ok(Self::X_UP),
            "Y_UP" => Ok(Self::Y_UP),
            "Z_UP" => Ok(Self::Z_UP),
            _ => Err(format!("Invalid UpAxis: {}", s)),
        }
    }
}

#[derive(Default, Debug)]
enum AltitudeMode {
    #[default]
    RelativeToGround,
    Absolute,
}

impl std::str::FromStr for AltitudeMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "absolute" => Ok(Self::Absolute),
            "relativeToGround" => Ok(Self::RelativeToGround),
            _ => Err(format!("Invalid AltitudeMode: {}", s)),
        }
    }
}

#[derive(XmlRead, Debug)]
enum Library {
    #[xml(tag = "library_animation_clips")]
    AnimationClips(LibraryAnimationClips),
    #[xml(tag = "library_animations")]
    Animations(LibraryAnimations),
    #[xml(tag = "library_articulated_systems")]
    ArticulatedSystems(LibraryArticulatedSystems),
    #[xml(tag = "library_cameras")]
    Cameras(LibraryCameras),
    #[xml(tag = "library_controllers")]
    Controllers(LibraryControllers),
    #[xml(tag = "library_effects")]
    Effects(LibraryEffects),
    #[xml(tag = "library_force_fields")]
    ForceFields(LibraryForceFields),
    #[xml(tag = "library_formulas")]
    Formulas(LibraryFormulas),
    #[xml(tag = "library_geometries")]
    Geometries(LibraryGeometries),
    #[xml(tag = "library_images")]
    Images(LibraryImages),
    #[xml(tag = "library_joints")]
    Joints(LibraryJoints),
    #[xml(tag = "library_kinematics_models")]
    KinematicsModels(LibraryKinematicsModels),
    #[xml(tag = "library_kinematics_scenes")]
    KinematicsScenes(LibraryKinematicsScenes),
    #[xml(tag = "library_lights")]
    Lights(LibraryLights),
    #[xml(tag = "library_materials")]
    Materials(LibraryMaterials),
    #[xml(tag = "library_nodes")]
    Nodes(LibraryNodes),
    #[xml(tag = "library_physics_materials")]
    PhysicsMaterials(LibraryPhysicsMaterials),
    #[xml(tag = "library_physics_models")]
    PhysicsModels(LibraryPhysicsModels),
    #[xml(tag = "library_physics_scenes")]
    PhysicsScenes(LibraryPhysicsScenes),
    #[xml(tag = "library_visual_scenes")]
    VisualScenes(LibraryVisualScenes),
}

#[derive(XmlRead, Debug)]
#[xml(tag = "technique_hint")]
struct TechniqueHint {
    #[xml(attr = "platform")]
    attr_platform: Option<String>,
    #[xml(attr = "ref")]
    attr_ref: String,
    #[xml(attr = "profile")]
    attr_profile: Option<String>,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "newparam")]
struct NewParam {
    #[xml(attr = "sid")]
    attr_sid: String,

    // TODO: children
}

#[derive(XmlRead, Debug)]
#[xml(tag = "setparam")]
struct SetParam {
    #[xml(attr = "ref")]
    attr_ref: String,

    // TODO: the child element can be any of Parameter-Type Elements
    // #[xml(child = "TODO")]
    // child: (),
}

#[derive(XmlRead, Debug)]
#[xml(tag = "instance_effect")]
struct InstanceEffect {
    #[xml(attr = "sid")]
    attr_sid: Option<String>,
    #[xml(attr = "name")]
    attr_name: Option<String>,
    #[xml(attr = "url")]
    attr_url: String,

    #[xml(child = "technique_hint")]
    technique_hints: Vec<TechniqueHint>,
    #[xml(child = "setparam")]
    setparams: Vec<SetParam>,
    #[xml(child = "extra")]
    extras: Vec<Extra>,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "material")]
struct Material {
    #[xml(attr = "id")]
    attr_id: Option<String>,
    #[xml(attr = "name")]
    attr_name: Option<String>,

    #[xml(child = "asset")]
    asset: Option<Asset>,
    #[xml(child = "instance_effect")]
    instance_effect: InstanceEffect,
    #[xml(child = "extra")]
    extras: Vec<Extra>,
}

// NOTE: this panics if the enum value is not of type struct or enum
// e.g bool, String, f32
// #[derive(XmlRead, Debug)]
// enum ColladaType {
//     #[xml(tag = "bool")]
//     Bool(bool),
//     #[xml(tag = "bool2")]
//     Bool2([bool; 2]),
//     #[xml(tag = "bool3")]
//     Bool3([bool; 3]),
//     #[xml(tag = "bool4")]
//     Bool4([bool; 4]),
//     #[xml(tag = "int")]
//     Int(i32),
//     #[xml(tag = "int2")]
//     Int2([i32; 2]),
//     #[xml(tag = "int3")]
//     Int3([i32; 3]),
//     #[xml(tag = "int4")]
//     Int4([i32; 4]),
//     #[xml(tag = "float")]
//     Float(f32),
//     #[xml(tag = "float2")]
//     Float2([f32; 2]),
//     #[xml(tag = "float3")]
//     Float3([f32; 3]),
//     #[xml(tag = "float4")]
//     Float4([f32; 4]),
//     #[xml(tag = "float2x2")]
//     Float2x2([[f32; 2]; 2]),
//     #[xml(tag = "float3x3")]
//     Float3x3([[f32; 3]; 3]),
//     #[xml(tag = "float4x4")]
//     Float4x4([[f32; 4]; 4]),
//     #[xml(tag = "string")]
//     String(String),
// }

#[derive(XmlRead, Debug)]
#[xml(tag = "annotate")]
struct Annotate {
    #[xml(attr = "name")]
    attr_name: String,

    // #[xml(
    //     child = "bool",
    //     child = "bool2",
    //     child = "bool3",
    //     child = "bool4",
    //     child = "int",
    //     child = "int2",
    //     child = "int3",
    //     child = "int4",
    //     child = "float",
    //     child = "float2",
    //     child = "float3",
    //     child = "float4",
    //     child = "float2x2",
    //     child = "float3x3",
    //     child = "float4x4",
    //     child = "string",
    // )]
    // value: ColladaType,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "effect")]
struct Effect {
    #[xml(attr = "id")]
    attr_id: String,
    #[xml(attr = "name")]
    attr_name: Option<String>,

    #[xml(child = "asset")]
    asset: Option<Asset>,
    
    #[xml(child = "annotate")]
    annotations: Vec<Annotate>,
    // TODO: newparam
    // TODO: _profile_

    #[xml(child = "extra")]
    extras: Vec<Extra>,
}

#[derive(XmlRead, Debug)]
enum GeometricElement {
    #[xml(tag = "convex_mesh")]
    ConvexMesh(ConvexMesh),
    #[xml(tag = "mesh")]
    Mesh(Mesh),
    #[xml(tag = "spline")]
    Spline(Spline),
    #[xml(tag = "brep")]
    Brep(Brep),
}

#[derive(XmlRead, Debug)]
enum Primitive {
    #[xml(tag = "lines")]
    Lines(Lines),
    #[xml(tag = "linestrips")]
    LineStrips(LineStrips),
    #[xml(tag = "polygons")]
    Polygons(Polygons),
    #[xml(tag = "polylist")]
    PolyList(PolyList),
    #[xml(tag = "triangles")]
    Triangles(Triangles),
    #[xml(tag = "trifans")]
    TriFans(TriFans),
    #[xml(tag = "tristrips")]
    TriStrips(TriStrips),
}

#[derive(XmlRead, Debug)]
#[xml(tag = "lines")]
struct Lines {
    #[xml(attr = "name")]
    attr_name: Option<String>,
    #[xml(attr = "count")]
    attr_count: u32,
    #[xml(attr = "material")]
    attr_material: Option<String>,

    #[xml(child = "input")]
    inputs: Vec<InputShared>,
    #[xml(child = "p")]
    p: Option<PrimitiveData>,
    #[xml(child = "extra")]
    extras: Vec<Extra>
}

#[derive(XmlRead, Debug)]
#[xml(tag = "linestrips")]
struct LineStrips {
    #[xml(attr = "name")]
    attr_name: Option<String>,
    #[xml(attr = "count")]
    attr_count: u32,
    #[xml(attr = "material")]
    attr_material: Option<String>,

    #[xml(child = "input")]
    inputs: Vec<InputShared>,
    #[xml(child = "p")]
    p: Vec<PrimitiveData>,
    #[xml(child = "extra")]
    extras: Vec<Extra>
}

#[derive(XmlRead, Debug)]
#[xml(tag = "polygons")]
struct Polygons {
    #[xml(attr = "count")]
    attr_count: u32,
    #[xml(attr = "material")]
    attr_material: Option<String>,
    #[xml(attr = "name")]
    attr_name: Option<String>,

    #[xml(child = "input")]
    inputs: Vec<InputShared>,
    #[xml(child = "p")]
    p: Vec<PrimitiveData>,
    #[xml(child = "ph")]
    ph: Vec<PrimitiveWithHolesData>,
    #[xml(child = "extra")]
    extras: Vec<Extra>
}

#[derive(XmlRead, Debug)]
#[xml(tag = "polylist")]
struct PolyList {
    #[xml(attr = "name")]
    attr_name: Option<String>,
    #[xml(attr = "count")]
    attr_count: u32,
    #[xml(attr = "material")]
    attr_material: Option<String>,

    #[xml(child = "input")]
    inputs: Vec<InputShared>,
    #[xml(child = "vcount")]
    vcount: Option<VCount>,
    #[xml(child = "p")]
    p: Option<PrimitiveData>,
    #[xml(child = "extra")]
    extras: Vec<Extra>
}

#[derive(XmlRead, Debug)]
#[xml(tag = "vcount")]
struct VCount {
    #[xml(text)]
    value: String,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "p")]
struct PrimitiveData {
    #[xml(text)]
    value: String,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "p")]
struct PrimitiveWithHolesData {
    #[xml(child = "p")]
    p: PrimitiveData,
    #[xml(child = "h")]
    h: Vec<PrimitiveData>,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "triangles")]
struct Triangles {
    #[xml(attr = "name")]
    attr_name: Option<String>,
    #[xml(attr = "count")]
    attr_count: u32,
    #[xml(attr = "material")]
    attr_material: Option<String>,

    #[xml(child = "input")]
    inputs: Vec<InputShared>,
    #[xml(child = "p")]
    p: Option<PrimitiveData>,
    #[xml(child = "extra")]
    extras: Vec<Extra>
}

#[derive(XmlRead, Debug)]
#[xml(tag = "trifans")]
struct TriFans {
    #[xml(attr = "name")]
    attr_name: Option<String>,
    #[xml(attr = "count")]
    attr_count: u32,
    #[xml(attr = "material")]
    attr_material: Option<String>,

    #[xml(child = "input")]
    inputs: Vec<InputShared>,
    #[xml(child = "p")]
    p: Vec<PrimitiveData>,
    #[xml(child = "extra")]
    extras: Vec<Extra>
}

#[derive(XmlRead, Debug)]
#[xml(tag = "tristrips")]
struct TriStrips {
    #[xml(attr = "name")]
    attr_name: Option<String>,
    #[xml(attr = "count")]
    attr_count: u32,
    #[xml(attr = "material")]
    attr_material: Option<String>,

    #[xml(child = "input")]
    inputs: Vec<InputShared>,
    #[xml(child = "p")]
    p: Vec<PrimitiveData>,
    #[xml(child = "extra")]
    extras: Vec<Extra>
}

#[derive(XmlRead, Debug)]
#[xml(tag = "bool_array")]
struct BoolArray {
    #[xml(attr = "count")]
    attr_count: u32,
    #[xml(attr = "id")]
    attr_id: Option<String>,
    #[xml(attr = "name")]
    attr_name: Option<String>,

    #[xml(text)]
    data: String,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "float_array")]
struct FloatArray {
    #[xml(attr = "count")]
    attr_count: u32,
    #[xml(attr = "id")]
    attr_id: Option<String>,
    #[xml(attr = "name")]
    attr_name: Option<String>,
    #[xml(attr = "digits")]
    attr_digits: Option<u8>,
    #[xml(attr = "magnitude")]
    attr_magnitude: Option<i16>,

    #[xml(text)]
    data: String,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "IDREF_array")]
struct IDREFArray {
    #[xml(attr = "count")]
    attr_count: u32,
    #[xml(attr = "id")]
    attr_id: Option<String>,
    #[xml(attr = "name")]
    attr_name: Option<String>,

    #[xml(text)]
    data: String,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "int_array")]
struct IntArray {
    #[xml(attr = "count")]
    attr_count: u32,
    #[xml(attr = "id")]
    attr_id: Option<String>,
    #[xml(attr = "name")]
    attr_name: Option<String>,
    #[xml(attr = "minInclusive")]
    attr_min_inclusive: Option<i32>,
    #[xml(attr = "maxInclusive")]
    attr_max_inclusive: Option<i32>,

    #[xml(text)]
    data: String,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "Name_array")]
struct NameArray {
    #[xml(attr = "count")]
    attr_count: u32,
    #[xml(attr = "id")]
    attr_id: Option<String>,
    #[xml(attr = "name")]
    attr_name: Option<String>,

    #[xml(text)]
    data: String,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "SIDREF_array")]
struct SIDREFArray {
    #[xml(attr = "count")]
    attr_count: u32,
    #[xml(attr = "id")]
    attr_id: Option<String>,
    #[xml(attr = "name")]
    attr_name: Option<String>,

    #[xml(text)]
    data: String,
}

// ??? No documentation
// struct TokenArray {
//     #[xml(attr = "count")]
//     attr_count: u32,
//     #[xml(attr = "id")]
//     attr_id: Option<String>,
//     #[xml(attr = "name")]
//     attr_name: Option<String>,

//     #[xml(text)]
//     data: String,
// }

#[derive(XmlRead, Debug)]
enum ArrayElement {
    #[xml(tag = "bool_array")]
    BoolArray(BoolArray),
    #[xml(tag = "float_array")]
    FloatArray(FloatArray),
    #[xml(tag = "IDREF_array")]
    IDREFArray(IDREFArray),
    #[xml(tag = "int_array")]
    IntArray(IntArray),
    #[xml(tag = "Name_array")]
    NameArray(NameArray),
    #[xml(tag = "SIDREF_array")]
    SIDREFArray(SIDREFArray),
    // NOTE: this is the only mention of <token_array> in the spec
    // no explanation or anything
    // #[xml(tag = "token_array")]
    // TokenArray(TokenArray),
}
impl ArrayElement {
    fn as_float_array(&self) -> Option<&FloatArray> {
        match self {
            ArrayElement::FloatArray(float_array) => Some(float_array),
            _ => None,
        }
    }
    // fn as_int_array(&self) -> _ {
    //     match self {
    //         ArrayElement::IntArray(int_array) => Some(int_array),
    //         _ => None,
    //     }
    // }
    // fn as_name_array(&self) -> _ {
    //     match self {
    //         ArrayElement::NameArray(name_array) => Some(name_array),
    //         _ => None,
    //     }
    // }
    // fn as_sidref_array(&self) -> _ {
    //     match self {
    //         ArrayElement::SIDREFArray(sidref_array) => Some(sidref_array),
    //         _ => None,
    //     }
    // }
    // fn as_bool_array(&self) -> _ {
    //     match self {
    //         ArrayElement::BoolArray(bool_array) => Some(bool_array),
    //         _ => None,
    //     }
    // }
    // fn as_idref_array(&self) -> _ {
    //     match self {
    //         ArrayElement::IDREFArray(idref_array) => Some(idref_array),
    //         _ => None,
    //     }
    // }
    // fn as_token_array(&self) -> _ {
    //     match self {
    //         ArrayElement::TokenArray(token_array) => Some(token_array),
    //         _ => None,
    //     }
    // }
}

#[derive(XmlRead, Debug)]
#[xml(tag = "source")]
struct SourceCore {
    #[xml(attr = "id")]
    attr_id: String,
    #[xml(attr = "name")]
    attr_name: Option<String>,

    #[xml(child = "asset")]
    asset: Option<Asset>,
    #[xml(
        child = "bool_array",
        child = "float_array",
        child = "IDREF_array",
        child = "int_array",
        child = "Name_array",
        child = "SIDREF_array",
        // child = "token_array",
    )]
    array_element: ArrayElement,
    // NOTE: conflicting information about whether this is required or not
    // TODO: when source (core) is under <mesh> or similar, it has Accessor and not Asset
    // under technique_common
    // #[xml(child = "technique_common")]
    // technique_common: Option<TechniqueCommon<Asset>>,
    #[xml(child = "technique")]
    techniques: Vec<Technique>,
}

#[derive(Debug)]
enum InputSemantic {
    Binormal,
    Color,
    Continuity,
    Image,
    Input,
    InTangent,
    Interpolation,
    INV_BIND_MATRIX,
    Joint,
    LinearSteps,
    MorphTarget,
    MorphWeight,
    Normal,
    Output,
    OutTangent,
    Position,
    Tangent,
    TexBinormal,
    Texcoord,
    TexTangent,
    UV,
    Vertex,
    Weight,
}

impl std::str::FromStr for InputSemantic {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "BINORMAL" => Ok(Self::Binormal),
            "COLOR" => Ok(Self::Color),
            "CONTINUITY" => Ok(Self::Continuity),
            "IMAGE" => Ok(Self::Image),
            "INPUT" => Ok(Self::Input),
            "IN_TANGENT" => Ok(Self::InTangent),
            "INTERPOLATION" => Ok(Self::Interpolation),
            "INV_BIND_MATRIX" => Ok(Self::INV_BIND_MATRIX),
            "JOINT" => Ok(Self::Joint),
            "LINEAR_STEPS" => Ok(Self::LinearSteps),
            "MORPH_TARGET" => Ok(Self::MorphTarget),
            "MORPH_WEIGHT" => Ok(Self::MorphWeight),
            "NORMAL" => Ok(Self::Normal),
            "OUTPUT" => Ok(Self::Output),
            "OUT_TANGENT" => Ok(Self::OutTangent),
            "POSITION" => Ok(Self::Position),
            "TANGENT" => Ok(Self::Tangent),
            "TEXBINORMAL" => Ok(Self::TexBinormal),
            "TEXCOORD" => Ok(Self::Texcoord),
            "TEXTANGENT" => Ok(Self::TexTangent),
            "UV" => Ok(Self::UV),
            "VERTEX" => Ok(Self::Vertex),
            "WEIGHT" => Ok(Self::Weight),
            _ => Err(format!("Invalid input semantic: {}", s)),
        }
    }
}

#[derive(XmlRead, Debug)]
#[xml(tag = "input")]
struct InputUnshared {
    #[xml(attr = "semantic")]
    attr_semantic: InputSemantic,
    #[xml(attr = "source")]
    attr_source: String,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "input")]
struct InputShared {
    #[xml(attr = "offset")]
    attr_offset: u32,
    #[xml(attr = "semantic")]
    attr_semantic: InputSemantic,
    #[xml(attr = "source")]
    attr_source: String,
    #[xml(attr = "set")]
    set: Option<u32>,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "vertices")]
struct Vertices {
    #[xml(attr = "id")]
    attr_id: String,
    #[xml(attr = "name")]
    attr_name: Option<String>,

    #[xml(child = "input")]
    inputs: Vec<InputUnshared>,
    #[xml(child = "extra")]
    extras: Vec<Extra>,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "convex_mesh")]
struct ConvexMesh {
    #[xml(attr = "convex_hull_of")]
    attr_convex_hull_of: Option<String>,

    #[xml(child = "source")]
    sources: Vec<SourceCore>,
    #[xml(child = "vertices")]
    vertices: Vertices,
    #[xml(
        child = "lines",
        child = "linestrips",
        child = "polygons",
        child = "polylist",
        child = "triangles",
        child = "trifans",
        child = "tristrips",
    )]
    primitives: Vec<Primitive>,
    #[xml(child = "extra")]
    extras: Vec<Extra>
}

#[derive(XmlRead, Debug)]
#[xml(tag = "mesh")]
struct Mesh {
    #[xml(child = "source")]
    sources: Vec<SourceCore>,
    #[xml(child = "vertices")]
    vertices: Vertices,
    #[xml(
        child = "lines",
        child = "linestrips",
        child = "polygons",
        child = "polylist",
        child = "triangles",
        child = "trifans",
        child = "tristrips",
    )]
    primitives: Vec<Primitive>,
    #[xml(child = "extra")]
    extras: Vec<Extra>
}

#[derive(XmlRead, Debug)]
#[xml(tag = "control_vertices")]
struct ControlVertices {
    #[xml(child = "input")]
    input: Vec<InputUnshared>,
    #[xml(child = "extra")]
    extras: Vec<Extra>,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "spline")]
struct Spline {
    #[xml(default, attr = "closed")]
    attr_closed: bool,

    #[xml(child = "source")]
    sources: Vec<SourceCore>,
    #[xml(child = "control_vertices")]
    control_vertices: ControlVertices,
    #[xml(child = "extra")]
    extras: Vec<Extra>,
}

#[derive(XmlRead, Debug)]
enum CurveElement {
    #[xml(tag = "line")]
    Line(Line),
    #[xml(tag = "circle")]
    Circle(Circle),
    #[xml(tag = "ellipse")]
    Ellipse(Ellipse),
    #[xml(tag = "parabola")]
    Parabola(Parabola),
    #[xml(tag = "hyperbola")]
    Hyperbola(Hyperbola),
    #[xml(tag = "nurbs")]
    Nurbs(Nurbs),
}

#[derive(XmlRead, Debug)]
#[xml(tag = "line")]
struct Line {
    #[xml(flatten_text = "origin")]
    origin: String, // 3 float values
    #[xml(flatten_text = "direction")]
    direction: String, // 3 float values
    #[xml(child = "extra")]
    extras: Vec<Extra>,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "circle")]
struct Circle {
    #[xml(flatten_text = "radius")]
    radius: f32,
    #[xml(child = "extra")]
    extras: Vec<Extra>
}

#[derive(XmlRead, Debug)]
#[xml(tag = "ellipse")]
struct Ellipse {
    #[xml(flatten_text = "radius")]
    radius: String, // 2 float values
    #[xml(child = "extra")]
    extras: Vec<Extra>
}

#[derive(XmlRead, Debug)]
#[xml(tag = "parabola")]
struct Parabola {
    #[xml(flatten_text = "focal")]
    focal: f32,
    #[xml(child = "extra")]
    extras: Vec<Extra>,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "hyperbola")]
struct Hyperbola {
    #[xml(flatten_text = "radius")]
    radius: String, // 2 float values
    #[xml(child = "extra")]
    extras: Vec<Extra>,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "nurbs")]
struct Nurbs {
    #[xml(attr = "degree")]
    attr_degree: u32,
    #[xml(default, attr = "closed")]
    attr_closed: bool,

    #[xml(child = "source")]
    sources: Vec<SourceCore>,
    #[xml(child = "control_vertices")]
    control_vertices: ControlVertices,
    #[xml(child = "extra")]
    extras: Vec<Extra>,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "origin")]
struct Origin {
    #[xml(text)]
    value: String,
}

impl Default for Origin {
    fn default() -> Self {
        Self {
            value: "0 0 0".to_string(),
        }
    }
}

#[derive(XmlRead, Debug)]
#[xml(tag = "orient")]
struct Orient {
    #[xml(text)]
    value: String,
}
 
#[derive(XmlRead, Debug)]
#[xml(tag = "curve")]
struct Curve {
    #[xml(attr = "id")]
    attr_sid: Option<String>,
    #[xml(attr = "name")]
    attr_name: Option<String>,

    #[xml(
        child = "line",
        child = "circle",
        child = "ellipse",
        child = "parabola",
        child = "hyperbola",
        child = "nurbs",
    )]
    curve_element: CurveElement,
    #[xml(child = "orient")]
    orient: Vec<Orient>,
    #[xml(default, child = "origin")]
    origin: Origin,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "curves")]
struct Curves {
    #[xml(child = "curve")]
    curves: Vec<Curve>,
    #[xml(child = "extra")]
    extras: Vec<Extra>,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "surface_curves")]
struct SurfaceCurves {
    #[xml(child = "curve")]
    curves: Vec<Curve>,
    #[xml(child = "extra")]
    extras: Vec<Extra>,
}

#[derive(XmlRead, Debug)]
enum SurfaceElement {
    #[xml(tag = "cone")]
    Cone(Cone),
    #[xml(tag = "plane")]
    Plane(Plane),
    #[xml(tag = "cylinder")]
    Cylinder(Cylinder),
    #[xml(tag = "nurbs_surface")]
    NurbsSurface(NurbsSurface),
    #[xml(tag = "sphere")]
    Sphere(Sphere),
    #[xml(tag = "torus")]
    Torus(Torus),
    #[xml(tag = "swept_surface")]
    SweptSurface(SweptSurface),
}

#[derive(XmlRead, Debug)]
#[xml(tag = "cone")]
struct Cone {
    #[xml(flatten_text = "radius")]
    radius: f32,
    #[xml(flatten_text = "angle")]
    angle: f32,
    #[xml(child = "extra")]
    extras: Vec<Extra>,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "plane")]
struct Plane {
    #[xml(flatten_text = "equation")]
    equation: String,
    #[xml(child = "extra")]
    extras: Vec<Extra>,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "cylinder")]
struct Cylinder {
    #[xml(flatten_text = "height")]
    height: f32,
    #[xml(flatten_text = "radius")]
    radius: String,
    #[xml(child = "extra")]
    extras: Vec<Extra>,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "nurbs_surface")]
struct NurbsSurface {
    #[xml(attr = "degree_u")]
    attr_degree_u: u32,
    #[xml(default, attr = "closed_u")]
    attr_closed_u: bool,
    #[xml(attr = "degree_v")]
    attr_degree_v: u32,
    #[xml(default, attr = "closed_v")]
    attr_closed_v: bool,

    #[xml(child = "source")]
    sources: Vec<SourceCore>,
    #[xml(child = "control_vertices")]
    control_vertices: ControlVertices,
    #[xml(child = "extra")]
    extras: Vec<Extra>,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "sphere")]
struct Sphere {
    #[xml(flatten_text = "radius")]
    radius: f32,
    #[xml(child = "extra")]
    extras: Vec<Extra>,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "torus")]
struct Torus {
    #[xml(flatten_text = "radius")]
    radius: String, // 2 float values
    #[xml(child = "extra")]
    extras: Vec<Extra>,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "swept_surface")]
struct SweptSurface {
    #[xml(child = "curve")]
    curve: Curve,
    #[xml(flatten_text = "direction")]
    direction: String, // 3 float values
    #[xml(flatten_text = "origin")]
    origin: String, // 3 float values
    #[xml(flatten_text = "axis")]
    axis: String, // 3 float values
}

#[derive(XmlRead, Debug)]
#[xml(tag = "surface")]
struct Surface {
    #[xml(attr = "sid")]
    attr_sid: Option<String>,
    #[xml(attr = "name")]
    attr_name: Option<String>,

    #[xml(
        child = "cone",
        child = "plane",
        child = "cylinder",
        child = "nurbs_surface",
        child = "sphere",
        child = "torus",
        child = "swept_surface",
    )]
    surface_element: SurfaceElement,
    #[xml(child = "orient")]
    orient: Vec<Orient>,
    #[xml(default, child = "origin")]
    origin: Origin,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "surfaces")]
struct Surfaces {
    #[xml(child = "surface")]
    surfaces: Vec<Surface>,
    #[xml(child = "extra")]
    extras: Vec<Extra>,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "edges")]
struct Edges {
    #[xml(attr = "id")]
    attr_id: String,
    #[xml(attr = "name")]
    attr_name: Option<String>,
    #[xml(attr = "count")]
    attr_count: u32,

    #[xml(child = "input")]
    inputs: Vec<InputShared>,
    #[xml(child = "p")]
    p: Option<PrimitiveData>,
    #[xml(child = "extra")]
    extras: Vec<Extra>,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "wires")]
struct Wires {
    #[xml(attr = "id")]
    attr_id: String,
    #[xml(attr = "name")]
    attr_name: Option<String>,
    #[xml(attr = "count")]
    attr_count: u32,

    #[xml(child = "input")]
    inputs: Vec<InputShared>,
    #[xml(child = "vcount")]
    vcount: VCount,
    #[xml(child = "p")]
    p: Option<PrimitiveData>,
    #[xml(child = "extra")]
    extras: Vec<Extra>,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "faces")]
struct Faces {
    #[xml(attr = "id")]
    attr_id: String,
    #[xml(attr = "name")]
    attr_name: Option<String>,
    #[xml(attr = "count")]
    attr_count: u32,

    #[xml(child = "input")]
    inputs: Vec<InputShared>,
    #[xml(child = "vcount")]
    vcount: VCount,
    #[xml(child = "p")]
    p: Option<PrimitiveData>,
    #[xml(child = "extra")]
    extras: Vec<Extra>,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "pcurves")]
struct PCurves {
    #[xml(attr = "id")]
    attr_id: String,
    #[xml(attr = "name")]
    attr_name: Option<String>,
    #[xml(attr = "count")]
    attr_count: u32,

    #[xml(child = "input")]
    inputs: Vec<InputShared>,
    #[xml(child = "vcount")]
    vcount: VCount,
    #[xml(child = "p")]
    p: Option<PrimitiveData>,
    #[xml(child = "extra")]
    extras: Vec<Extra>,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "shells")]
struct Shells {
    #[xml(attr = "id")]
    attr_id: String,
    #[xml(attr = "name")]
    attr_name: Option<String>,
    #[xml(attr = "count")]
    attr_count: u32,

    #[xml(child = "input")]
    inputs: Vec<InputShared>,
    #[xml(child = "vcount")]
    vcount: VCount,
    #[xml(child = "p")]
    p: Option<PrimitiveData>,
    #[xml(child = "extra")]
    extras: Vec<Extra>,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "solids")]
struct Solids {
    #[xml(attr = "id")]
    attr_id: String,
    #[xml(attr = "name")]
    attr_name: Option<String>,
    #[xml(attr = "count")]
    attr_count: u32,

    #[xml(child = "input")]
    inputs: Vec<InputShared>,
    #[xml(child = "vcount")]
    vcount: VCount,
    #[xml(child = "p")]
    p: Option<PrimitiveData>,
    #[xml(child = "extra")]
    extras: Vec<Extra>,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "brep")]
struct Brep {
    #[xml(child = "curves")]
    curves: Option<Curves>,
    #[xml(child = "surface_curves")]
    surface_curves: Option<SurfaceCurves>,
    #[xml(child = "surfaces")]
    surfaces: Option<Surfaces>,
    #[xml(child = "source")]
    source: Vec<SourceCore>,
    #[xml(child = "vertices")]
    vertices: Vertices,
    #[xml(child = "edges")]
    edges: Option<Edges>,
    #[xml(child = "wires")]
    wires: Option<Wires>,
    #[xml(child = "faces")]
    faces: Option<Faces>,
    #[xml(child = "pcurves")]
    pcurves: Option<PCurves>,
    #[xml(child = "shells")]
    shells: Option<Shells>,
    #[xml(child = "solids")]
    solids: Option<Solids>,
    #[xml(child = "extra")]
    extras: Vec<Extra>,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "geometry")]
struct Geometry {
    #[xml(attr = "id")]
    attr_id: Option<String>,
    #[xml(attr = "name")]
    attr_name: Option<String>,

    #[xml(child = "asset")]
    asset: Option<Asset>,
    #[xml(
        child = "convex_mesh",
        child = "mesh",
        child = "spline",
        child = "brep"
    )]
    geometric_element: GeometricElement,
    #[xml(child = "extra")]
    extras: Vec<Extra>,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "library_animation_clips")]
struct LibraryAnimationClips {
    // TODO:
}

#[derive(XmlRead, Debug)]
#[xml(tag = "library_animations")]
struct LibraryAnimations {
    // TODO:
}

#[derive(XmlRead, Debug)]
#[xml(tag = "library_articulated_systems")]
struct LibraryArticulatedSystems {
    // TODO:
}

#[derive(XmlRead, Debug)]
#[xml(tag = "library_cameras")]
struct LibraryCameras {
    // TODO:
}

#[derive(XmlRead, Debug)]
#[xml(tag = "library_controllers")]
struct LibraryControllers {
    // TODO:
}

#[derive(XmlRead, Debug)]
#[xml(tag = "library_effects")]
struct LibraryEffects {
    #[xml(attr = "id")]
    attr_id: Option<String>,
    #[xml(attr = "name")]
    attr_name: Option<String>,

    #[xml(child = "asset")]
    asset: Option<Asset>,
    #[xml(child = "effect")]
    effects: Vec<Effect>,
    #[xml(child = "extra")]
    extras: Vec<Extra>,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "library_force_fields")]
struct LibraryForceFields {
    // TODO:
}

#[derive(XmlRead, Debug)]
#[xml(tag = "library_formulas")]
struct LibraryFormulas {
    // TODO:
}

#[derive(XmlRead, Debug)]
#[xml(tag = "library_geometries")]
struct LibraryGeometries {
    #[xml(attr = "id")]
    attr_id: Option<String>,
    #[xml(attr = "name")]
    attr_name: Option<String>,

    #[xml(child = "asset")]
    asset: Option<Asset>,
    #[xml(child = "geometry")]
    geometries: Vec<Geometry>,
    #[xml(child = "extra")]
    extras: Vec<Extra>,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "library_images")]
struct LibraryImages {
    #[xml(attr = "id")]
    attr_id: Option<String>,
    #[xml(attr = "name")]
    attr_name: Option<String>,

    #[xml(child = "asset")]
    asset: Option<Asset>,
    #[xml(child = "image")]
    images: Vec<Image>,
    #[xml(child = "extra")]
    extras: Vec<Extra>,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "renderable")]
struct Renderable {
    #[xml(attr = "share")]
    attr_share: bool,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "init_from")]
struct InitFrom {
    // TODO:
}
#[derive(XmlRead, Debug)]
#[xml(tag = "create_2d")]
struct Create2D {
    // TODO:
}
#[derive(XmlRead, Debug)]
#[xml(tag = "create_3d")]
struct Create3D {
    // TODO:
}
#[derive(XmlRead, Debug)]
#[xml(tag = "create_cube")]
struct CreateCube {
    // TODO:
}

#[derive(XmlRead, Debug)]
#[xml(tag = "image")]
struct Image {
    #[xml(attr = "id")]
    attr_id: Option<String>,
    #[xml(attr = "sid")]
    attr_sid: Option<String>,
    #[xml(attr = "name")]
    attr_name: Option<String>,

    #[xml(child = "asset")]
    asset: Option<Asset>,
    #[xml(child = "renderable")]
    renderable: Option<Renderable>,
    #[xml(child = "init_from")]
    init_from: Option<InitFrom>,
    #[xml(child = "create_2d")]
    create_2d: Option<Create2D>,
    #[xml(child = "create_3d")]
    create_3d: Option<Create3D>,
    #[xml(child = "create_cube")]
    create_cube: Option<CreateCube>,
    #[xml(child = "extra")]
    extras: Vec<Extra>,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "library_joints")]
struct LibraryJoints {
    // TODO:
}

#[derive(XmlRead, Debug)]
#[xml(tag = "library_kinematics_models")]
struct LibraryKinematicsModels {
    // TODO:
}

#[derive(XmlRead, Debug)]
#[xml(tag = "library_kinematics_scenes")]
struct LibraryKinematicsScenes {
    // TODO:
}

#[derive(XmlRead, Debug)]
#[xml(tag = "library_lights")]
struct LibraryLights {
    // TODO:
}

#[derive(XmlRead, Debug)]
#[xml(tag = "library_materials")]
struct LibraryMaterials {
    #[xml(attr = "id")]
    attr_id: Option<String>,
    #[xml(attr = "name")]
    attr_name: Option<String>,

    #[xml(child = "asset")]
    asset: Option<Asset>,
    #[xml(child = "material")]
    materials: Vec<Material>,
    #[xml(child = "extra")]
    extras: Vec<Extra>,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "library_nodes")]
struct LibraryNodes {
    // TODO:
}

#[derive(XmlRead, Debug)]
#[xml(tag = "library_physics_materials")]
struct LibraryPhysicsMaterials {
    // TODO:
}

#[derive(XmlRead, Debug)]
#[xml(tag = "library_physics_models")]
struct LibraryPhysicsModels {
    // TODO:
}

#[derive(XmlRead, Debug)]
#[xml(tag = "library_physics_scenes")]
struct LibraryPhysicsScenes {
    // TODO:
}

#[derive(Default, Debug, PartialEq)]
enum NodeType {
    #[default]
    Node,
    Joint,
}

impl std::str::FromStr for NodeType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "NODE" => Ok(NodeType::Node),
            "JOINT" => Ok(NodeType::Joint),
            _ => Err(format!("Unknown node type: {}", s)),
        }
    }
}

#[derive(XmlRead, Debug)]
enum TransformationElement {
    #[xml(tag = "lookat")]
    LookAt(LookAt),
    #[xml(tag = "matrix")]
    Matrix(Matrix),
    #[xml(tag = "rotate")]
    Rotate(Rotate),
    #[xml(tag = "scale")]
    Scale(Scale),
    #[xml(tag = "skew")]
    Skew(Skew),
    #[xml(tag = "translate")]
    Translate(Translate),
}

#[derive(XmlRead, Debug)]
#[xml(tag = "lookat")]
struct LookAt {
    #[xml(attr = "sid")]
    attr_sid: Option<String>,

    #[xml(text)]
    value: String, // 3x3 float matrix
}

#[derive(XmlRead, Debug)]
#[xml(tag = "matrix")]
struct Matrix {
    #[xml(attr = "sid")]
    attr_sid: Option<String>,

    #[xml(text)]
    value: String, // 4x4 float matrix
}

#[derive(XmlRead, Debug)]
#[xml(tag = "rotate")]
struct Rotate {
    #[xml(attr = "sid")]
    attr_sid: Option<String>,

    #[xml(text)]
    value: String, // float vec3 representing axis of rotation + angle
}

#[derive(XmlRead, Debug)]
#[xml(tag = "scale")]
struct Scale {
    #[xml(attr = "sid")]
    attr_sid: Option<String>,

    #[xml(text)]
    value: String, // vec3 of floats
}

#[derive(XmlRead, Debug)]
#[xml(tag = "skew")]
struct Skew {
    #[xml(attr = "sid")]
    attr_sid: Option<String>,

    #[xml(text)]
    value: String, // 7 floats, 1 for angle, 3 for rotation, 3 for translation
}

#[derive(XmlRead, Debug)]
#[xml(tag = "translate")]
struct Translate {
    #[xml(attr = "sid")]
    attr_sid: Option<String>,

    #[xml(text)]
    value: String, // vec3 of floats (x, y, z)
}

#[derive(XmlRead, Debug)]
#[xml(tag = "node")]
struct Node {
    #[xml(attr = "id")]
    attr_id: Option<String>,
    #[xml(attr = "name")]
    attr_name: Option<String>,
    #[xml(attr = "sid")]
    attr_sid: Option<String>,
    #[xml(default, attr = "type")]
    attr_type: NodeType,
    #[xml(attr = "layer")]
    attr_layer: Option<String>,

    #[xml(child = "asset")]
    asset: Option<Asset>,
    #[xml(
        child = "lookat",
        child = "matrix",
        child = "rotate",
        child = "scale",
        child = "skew",
        child = "translate",
    )]
    transformations: Vec<TransformationElement>,
    #[xml(child = "instance_camera")]
    instance_camera: Vec<InstanceCamera>,
    #[xml(child = "instance_controller")]
    instance_controller: Vec<InstanceController>,
    #[xml(child = "instance_geometry")]
    instance_geometry: Vec<InstanceGeometry>,
    #[xml(child = "instance_light")]
    instance_light: Vec<InstanceLight>,
    #[xml(child = "instance_node")]
    instance_node: Vec<InstanceNode>,
    #[xml(child = "node")]
    children: Vec<Node>,
    #[xml(child = "extra")]
    extras: Vec<Extra>,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "instance_camera")]
struct InstanceCamera {
    #[xml(attr = "sid")]
    attr_sid: Option<String>,
    #[xml(attr = "name")]
    attr_name: Option<String>,
    #[xml(attr = "url")]
    attr_url: String,

    #[xml(child = "extra")]
    extras: Vec<Extra>,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "instance_controller")]
struct InstanceController {
    // TODO:
}

#[derive(XmlRead, Debug)]
#[xml(tag = "bind_material")]
struct BindMaterial {
    // TODO:
    // #[xml(child = "param")]
    // param: Vec<ParamCore>,
    #[xml(child = "technique_common")]
    technique_common: TechniqueCommon<InstanceMaterialGeometry>,
    #[xml(child = "technique")]
    techniques: Vec<Technique>,
    #[xml(child = "extra")]
    extras: Vec<Extra>,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "instance_geometry")]
struct InstanceGeometry {
    #[xml(attr = "sid")]
    attr_sid: Option<String>,
    #[xml(attr = "name")]
    attr_name: Option<String>,
    #[xml(attr = "url")]
    attr_url: String,

    #[xml(child = "bind_material")]
    bind_material: Option<BindMaterial>,
    #[xml(child = "extra")]
    extras: Vec<Extra>,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "instance_light")]
struct InstanceLight {
    #[xml(attr = "sid")]
    attr_sid: Option<String>,
    #[xml(attr = "name")]
    attr_name: Option<String>,
    #[xml(attr = "url")]
    attr_url: String,

    #[xml(child = "extra")]
    extras: Vec<Extra>,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "instance_node")]
struct InstanceNode {
    #[xml(attr = "sid")]
    attr_sid: Option<String>,
    #[xml(attr = "name")]
    attr_name: Option<String>,
    #[xml(attr = "url")]
    attr_url: String,
    #[xml(attr = "proxy")]
    attr_proxy: Option<String>,

    #[xml(child = "extra")]
    extras: Vec<Extra>,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "technique_override")]
struct TechniqueOverride {
    #[xml(attr = "ref")]
    attr_ref: String,
    #[xml(attr = "pass")]
    attr_pass: Option<String>
}

#[derive(XmlRead, Debug)]
#[xml(tag = "bind")]
struct Bind {
    #[xml(attr = "semantic")]
    attr_semantic: String,
    #[xml(attr = "target")]
    attr_target: String,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "instance_material")]
struct InstanceMaterialRendering {
    #[xml(attr = "url")]
    attr_url: String,

    #[xml(child = "technique_override")]
    technique_override: Option<TechniqueOverride>,
    #[xml(child = "bind")]
    bind: Vec<Bind>,
    #[xml(child = "extra")]
    extras: Vec<Extra>,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "instance_material")]
struct InstanceMaterialGeometry {
    #[xml(attr = "sid")]
    attr_sid: Option<String>,
    #[xml(attr = "name")]
    attr_name: Option<String>,
    #[xml(attr = "target")]
    attr_target: String,
    #[xml(attr = "symbol")]
    attr_symbol: String,

    // TODO:
    // #[xml(child = "bind")]
    // bind: Vec<Bind>,
    // #[xml(child = "bind_vertex_input")]
    // bind_vertex_input: Vec<BindVertexInput>,
    #[xml(child = "extra")]
    extras: Vec<Extra>,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "render")]
struct Render {
    #[xml(attr = "name")]
    attr_name: Option<String>,
    #[xml(attr = "id")]
    attr_id: Option<String>,
    #[xml(attr = "camera_node")]
    attr_camera_node: Option<String>,

    #[xml(flatten_text = "layer")]
    layers: String,
    #[xml(child = "instance_material")]
    instance_material: Option<InstanceMaterialRendering>,
    #[xml(child = "extra")]
    extras: Vec<Extra>,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "evaluate_scene")]
struct EvaluateScene {
    #[xml(attr = "id")]
    attr_id: Option<String>,
    #[xml(attr = "name")]
    attr_name: Option<String>,
    #[xml(attr = "sid")]
    attr_sid: Option<String>,
    #[xml(attr = "enable")]
    attr_enable: Option<bool>, // default is true

    #[xml(child = "asset")]
    asset: Option<Asset>,
    #[xml(child = "render")]
    renders: Vec<Render>,
    #[xml(child = "extra")]
    extras: Vec<Extra>,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "visual_scene")]
struct VisualScene {
    #[xml(attr = "id")]
    attr_id: Option<String>,
    #[xml(attr = "name")]
    attr_name: Option<String>,

    #[xml(child = "asset")]
    asset: Option<Asset>,
    #[xml(child = "node")]
    nodes: Vec<Node>,
    #[xml(child = "evaluate_scene")]
    evaluate_scene: Vec<EvaluateScene>,
    #[xml(child = "extra")]
    extras: Option<Extra>,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "library_visual_scenes")]
struct LibraryVisualScenes {
    #[xml(attr = "id")]
    attr_id: Option<String>,
    #[xml(attr = "name")]
    attr_name: Option<String>,

    #[xml(child = "asset")]
    asset: Option<Asset>,
    #[xml(child = "visual_scene")]
    visual_scenes: Vec<VisualScene>,
    #[xml(child = "extra")]
    extras: Vec<Extra>,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "COLLADA")]
// #[xml(strict(unknown_attribute, unknown_element))]
struct ColladaDocument {
    #[xml(attr = "version")]
    attr_version: String,

    #[xml(child = "asset")]
    asset: Asset,
    #[xml(child = "scene")]
    scene: Option<Scene>,
    #[xml(
        child = "library_animation_clips",
        child = "library_animations",
        child = "library_articulated_systems",
        child = "library_cameras",
        child = "library_controllers",
        child = "library_effects",
        child = "library_force_fields",
        child = "library_formulas",
        child = "library_geometries",
        child = "library_images",
        child = "library_joints",
        child = "library_kinematics_models",
        child = "library_kinematics_scenes",
        child = "library_lights",
        child = "library_materials",
        child = "library_nodes",
        child = "library_physics_materials",
        child = "library_physics_models",
        child = "library_physics_scenes",
        child = "library_visual_scenes",
    )]
    libraries: Vec<Library>,
    #[xml(child = "extra")]
    extras: Vec<Extra>,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "scene")]
struct Scene {
    #[xml(child = "instance_physics_scene")]
    instance_physics_scene: Vec<InstancePhysicsScene>,
    #[xml(child = "instance_visual_scene")]
    instance_visual_scene: Option<InstanceVisualScene>,
    #[xml(child = "instance_kinematics_scene")]
    instance_kinematics_scene: Option<InstanceKinematicsScene>,
    #[xml(child = "extra")]
    extras: Vec<Extra>,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "instance_physics_scene")]
struct InstancePhysicsScene {
    #[xml(attr = "sid")]
    attr_sid: Option<String>,
    #[xml(attr = "name")]
    attr_name: Option<String>,
    #[xml(attr = "url")]
    attr_url: String,

    #[xml(child = "extra")]
    extras: Vec<Extra>,
}
#[derive(XmlRead, Debug)]
#[xml(tag = "instance_visual_scene")]
struct InstanceVisualScene {
    #[xml(attr = "sid")]
    attr_sid: Option<String>,
    #[xml(attr = "name")]
    attr_name: Option<String>,
    #[xml(attr = "url")]
    attr_url: String,

    #[xml(child = "extra")]
    extras: Vec<Extra>,
}
#[derive(XmlRead, Debug)]
#[xml(tag = "instance_kinematics_scene")]
struct InstanceKinematicsScene {
    #[xml(attr = "sid")]
    attr_sid: Option<String>,
    #[xml(attr = "name")]
    attr_name: Option<String>,
    #[xml(attr = "url")]
    attr_url: String,

    #[xml(child = "asset")]
    asset: Option<Asset>,
    #[xml(child = "newparam")]
    new_param: Vec<NewParam>,
    #[xml(child = "setparam")]
    set_param: Vec<SetParam>,
    #[xml(child = "bind_kinematics_model")]
    bind_kinematics_model: Vec<BindKinematicsModel>,
    #[xml(child = "bind_joint_axis")]
    bind_joint_axis: Vec<BindJointAxis>,
    #[xml(child = "extra")]
    extras: Vec<Extra>,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "bind_kinematics_model")]
struct BindKinematicsModel {
    #[xml(attr = "node")]
    attr_node: Option<String>,

    // TODO:
    // param: Param,
    // sidref: SidRef,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "bind_joint_axis")]
struct BindJointAxis {
    #[xml(attr = "target")]
    attr_target: Option<String>,

    // TODO: children
}

#[derive(XmlRead, Debug)]
#[xml(tag = "contributor")]
struct Contributor {
    #[xml(flatten_text = "author")]
    author: Option<String>,
    #[xml(flatten_text = "author_email")]
    author_email: Option<String>,
    #[xml(flatten_text = "author_website")]
    author_website: Option<String>,
    #[xml(flatten_text = "authoring_tool")]
    authoring_tool: Option<String>,
    #[xml(flatten_text = "comments")]
    comments: Option<String>,
    #[xml(flatten_text = "copyright")]
    copyright: Option<String>,
    #[xml(flatten_text = "source_data")]
    source_data: Option<String>,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "altitude")]
struct Altitude {
    #[xml(default, attr = "mode")]
    attr_mode: AltitudeMode,
    #[xml(text)]
    value: f32,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "geographic_location")]
struct GeographicLocation {
    #[xml(flatten_text = "latitude")]
    latitude: f32,
    #[xml(flatten_text = "longitude")]
    longitude: f32,
    #[xml(child = "altitude")]
    altitude: Altitude,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "coverage")]
struct Coverage {
    #[xml(child = "geographic_location")]
    geographic_location: Option<GeographicLocation>,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "unit")]
struct Unit {
    #[xml(default, attr = "name")]
    attr_name: String,
    #[xml(default, attr = "meter")]
    attr_meter: f32,
}

impl Default for Unit {
    fn default() -> Self {
        Unit {
            attr_name: "meter".to_string(),
            attr_meter: 1.0,
        }
    }
}

#[derive(XmlRead, Debug)]
#[xml(tag = "technique_common")]
struct TechniqueCommon<T> where T: for<'a> XmlRead<'a> {
    #[xml(
        child = "asset", // for <source>
        child = "accessor", // for <source>
        child = "instance_material", // for <bind_material>
        // TODO: other possible tags
    )]
    data: T,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "technique")]
struct Technique {
    #[xml(attr = "profile")]
    attr_profile: String,

    // TODO: can't have this as text
    // #[xml(text)]
    // data: String, // Can be any legal xml. hard to type this, will probably just dump all the xml
                  // into a string
}

#[derive(XmlRead, Debug)]
#[xml(tag = "extra")]
struct Extra {
    #[xml(attr = "id")]
    attr_id: Option<String>,
    #[xml(attr = "name")]
    attr_name: Option<String>,
    #[xml(attr = "type")]
    attr_type: Option<String>,

    #[xml(child = "asset")]
    asset: Option<Asset>,
    #[xml(child = "technique")]
    techniques: Vec<Technique>,
}

#[derive(XmlRead, Debug)]
#[xml(tag = "asset")]
struct Asset {
    #[xml(child = "contributor")]
    contributors: Vec<Contributor>,
    #[xml(child = "coverage")]
    coverage: Option<Coverage>,
    #[xml(flatten_text = "created")]
    created: String,
    #[xml(flatten_text = "keywords")]
    keywords: Option<String>,
    #[xml(flatten_text = "modified")]
    modified: String,
    #[xml(flatten_text = "revision")]
    revision: Option<String>,
    #[xml(flatten_text = "subject")]
    subject: Option<String>,
    #[xml(flatten_text = "title")]
    title: Option<String>,
    #[xml(default, child = "unit")]
    unit: Unit,
    #[xml(default, flatten_text = "up_axis")]
    up_axis: UpAxis, 
    #[xml(child = "extra")]
    extras: Vec<Extra>,
}

fn parse_vertices(
    inputs: &[InputUnshared],
    sources: &HashMap<String, &ArrayElement>,
    min_aabb: &mut glm::Vec3,
    max_aabb: &mut glm::Vec3,
) -> (Vec<glm::Vec3>, Vec<glm::Vec3>, Vec<glm::Vec2>, i32, i32) {
    let mut normal_offset = -1;
    let mut texcoord_offset = -1;

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut tex_coords = Vec::new();

    for input in inputs {
        match input.attr_semantic {
            InputSemantic::Position => {
                positions = sources
                    .get(&input.attr_source[1..])
                    .unwrap()
                    .as_float_array()
                    .unwrap()
                    .data
                    .trim()
                    .split_ascii_whitespace()
                    .collect::<Vec<_>>()
                    .chunks_exact(3)
                    .map(|v| {
                        let vertex = glm::vec3(v[0].parse::<f32>().unwrap(), v[1].parse::<f32>().unwrap(), v[2].parse::<f32>().unwrap());

                        *min_aabb = glm::vec3(
                            min_aabb.x.min(vertex.x),
                            min_aabb.y.min(vertex.y),
                            min_aabb.z.min(vertex.z),
                        );

                        *max_aabb = glm::vec3(
                            max_aabb.x.max(vertex.x),
                            max_aabb.y.max(vertex.y),
                            max_aabb.z.max(vertex.z),
                        );

                        vertex
                    })
                .collect::<Vec<_>>();
                }
            InputSemantic::Normal => {
                normal_offset = 0;

                normals = sources
                    .get(&input.attr_source[1..])
                    .unwrap()
                    .as_float_array()
                    .unwrap()
                    .data
                    .trim()
                    .split_ascii_whitespace()
                    .collect::<Vec<_>>()
                    .chunks_exact(3)
                    .map(|v| glm::vec3(v[0].parse::<f32>().unwrap(), v[1].parse::<f32>().unwrap(), v[2].parse::<f32>().unwrap()))
                    .collect::<Vec<_>>();
                }
            InputSemantic::Texcoord => {
                texcoord_offset = 0;

                tex_coords = sources
                    .get(&input.attr_source[1..])
                    .unwrap()
                    .as_float_array()
                    .unwrap()
                    .data
                    .trim()
                    .split_ascii_whitespace()
                    .collect::<Vec<_>>()
                    .chunks_exact(2)
                    .map(|v| glm::vec2(v[0].parse::<f32>().unwrap(), v[1].parse::<f32>().unwrap()))
                    .collect::<Vec<_>>();
                }
            _ => {},
        }
    }

    (positions, normals, tex_coords, normal_offset, texcoord_offset)
}

fn parse_triangles(
    node_name: String,
    triangles: &Triangles,
    positions: &[glm::Vec3],
    mut normal_offset: i32,
    mut normals: Vec<glm::Vec3>,
    mut texcoord_offset: i32,
    mut tex_coords: Vec<glm::Vec2>,
    sources: &HashMap<String, &ArrayElement>,
) -> ObjMesh {
    const VCOUNT: usize = 3;

    let mut vertices: Vec<Vertex> = Vec::with_capacity(triangles.attr_count as usize);
    let mut indices: Vec<u32> = Vec::with_capacity(triangles.attr_count as usize * 3);

    // TODO: negative indices
    let p = triangles.p.as_ref().unwrap().value
        .split_ascii_whitespace()
        .map(|s| s.parse::<u32>().unwrap())
        .collect::<Vec<u32>>();

    let mut max_offset = 1;

    for input in &triangles.inputs {
        match input.attr_semantic {
            InputSemantic::Normal => {
                normal_offset = input.attr_offset as i32;
                max_offset = max_offset.max(normal_offset + 1);

                normals = sources
                    .get(&input.attr_source[1..])
                    .unwrap()
                    .as_float_array()
                    .unwrap()
                    .data
                    .trim()
                    .split_ascii_whitespace()
                    .collect::<Vec<_>>()
                    .chunks_exact(3)
                    .map(|v| glm::vec3(v[0].parse::<f32>().unwrap(), v[1].parse::<f32>().unwrap(), v[2].parse::<f32>().unwrap()))
                    .collect::<Vec<_>>();
                },
            InputSemantic::Texcoord => {
                texcoord_offset = input.attr_offset as i32;
                max_offset = max_offset.max(texcoord_offset + 1);

                tex_coords = sources
                    .get(&input.attr_source[1..])
                    .unwrap()
                    .as_float_array()
                    .unwrap()
                    .data
                    .trim()
                    .split_ascii_whitespace()
                    .collect::<Vec<_>>()
                    .chunks_exact(2)
                    .map(|v| glm::vec2(v[0].parse::<f32>().unwrap(), v[1].parse::<f32>().unwrap()))
                    .collect::<Vec<_>>();
            }
            _ => {}, // ignore others
        }
    }

    for (i, triangle) in p.chunks_exact(VCOUNT * max_offset as usize).enumerate() {
        let norm = {
            if normal_offset != -1 {
                let idx = triangle[normal_offset as usize];
                normals[idx as usize]
            } else {
                // TODO: generate normals for this face
                glm::vec3(0.0, 0.0, 0.0)
            }
        };
        let texcoords = {
            if texcoord_offset != -1 {
                let idx = triangle[texcoord_offset as usize];
                tex_coords[idx as usize]
            } else {
                glm::vec2(0.0, 0.0)
            }
        };

        for i in 0..VCOUNT {
            // position offset is assumed to be 0
            let pos_idx = triangle[i * max_offset as usize];
            let pos = positions[pos_idx as usize];

            let vertex = Vertex {
                position: pos,
                normal: norm,
                tex_coords: texcoords,
            };

            vertices.push(vertex);
        }

        indices.push((i * VCOUNT) as u32);
        indices.push((i * VCOUNT) as u32 + 1);
        indices.push((i * VCOUNT) as u32 + 2);
    }

    // TODO: materials
    ObjMesh {
        name: node_name,
        vertices,
        indices,
        material: None,
    }
}

fn parse_polylist(
    node_name: String,
    polylist: &PolyList,
    positions: &[glm::Vec3],
    mut normal_offset: i32,
    mut normals: Vec<glm::Vec3>,
    mut texcoord_offset: i32,
    mut tex_coords: Vec<glm::Vec2>,
    sources: &HashMap<String, &ArrayElement>,
) -> ObjMesh {
    let mut vertices: Vec<Vertex> = Vec::with_capacity(polylist.attr_count as usize);
    let mut indices: Vec<u32> = Vec::with_capacity(polylist.attr_count as usize * 3);
    let mut indices_counter = 0;

    let p = polylist.p.as_ref().unwrap().value
        .split_ascii_whitespace()
        .map(|s| s.parse::<i32>().unwrap())
        .collect::<Vec<_>>();

    // it's assumed that if no <p> exists, then <vcount> doesn't exist either
    // so we can safely (?) unwrap here
    let vcounts = polylist.vcount.as_ref().unwrap().value
        .split_ascii_whitespace()
        .map(|s| s.parse::<u32>().unwrap())
        .collect::<Vec<u32>>();

    let mut max_offset = 1;

    for input in &polylist.inputs {
        match input.attr_semantic {
            InputSemantic::Normal => {
                normal_offset = input.attr_offset as i32;
                max_offset = max_offset.max(normal_offset + 1);

                normals = sources
                    .get(&input.attr_source[1..])
                    .unwrap()
                    .as_float_array()
                    .unwrap()
                    .data
                    .trim()
                    .split_ascii_whitespace()
                    .collect::<Vec<_>>()
                    .chunks_exact(3)
                    .map(|v| glm::vec3(v[0].parse::<f32>().unwrap(), v[1].parse::<f32>().unwrap(), v[2].parse::<f32>().unwrap()))
                    .collect::<Vec<_>>();
                },
            InputSemantic::Texcoord => {
                texcoord_offset = input.attr_offset as i32;
                max_offset = max_offset.max(texcoord_offset + 1);

                tex_coords = sources
                    .get(&input.attr_source[1..])
                    .unwrap()
                    .as_float_array()
                    .unwrap()
                    .data
                    .trim()
                    .split_ascii_whitespace()
                    .collect::<Vec<_>>()
                    .chunks_exact(2)
                    .map(|v| glm::vec2(v[0].parse::<f32>().unwrap(), v[1].parse::<f32>().unwrap()))
                    .collect::<Vec<_>>();
            }
            _ => {}, // ignore others
        }
    }

    let mut skip_by = 0;

    // TODO: ideally we'd return an error because it's not a bug with the code
    // but rather a malformed collada document
    assert_eq!(polylist.attr_count as usize, vcounts.len(), "polylist attr_count and vcount count mismatch");

    for i in 0..polylist.attr_count {
        let vcount = vcounts[i as usize];
        let poly = &p[skip_by..][0..vcount as usize * max_offset as usize];

        skip_by += max_offset as usize * vcount as usize;

        let norm = {
            if normal_offset != -1 {
                let idx = poly[normal_offset as usize];
                let idx = if idx < 0 {
                    normals.len() as i32 + idx
                } else {
                    idx
                };
                normals[idx as usize]
            } else {
                let a = positions[poly[0] as usize];
                let b = positions[poly[max_offset as usize] as usize];
                let c = positions[poly[max_offset as usize * 2] as usize];

                glm::normalize(glm::cross(
                        b - a,
                        c - a
                ))
            }
        };
        let texcoords = {
            if texcoord_offset != -1 {
                let idx = poly[texcoord_offset as usize];
                let idx = if idx < 0 {
                    tex_coords.len() as i32 + idx
                } else {
                    idx
                };
                tex_coords[idx as usize]
            } else {
                glm::vec2(0.0, 0.0)
            }
        };

        for i in 0..vcount {
            // position offset is assumed to be 0
            let pos_idx = poly[i as usize * max_offset as usize];
            let idx = if pos_idx < 0 {
                positions.len() as i32 + pos_idx
            } else {
                pos_idx
            };
            let pos = positions[idx as usize];

            let vertex = Vertex {
                position: pos,
                normal: norm,
                tex_coords: texcoords,
            };

            vertices.push(vertex);
        }

        // triangulate
        for j in 0..vcount - 2 {
            indices.push(indices_counter);
            indices.push(indices_counter + j + 1);
            indices.push(indices_counter + j + 2);
        }

        indices_counter += vcount;
    }

    // TODO: materials
    ObjMesh {
        name: node_name,
        vertices,
        indices,
        material: None,
    }
}

fn parse_dae(mut file: std::fs::File) -> Result<Object, Box<dyn std::error::Error>> {
    let now = std::time::Instant::now();

    let root = {
        let mut collada_str = String::new();
        file.read_to_string(&mut collada_str)?;
        ColladaDocument::from_str(&collada_str)?
    };
    let elapsed = now.elapsed();
    info!("Parsing took {}ms", elapsed.as_millis());

    let now = std::time::Instant::now();

    let (visual_scenes, geometries, materials, effects) = root.libraries
        .into_iter()
        .filter_map(|library| {
            match library {
                Library::VisualScenes(_) => Some(library),
                Library::Geometries(_) => Some(library),
                Library::Materials(_) => Some(library),
                Library::Effects(_) => Some(library),
                _ => None,
    }
        }).fold((
            HashMap::<String, VisualScene>::new(),
            HashMap::<String, Geometry>::new(),
            HashMap::<String, Material>::new(),
            HashMap::<String, Effect>::new(),
        ), move |mut acc, libraries| {
            match libraries {
                Library::VisualScenes(library) => {
                    for visual_scene in library.visual_scenes {
                        acc.0.insert(visual_scene.attr_id.to_owned().expect("invalid collada data. <visual_scene> tag missing id attribute"), visual_scene);
                    }
                },
                Library::Geometries(library) => {
                    for geometry in library.geometries {
                        // TODO: maybe we should just ignore the ones that dont have an id since
                        // there's no way to reference them anyway
                        acc.1.insert(geometry.attr_id.to_owned().expect("invalid collada data. <geometry> tag missing id attribute"), geometry);
                    }
                },
                Library::Materials(library) => {
                    for material in library.materials {
                        acc.2.insert(material.attr_id.to_owned().expect("invalid collada data. <material> tag missing id attribute"), material);
                    }
                },
                Library::Effects(library) => {
                    for effect in library.effects {
                        acc.3.insert(effect.attr_id.to_owned(), effect);
                    }
                },
                _ => {},
    }
            acc
        });

    let mut meshes = Vec::new();
    let mut min_aabb = glm::vec3(f32::MAX, f32::MAX, f32::MAX);
    let mut max_aabb = glm::vec3(f32::MIN, f32::MIN, f32::MIN);

    if let Some(s) = root.scene {
        if let Some(vs) = s.instance_visual_scene {
            // remove the # from the url
            let vs_url = &vs.attr_url[1..];
            let visual_scene = visual_scenes.get(vs_url).unwrap();

            if visual_scene.nodes.iter().all(|node| node.attr_type != NodeType::Node) {
                return Err("Visual Scene does not contain any supported nodes. JOINT nodes are not implemented yet".into());
            }

            if visual_scene.nodes.iter().all(|node| node.instance_geometry.is_empty()) {
                return Err("No top-level node in the visual scene contains an instance of a mesh. Controller instances are not implemented yet".into());
            }

            for node in &visual_scene.nodes {
                if let NodeType::Node = node.attr_type {
                    let node_name = node.attr_name.to_owned().unwrap_or("default_mesh".to_string());

                    // TODO: parse the transformation matrices
                    // for transformation in &node.transformations {
                    // }

                    for geometry_instance in &node.instance_geometry {
                        let geometry = geometries.get(&geometry_instance.attr_url[1..]).unwrap();
                        // we only care about meshes for now
                        if let GeometricElement::Mesh(mesh) = &geometry.geometric_element {
                            let sources = mesh.sources.iter().filter(|source| {
                                matches!(&source.array_element, ArrayElement::FloatArray(_))
                            }).fold(HashMap::new(), |mut acc, source| {
                                acc.insert(source.attr_id.clone(), &source.array_element);
                                acc
                            });

                            let (positions, normals, tex_coords,
                                normal_offset, texcoord_offset) =
                                parse_vertices(&mesh.vertices.inputs, &sources, &mut min_aabb, &mut max_aabb);

                            for primitive in &mesh.primitives {
                                match primitive {
                                    Primitive::Triangles(triangles) => {
                                        if triangles.p.is_some() {
                                            meshes.push(parse_triangles(
                                                    node_name.clone(),
                                                    triangles,
                                                    &positions,
                                                    normal_offset,
                                                    normals.clone(),
                                                    texcoord_offset,
                                                    tex_coords.clone(),
                                                    &sources
                                            ));
                                        }
                                    },
                                    Primitive::PolyList(polylist) => {
                                        if polylist.p.is_some() {
                                            meshes.push(parse_polylist(
                                                    node_name.clone(),
                                                    polylist,
                                                    &positions,
                                                    normal_offset,
                                                    normals.clone(),
                                                    texcoord_offset,
                                                    tex_coords.clone(),
                                                    &sources
                                            ));
                                        }
                                    },
                                    _ => {}, // ignore the other primitives for now
                                }
                            }
                        }
                    }

                    // TODO: parse the node hierarchy. needs a whole overhaul
                    // of the mesh struct and stuff
                }
            }
        } else {
            return Err("No visual scene found".into());
        }
    }

    let elapsed = now.elapsed();
    info!("Importing took {}ms", elapsed.as_millis());

    let aabb = AABB::new(min_aabb, max_aabb);

    Ok(Object{
        name: "default_obj".to_string(),
        meshes,
        aabb
    })
}

pub fn load_dae(file: std::fs::File) -> Result<Object, Box<dyn std::error::Error>> {
    parse_dae(file)

    // todo!()
}
