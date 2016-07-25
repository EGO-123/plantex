use base::world::{self, Chunk, HexPillar, PillarSection, PropType};
use base::math::*;
use glium::{self, DrawParameters, IndexBuffer, Program, VertexBuffer};
use glium::draw_parameters::{BackfaceCullingMode, DepthTest};
use glium::backend::Facade;
use glium::index::PrimitiveType;
use Camera;
use util::ToArr;
use std::f32::consts;
use world::plant_view::PlantView;

/// Graphical representation of the `base::Chunk`.
pub struct ChunkView {
    vertices: VertexBuffer<Vertex>,
    program: Program,
    pillars: Vec<PillarView>,
    index_buffer: IndexBuffer<u32>,
}



impl ChunkView {
    /// Creates the graphical representation of given chunk at the given chunk
    /// offset
    pub fn from_chunk<F: Facade>(chunk: &Chunk, offset: AxialPoint, facade: &F) -> Self {


        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        get_top_hexagon_model(&mut vertices, &mut indices);
        get_bottom_hexagon_model(&mut vertices, &mut indices);
        get_side_hexagon_model(4, 5, &mut vertices, &mut indices);
        get_side_hexagon_model(1, 2, &mut vertices, &mut indices);
        get_side_hexagon_model(5, 0, &mut vertices, &mut indices);
        get_side_hexagon_model(0, 1, &mut vertices, &mut indices);
        get_side_hexagon_model(3, 4, &mut vertices, &mut indices);
        get_side_hexagon_model(2, 3, &mut vertices, &mut indices);



        let vbuf = VertexBuffer::new(facade, &vertices).unwrap();
        let prog = Program::from_source(facade,
                                        include_str!("chunk_std.vert"),
                                        include_str!("chunk_std.frag"),
                                        None)
            .unwrap();

        let mut pillars = Vec::new();
        for q in 0..world::CHUNK_SIZE * world::CHUNK_SIZE {
            let pos = offset.to_real() +
                      AxialVector::new((q / world::CHUNK_SIZE).into(),
                                       (q % world::CHUNK_SIZE).into())
                .to_real();
            let pillar = &chunk.pillars()[q as usize];
            pillars.push(PillarView::from_pillar(pos, pillar, facade));
        }

        let ibuf = IndexBuffer::new(facade, PrimitiveType::TrianglesList, &indices).unwrap();

        ChunkView {
            vertices: vbuf,
            program: prog,
            pillars: pillars,
            index_buffer: ibuf,
        }
    }

    pub fn draw<S: glium::Surface>(&self, surface: &mut S, camera: &Camera) {
        for pillar in &self.pillars {
            for section in &pillar.sections {
                let height = section.top.units() - section.bottom.units();

                let uniforms = uniform! {
                    height: height as f32,
                    offset: [pillar.pos.x, pillar.pos.y, section.bottom.to_real()],
                    proj_matrix: camera.proj_matrix().to_arr(),
                    view_matrix: camera.view_matrix().to_arr(),
                    material_color: section.ground.get_color(),
                };
                let params = DrawParameters {
                    depth: glium::Depth {
                        write: true,
                        test: DepthTest::IfLess,
                        ..Default::default()
                    },
                    backface_culling: BackfaceCullingMode::CullCounterClockwise,
                    ..Default::default()
                };

                surface.draw(&self.vertices,
                          &self.index_buffer,
                          &self.program,
                          &uniforms,
                          &params)
                    .unwrap();
            }

            for plant in &pillar.plants {
                plant.draw(surface, camera);
            }
        }
    }
}


/// Vertex type used to render chunks (or hex pillars).
#[derive(Debug, Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}

implement_vertex!(Vertex, position, normal);

pub struct PillarView {
    pos: Point2f,
    sections: Vec<PillarSection>,
    plants: Vec<PlantView>,
}

impl PillarView {
    fn from_pillar<F: Facade>(pos: Point2f, pillar: &HexPillar, facade: &F) -> PillarView {
        PillarView {
            pos: pos,
            sections: pillar.sections().to_vec(),
            plants: pillar.props()
                .iter()
                .map(|prop| {
                    match prop.prop {
                        PropType::Plant(ref plant) => {
                            let pos = Point3f::new(pos.x, pos.y, prop.baseline.to_real());
                            PlantView::from_plant(pos, plant, facade)
                        }
                    }
                })
                .collect(),
        }
    }
}


