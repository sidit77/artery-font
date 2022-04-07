use glium::{implement_vertex, program, Surface, uniform};
use glium::index::{NoIndices, PrimitiveType};
use glutin::dpi::PhysicalSize;
use glam::Mat4;

fn main() {
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(1280.0, 720.0))
        .with_resizable(false);
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    // building the vertex buffer, which contains all the vertices that we will draw
    let vertex_buffer = {
        #[derive(Copy, Clone)]
        struct Vertex {
            position: [f32; 2],
            color: [f32; 3],
        }

        implement_vertex!(Vertex, position, color);

        glium::VertexBuffer::new(&display,
                                 &[
                                     Vertex { position: [-1.0, -1.0], color: [0.0, 1.0, 0.0] },
                                     Vertex { position: [ 1.0, -1.0], color: [0.0, 0.0, 1.0] },
                                     Vertex { position: [-1.0,  1.0], color: [1.0, 0.0, 0.0] },
                                     Vertex { position: [-1.0,  1.0], color: [1.0, 0.0, 0.0] },
                                     Vertex { position: [ 1.0, -1.0], color: [0.0, 0.0, 1.0] },
                                     Vertex { position: [ 1.0,  1.0], color: [0.0, 1.0, 0.0] },
                                 ]
        ).unwrap()
    };

    // compiling shaders and linking them together
    let program = program!(&display,
        450 => {
            vertex: "
                #version 450 core

                layout(location = 0) in vec2 position;
                layout(location = 1) in vec3 color;

                out vec3 vColor;

                uniform mat4 matrix;

                void main() {
                    gl_Position = matrix * vec4(position, 0.0, 1.0);
                    vColor = color;
                }
            ",

            fragment: "
                #version 450 core
                in vec3 vColor;
                out vec4 f_color;
                void main() {
                    f_color = vec4(vColor, 1.0);
                }
            "
        }
    ).unwrap();


    // Here we draw the black background and triangle to the screen using the previously
    // initialised resources.
    //
    // In this case we use a closure for simplicity, however keep in mind that most serious
    // applications should probably use a function that takes the resources as an argument.
    let draw = move || {
        // building the uniforms
        let uniforms = uniform! {
            matrix: Mat4::orthographic_rh(-6.4, 6.4, -3.6, 3.6, 0.0, 1.0).to_cols_array_2d()
        };

        // drawing a frame
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 0.0);
        target.draw(&vertex_buffer, NoIndices(PrimitiveType::TrianglesList), &program, &uniforms, &Default::default()).unwrap();
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

