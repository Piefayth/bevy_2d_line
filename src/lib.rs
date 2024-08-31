use bevy::{
    prelude::*,
    render::{
        mesh::{
            MeshVertexAttribute, MeshVertexBufferLayoutRef, PrimitiveTopology,
            VertexAttributeValues,
        },
        render_asset::RenderAssetUsages,
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
            VertexBufferLayout, VertexFormat, VertexStepMode,
        },
        Extract, RenderApp,
    },
    sprite::{Material2d, Material2dKey, Material2dPlugin, MaterialMesh2dBundle, Mesh2dHandle},
    utils::HashMap,
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

const ATTRIBUTE_POSITION: MeshVertexAttribute =
    MeshVertexAttribute::new("Vertex_Position", 0, VertexFormat::Float32x3);
const ATTRIBUTE_NORMAL: MeshVertexAttribute =
    MeshVertexAttribute::new("Vertex_Normal", 1, VertexFormat::Float32x2);
const ATTRIBUTE_MITER: MeshVertexAttribute =
    MeshVertexAttribute::new("Vertex_Miter", 2, VertexFormat::Float32);
const ATTRIBUTE_COLOR: MeshVertexAttribute =
    MeshVertexAttribute::new("Vertex_Color", 3, VertexFormat::Float32x4);

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
    mut material_cache: Local<HashMap<u32, Handle<LineMaterial>>>,
    query: Query<(Entity, &Line, Option<&Mesh2dHandle>), Changed<Line>>,
) {
    for (entity, line, maybe_mesh_handle) in query.iter() {
        if line.points.len() < 2 || line.colors.len() != line.points.len() {
            continue; // Not enough points or mismatched colors
        }

        let attribute_size = line.points.len() * 2;
        let rounded_key = (line.thickness * 1000.0).round() as u32; // thicknesses less than .00001 apart will use the same material

        let mesh = match maybe_mesh_handle {
            Some(mesh_handle) => meshes.get_mut(mesh_handle.id()).unwrap(),
            None => {
                let mut mesh = Mesh::new(
                    PrimitiveTopology::TriangleStrip,
                    RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
                );

                mesh.insert_attribute(
                    ATTRIBUTE_POSITION,
                    Vec::<[f32; 3]>::with_capacity(attribute_size),
                );
                mesh.insert_attribute(
                    ATTRIBUTE_NORMAL,
                    Vec::<[f32; 2]>::with_capacity(attribute_size),
                );
                mesh.insert_attribute(ATTRIBUTE_MITER, Vec::<f32>::with_capacity(attribute_size));
                mesh.insert_attribute(
                    ATTRIBUTE_COLOR,
                    Vec::<[f32; 4]>::with_capacity(attribute_size),
                );

                let mesh_handle = Mesh2dHandle(meshes.add(mesh));
                let id = mesh_handle.id();

                let material_handle = if let Some(cached) = material_cache.get(&rounded_key) {
                    cached
                } else {
                    let handle = materials.add(LineMaterial {
                        thickness: line.thickness,
                    });
                    material_cache.insert(rounded_key, handle.clone());
                    &handle.clone()
                };

                commands.entity(entity).insert(MaterialMesh2dBundle {
                    mesh: mesh_handle,
                    material: material_handle.clone(),
                    ..default()
                });

                meshes.get_mut(id).unwrap()
            }
        };

        let mut resize = false;
        mesh.attributes_mut().for_each(|(id, values)| {
            match id {
                id if id == ATTRIBUTE_POSITION.id => {
                    if values.len() != attribute_size {
                        resize = true; // we can check in any of these, they will all change at the same time
                        *values = VertexAttributeValues::Float32x3(Vec::<[f32; 3]>::with_capacity(
                            attribute_size,
                        ));
                    }
                }
                id if id == ATTRIBUTE_NORMAL.id => {
                    if values.len() != attribute_size {
                        *values = VertexAttributeValues::Float32x2(Vec::<[f32; 2]>::with_capacity(
                            attribute_size,
                        ));
                    }
                }
                id if id == ATTRIBUTE_MITER.id => {
                    if values.len() != attribute_size {
                        *values = VertexAttributeValues::Float32(Vec::<f32>::with_capacity(
                            attribute_size,
                        ));
                    }
                }
                id if id == ATTRIBUTE_COLOR.id => {
                    if values.len() != attribute_size {
                        *values = VertexAttributeValues::Float32x4(Vec::<[f32; 4]>::with_capacity(
                            attribute_size,
                        ));
                    }
                }
                _ => {}
            }
        });

        for i in 0..line.points.len() {
            let (normal, miter) = if i == 0 {
                let dir = (line.points[1] - line.points[0]).normalize();
                (Vec2::new(-dir.y, dir.x), 1.0)
            } else if i == line.points.len() - 1 {
                let dir = (line.points[i] - line.points[i - 1]).normalize();
                (Vec2::new(-dir.y, dir.x), 1.0)
            } else {
                let prev = (line.points[i] - line.points[i - 1]).normalize();
                let next = (line.points[i + 1] - line.points[i]).normalize();
                let tangent = (prev + next).normalize();
                let miter = Vec2::new(-tangent.y, tangent.x);
                let length = 1.0 / Vec2::dot(miter, Vec2::new(-prev.y, prev.x));
                (miter, length)
            };

            mesh.attributes_mut().for_each(|(id, values)| {
                match id {
                    id if id == ATTRIBUTE_POSITION.id => {
                        match values {
                            VertexAttributeValues::Float32x3(ref mut v) => {
                                if resize {
                                    v.push([line.points[i].x, line.points[i].y, 0.0]);
                                    v.push([line.points[i].x, line.points[i].y, 0.0]);
                                } else {
                                    v[i * 2] = [line.points[i].x, line.points[i].y, 0.0];
                                    v[(i * 2) + 1] = [line.points[i].x, line.points[i].y, 0.0];
                                }

                            },
                            _ => panic!("Position attribute was unexpected size. Developer was supposed to uphold this.")
                        }

                    },
                    id if id == ATTRIBUTE_NORMAL.id => {
                        match values {
                            VertexAttributeValues::Float32x2(ref mut v) => {
                                if resize {
                                    v.push([-normal.x, -normal.y]);
                                    v.push([normal.x, normal.y]);
                                } else {
                                    v[i * 2] = [-normal.x, -normal.y];
                                    v[(i * 2) + 1] = [normal.x, normal.y];
                                }
                            },
                            _ => panic!("Normal attribute was unexpected size. Developer was supposed to uphold this.")
                        }
                    },
                    id if id == ATTRIBUTE_MITER.id => {
                        match values {
                            VertexAttributeValues::Float32(ref mut v) => {
                                if resize {
                                    v.push(miter);
                                    v.push(miter);
                                } else {
                                    v[i * 2] = miter;
                                    v[(i * 2) + 1] = miter;
                                }
                            },
                            _ => panic!("Miter attribute was unexpected size. Developer was supposed to uphold this.")
                        }

                    },
                    id if id == ATTRIBUTE_COLOR.id => {
                        match values {
                            VertexAttributeValues::Float32x4(ref mut v) => {
                                if resize {
                                    v.push(line.colors[i].to_f32_array());
                                    v.push(line.colors[i].to_f32_array());
                                } else {
                                    v[i * 2] = line.colors[i].to_f32_array();
                                    v[(i * 2) + 1] = line.colors[i].to_f32_array();
                                }

                            },
                            _ => panic!("Color attribute was unexpected size. Developer was supposed to uphold this.")
                        }
                    },
                    _ => {}
                }
            });
        }
    }
}

fn extract_lines(mut commands: Commands, query: Extract<Query<(Entity, &Line)>>) {
    for (entity, line) in query.iter() {
        commands.get_or_spawn(entity).insert(line.clone());
    }
}
