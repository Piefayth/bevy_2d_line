use bevy::{
    prelude::*,
    render::{
        mesh::{MeshVertexAttribute, MeshVertexBufferLayoutRef, PrimitiveTopology},
        render_asset::RenderAssetUsages,
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
            VertexBufferLayout, VertexFormat, VertexStepMode,
        },
        Extract, RenderApp,
    },
    sprite::{Material2d, Material2dKey, Material2dPlugin, MaterialMesh2dBundle},
};

pub struct LineRenderingPlugin;

impl Plugin for LineRenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<LineMaterial>::default())
            .register_type::<Line>()
            .add_systems(Update, update_line_meshes);

        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_systems(ExtractSchedule, extract_lines);
    }
}

#[derive(Component, Clone, Reflect, Default)]
pub struct Line {
    pub points: Vec<Vec2>,
    pub colors: Vec<LinearRgba>,
    pub thickness: f32,
}

const ATTRIBUTE_POSITION: MeshVertexAttribute = MeshVertexAttribute::new("Vertex_Position", 0, VertexFormat::Float32x3);
const ATTRIBUTE_NORMAL: MeshVertexAttribute = MeshVertexAttribute::new("Vertex_Normal", 1, VertexFormat::Float32x2);
const ATTRIBUTE_MITER: MeshVertexAttribute = MeshVertexAttribute::new("Vertex_Miter", 2, VertexFormat::Float32);
const ATTRIBUTE_COLOR: MeshVertexAttribute = MeshVertexAttribute::new("Vertex_Color", 3, VertexFormat::Float32x4);

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct LineMaterial {
    #[uniform(0)]
    pub thickness: f32,
}

impl Material2d for LineMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/line.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/line.wgsl".into()
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = VertexBufferLayout::from_vertex_formats(
            VertexStepMode::Vertex,
            vec![
                VertexFormat::Float32x3, // position
                VertexFormat::Float32x2, // normal
                VertexFormat::Float32,   // miter
                VertexFormat::Float32x4, // color
            ],
        );
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}

fn update_line_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<LineMaterial>>,
    query: Query<(Entity, &Line), Changed<Line>>,
) {
    for (entity, line) in query.iter() {
        if line.points.len() < 2 || line.colors.len() != line.points.len() {
            continue; // Not enough points or mismatched colors
        }

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleStrip, RenderAssetUsages::RENDER_WORLD);

        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut miters = Vec::new();
        let mut colors = Vec::new();

        for i in 0..line.points.len() {
            let (normal, miter) = if i == 0 {
                let dir = (line.points[1] - line.points[0]).normalize();
                (Vec2::new(-dir.y, dir.x), 1.0)
            } else if i == line.points.len() - 1 {
                let dir = (line.points[i] - line.points[i-1]).normalize();
                (Vec2::new(-dir.y, dir.x), 1.0)
            } else {
                let prev = (line.points[i] - line.points[i-1]).normalize();
                let next = (line.points[i+1] - line.points[i]).normalize();
                let tangent = (prev + next).normalize();
                let miter = Vec2::new(-tangent.y, tangent.x);
                let length = 1.0 / Vec2::dot(miter, Vec2::new(-prev.y, prev.x));
                (miter, length)
            };

            positions.push([line.points[i].x, line.points[i].y, 0.0]);
            positions.push([line.points[i].x, line.points[i].y, 0.0]);
            normals.push([-normal.x, -normal.y]);
            normals.push([normal.x, normal.y]);
            miters.push(miter);
            miters.push(miter);
            colors.push(line.colors[i].to_f32_array());
            colors.push(line.colors[i].to_f32_array());
        }

        mesh.insert_attribute(ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(ATTRIBUTE_MITER, miters);
        mesh.insert_attribute(ATTRIBUTE_COLOR, colors);

        commands.entity(entity).try_insert(MaterialMesh2dBundle {
            mesh: bevy::sprite::Mesh2dHandle(meshes.add(mesh)),
            material: materials.add(LineMaterial { thickness: line.thickness }),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        });
    }
}

fn extract_lines(
    mut commands: Commands,
    query: Extract<Query<(Entity, &Line)>>,
) {
    for (entity, line) in query.iter() {
        commands.get_or_spawn(entity).insert(line.clone());
    }
}