/// Calculates one Point-coordinates of a Hexagon
fn hex_corner(size: f32, i: i32) -> (f32, f32) {
    let angle_deg = 60.0 * (i as f32) + 30.0;
    let angle_rad = (consts::PI / 180.0) * angle_deg;

    (size * angle_rad.cos(), size * angle_rad.sin())
}
/// Calculates the top face of the Hexagon and normals
fn get_top_hexagon_model(vertices: &mut Vec<Vertex>, indices: &mut Vec<u32>) {
    let cur_len = vertices.len() as u32;
    for i in 0..6 {
        let (x, y) = hex_corner(world::HEX_OUTER_RADIUS, i);

        vertices.push(Vertex {
            position: [x, y, world::PILLAR_STEP_HEIGHT],
            normal: [0.0, 0.0, 1.0],
        });
    }

    vertices.push(Vertex {
        position: [0.0, 0.0, world::PILLAR_STEP_HEIGHT],
        normal: [0.0, 0.0, 1.0],
    });

    indices.append(&mut vec![cur_len + 0,
                             cur_len + 6,
                             cur_len + 1,
                             cur_len + 5,
                             cur_len + 6,
                             cur_len + 0,
                             cur_len + 4,
                             cur_len + 6,
                             cur_len + 5,
                             cur_len + 3,
                             cur_len + 6,
                             cur_len + 4,
                             cur_len + 2,
                             cur_len + 6,
                             cur_len + 3,
                             cur_len + 1,
                             cur_len + 6,
                             cur_len + 2]);
}

/// Calculates the bottom face of the Hexagon and the normals
fn get_bottom_hexagon_model(vertices: &mut Vec<Vertex>, indices: &mut Vec<u32>) {
    let cur_len = vertices.len() as u32;
    for i in 0..6 {
        let (x, y) = hex_corner(world::HEX_OUTER_RADIUS, i);

        vertices.push(Vertex {
            position: [x, y, 0.0],
            normal: [0.0, 0.0, -1.0],
        });
    }

    vertices.push(Vertex {
        position: [0.0, 0.0, 0.0],
        normal: [0.0, 0.0, -1.0],
    });
    indices.append(&mut vec![cur_len + 1,
                             cur_len + 6,
                             cur_len + 0,
                             cur_len + 0,
                             cur_len + 6,
                             cur_len + 5,
                             cur_len + 5,
                             cur_len + 6,
                             cur_len + 4,
                             cur_len + 4,
                             cur_len + 6,
                             cur_len + 3,
                             cur_len + 3,
                             cur_len + 6,
                             cur_len + 2,
                             cur_len + 2,
                             cur_len + 6,
                             cur_len + 1]);
}

/// Calculates the sides of the Hexagon and normals
fn get_side_hexagon_model(ind1: i32,
                          ind2: i32,
                          vertices: &mut Vec<Vertex>,
                          indices: &mut Vec<u32>) {
    let cur_len = vertices.len() as u32;
    let (x1, y1) = hex_corner(world::HEX_OUTER_RADIUS, ind1);
    let (x2, y2) = hex_corner(world::HEX_OUTER_RADIUS, ind2);
    let normal = [y1 + y2, x1 + x2, 0.0];

    vertices.push(Vertex {
        position: [x1, y1, world::PILLAR_STEP_HEIGHT],
        normal: normal,
    });
    vertices.push(Vertex {
        position: [x1, y1, 0.0],
        normal: normal,
    });
    vertices.push(Vertex {
        position: [x2, y2, world::PILLAR_STEP_HEIGHT],
        normal: normal,
    });
    vertices.push(Vertex {
        position: [x2, y2, 0.0],
        normal: normal,
    });

    indices.append(&mut vec![cur_len + 0,
                             cur_len + 2,
                             cur_len + 1,
                             cur_len + 1,
                             cur_len + 2,
                             cur_len + 3]);
}