extern crate core;

use std::collections::HashMap;
use std::default::Default;
use std::fs::File;
use std::ops::Div;
use glium::{Blend, DrawParameters, implement_vertex, program, Surface, uniform};
use glium::index::{NoIndices, PrimitiveType};
use glutin::dpi::PhysicalSize;
use glam::Mat4;
use serde_json::Value;
use serde::Deserialize;
use artery_font::ArteryFont;

#[derive(Debug, Copy, Clone, Deserialize)]
struct Rect {
    left: f32,
    bottom: f32,
    right: f32,
    top: f32
}

impl Div<(f32, f32)> for Rect{
    type Output = Rect;

    fn div(self, (width, height): (f32, f32)) -> Self::Output {
        Self {
            left: self.left / width,
            bottom: self.bottom / height,
            right: self.right / width,
            top: self.top / height
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    tex_coord: [f32; 2],
}
implement_vertex!(Vertex, position, tex_coord);

#[derive(Debug, Copy, Clone)]
struct Quad {
    plane_bounds: Rect,
    atlas_bounds: Rect
}

impl Quad {

    fn vertices(&self, x_offset: f32, y_offset: f32) -> impl Iterator<Item=Vertex> {
        [
            Vertex { position: [x_offset + self.plane_bounds.left , y_offset + self.plane_bounds.bottom], tex_coord: [self.atlas_bounds.left , self.atlas_bounds.bottom] },
            Vertex { position: [x_offset + self.plane_bounds.right, y_offset + self.plane_bounds.bottom], tex_coord: [self.atlas_bounds.right, self.atlas_bounds.bottom] },
            Vertex { position: [x_offset + self.plane_bounds.left , y_offset + self.plane_bounds.top   ], tex_coord: [self.atlas_bounds.left , self.atlas_bounds.top   ] },
            Vertex { position: [x_offset + self.plane_bounds.left , y_offset + self.plane_bounds.top   ], tex_coord: [self.atlas_bounds.left , self.atlas_bounds.top   ] },
            Vertex { position: [x_offset + self.plane_bounds.right, y_offset + self.plane_bounds.bottom], tex_coord: [self.atlas_bounds.right, self.atlas_bounds.bottom] },
            Vertex { position: [x_offset + self.plane_bounds.right, y_offset + self.plane_bounds.top   ], tex_coord: [self.atlas_bounds.right, self.atlas_bounds.top   ] },
        ].into_iter()
    }

}

#[derive(Debug, Copy, Clone)]
struct Glyph {
    advance: f32,
    quad: Option<Quad>
}

impl Glyph {

    fn vertices(&self, x_offset: f32, y_offset: f32) -> impl Iterator<Item=Vertex> + '_ {
        self.quad.iter().flat_map(move |q|q.vertices(x_offset, y_offset))
    }

}

fn main() {
    let arfont = ArteryFont::read(&include_bytes!("../raw.arfont")[..]).map(|mut font| {
        for v in &mut font.variants {
            v.glyphs.clear();
        }
        for i in &mut font.images {
            i.data.clear();
        }
        font
    });
    println!("{:#?}", arfont);

    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(1280.0, 720.0))
        .with_resizable(false);
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    let image = image::open("test.png").unwrap().to_rgba8();
    let image_dimensions = image.dimensions();
    let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
    let opengl_texture = glium::texture::Texture2d::new(&display, image).unwrap();


    let (glyphs, line_height) = {
        let json: Value = serde_json::from_reader(File::open("test.json").unwrap()).unwrap();
        let width = json["atlas"]["width"].as_f64().unwrap() as f32;
        let height = json["atlas"]["height"].as_f64().unwrap() as f32;
        let line_height = json["metrics"]["lineHeight"].as_f64().unwrap() as f32;

        let mut glyphs = HashMap::new();

        for glyph in json["glyphs"].as_array().unwrap() {
            let unicode = std::char::from_u32(glyph["unicode"].as_u64().unwrap() as u32).unwrap();
            let advance = glyph["advance"].as_f64().unwrap() as f32;
            let plane_bounds = Rect::deserialize(&glyph["planeBounds"]).ok();
            let atlas_bounds = Rect::deserialize(&glyph["atlasBounds"]).map(|r| r / (width, height)).ok();
            glyphs.insert(unicode, Glyph {
                advance,
                quad: plane_bounds.and_then(|pb| atlas_bounds.map(|ab| Quad { plane_bounds: pb, atlas_bounds: ab }))
            });
        }

        (glyphs, line_height)
    };

    let text = "Hello World!\nThis an example for text rendering\nusing msdf fonts";

    // building the vertex buffer, which contains all the vertices that we will draw
    let vertex_buffer = {

        let mut vertices = Vec::new();
        let mut x;
        let mut y = 6.0;

        for line in text.lines() {
            x = 1.0;
            for glyph in line.chars().map(|c|glyphs[&c]) {
                for v in glyph.vertices(x, y) {
                    vertices.push(v);
                }
                x += glyph.advance;
            }
            y -= line_height * 0.55;
        }


        glium::VertexBuffer::new(&display,
                                 vertices.as_slice()
        ).unwrap()
    };

    // compiling shaders and linking them together
    let program = program!(&display,
        450 => {
            vertex: "
                #version 450 core

                in vec2 position;
                in vec2 tex_coord;

                out vec2 v_tex_coords;

                uniform mat4 matrix;

                void main() {
                    gl_Position = matrix * vec4(position, 0.0, 1.0);
                    v_tex_coords = tex_coord;
                }
            ",

            fragment: "
                #version 450 core
                in vec2 v_tex_coords;
                out vec4 f_color;
                uniform sampler2D tex;

                float median(float r, float g, float b) {
                    return max(min(r, g), min(max(r, g), b));
                }

                float screenPxRange = 4.5;

                void main() {
                    vec3 msd = texture(tex, v_tex_coords).rgb;
                    float sd = median(msd.r, msd.g, msd.b);
                    float screenPxDistance = screenPxRange * (sd - 0.5);
                    float opacity = clamp(screenPxDistance + 0.5, 0.0, 1.0);
                    f_color = vec4(1, 1, 1, opacity);
                }
            "
        }
    ).unwrap();


    let scale = 15.0;
    // Here we draw the black background and triangle to the screen using the previously
    // initialised resources.
    //
    // In this case we use a closure for simplicity, however keep in mind that most serious
    // applications should probably use a function that takes the resources as an argument.
    let draw = move || {
        // building the uniforms
        let uniforms = uniform! {
            matrix: Mat4::orthographic_rh(0.0, 1.28 * scale, 0.0, 0.72 * scale, 0.0, 1.0).to_cols_array_2d(),
            tex: &opengl_texture
        };

        // drawing a frame
        let mut target = display.draw();
        target.clear_color(0.3, 0.3, 0.3, 1.0);
        target.draw(&vertex_buffer, NoIndices(PrimitiveType::TrianglesList), &program, &uniforms, &DrawParameters {
            blend: Blend::alpha_blending(),
            ..Default::default()
        }).unwrap();
        target.finish().unwrap();
    };

    // Draw the triangle to the screen.
    draw();

    // the main loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = match event {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                // Break from the main loop when the window is closed.
                glutin::event::WindowEvent::CloseRequested => glutin::event_loop::ControlFlow::Exit,
                // Redraw the triangle when the window is resized.
                glutin::event::WindowEvent::Resized(..) => {
                    draw();
                    glutin::event_loop::ControlFlow::Poll
                },
                _ => glutin::event_loop::ControlFlow::Poll,
            },
            _ => glutin::event_loop::ControlFlow::Poll,
        };
    });
}

