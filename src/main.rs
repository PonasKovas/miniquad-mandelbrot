use miniquad::conf::Conf;
use miniquad::{
    Bindings, Buffer, BufferLayout, BufferType, Context, EventHandler, FilterMode, MouseButton,
    Pipeline, Shader, ShaderMeta, Texture, TouchPhase, UniformBlockLayout, UniformType, UserData,
    VertexAttribute, VertexFormat,
};

#[repr(C)]
struct Vec2 {
    x: f32,
    y: f32,
}
#[repr(C)]
struct Vertex {
    pos: Vec2,
}
#[repr(C)]
struct Uniforms {
    transform: [f32; 16],
    num_colors: i32,
}

#[derive(Copy, Clone, Debug)]
enum Action {
    Idle,
    ZoomingIn(f32, f32),
    ZoomingOut(f32, f32),
}

struct Mandelbrot {
    pipeline: Pipeline,
    bindings: Bindings,
    zoom: f32,
    center: (f32, f32),
    action: Action,
}
const NUM_COLORS: i32 = 12;

// HSV values in [0..1]
// returns [r, g, b] values from 0 to 255
//From https://martin.ankerl.com/2009/12/09/how-to-create-random-colors-programmatically/
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [u8; 3] {
    let h_i = (h * 6.) as i32;
    let f = h * 6. - h_i as f32;
    let p = v * (1. - s);
    let q = v * (1. - f * s);
    let t = v * (1. - (1. - f) * s);
    let [r, g, b] = match h_i {
        0 => [v, t, p],
        1 => [q, v, p],
        2 => [p, v, t],
        3 => [p, q, v],
        4 => [t, p, v],
        5 => [v, p, q],
        _ => panic!("Unknown H value {}", h_i),
    };
    [(r * 255.) as u8, (g * 255.) as u8, (b * 255.) as u8]
}

impl Mandelbrot {
    fn new(ctx: &mut Context) -> Self {
        let vertices: [Vertex; 4] = [
            Vertex {
                pos: Vec2 { x: -1.0, y: -1.0 },
            },
            Vertex {
                pos: Vec2 { x: 1.0, y: -1.0 },
            },
            Vertex {
                pos: Vec2 { x: 1.0, y: 1.0 },
            },
            Vertex {
                pos: Vec2 { x: -1.0, y: 1.0 },
            },
        ];
        let vertex_buffer = Buffer::immutable(ctx, BufferType::VertexBuffer, &vertices);

        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];
        let index_buffer = Buffer::immutable(ctx, BufferType::IndexBuffer, &indices);

        let mut colors = vec![];
        let num = NUM_COLORS as f32;
        for i in 0..NUM_COLORS {
            let degree = i as f32 / num;
            let c = hsv_to_rgb(degree, 1., 1.);
            colors.extend(c.iter());
            colors.push(255);
        }

        let texture = Texture::from_rgba8(ctx, NUM_COLORS as u16, 1, &colors);
        texture.set_filter(ctx, FilterMode::Nearest);
        let bindings = Bindings {
            vertex_buffers: vec![vertex_buffer],
            index_buffer,
            images: vec![texture],
        };

        let shader = Shader::new(ctx, SHADER_VERTEX, SHADER_FRAGMENT, SHADER_META);

        let pipeline = Pipeline::new(
            ctx,
            &[BufferLayout::default()],
            &[VertexAttribute::new("pos", VertexFormat::Float2)],
            shader,
        );

        Mandelbrot {
            pipeline,
            bindings,
            zoom: 1.0,
            center: (0.0, 0.0),
            action: Action::Idle,
        }
    }
    // Returns two floats (x and y) from -0.5 to 0.5, with (0.0, 0.0) being the center of the screen
    fn norm_mouse_pos(self: &Self, ctx: &mut Context, x: f32, y: f32) -> (f32, f32) {
        let screen_size = ctx.screen_size();
        let pos = (
            4.0 * (x / screen_size.0 - 0.5).powi(3),
            4.0 * (y / screen_size.1 - 0.5).powi(3),
        );

        pos
    }
}

impl EventHandler for Mandelbrot {
    fn update(&mut self, _ctx: &mut Context) {
        // zoom in/out
        match self.action {
            Action::ZoomingIn(x, y) => {
                self.zoom *= 1.01;
                self.center.0 -= x / self.zoom;
                self.center.1 += y / self.zoom;
            }
            Action::ZoomingOut(x, y) => {
                self.zoom /= 1.01;
                self.center.0 += x / self.zoom;
                self.center.1 -= y / self.zoom;
            }
            _ => {}
        }
    }

    fn draw(&mut self, ctx: &mut Context) {
        // draw the mandelbrot set
        ctx.begin_default_pass(Default::default());

        ctx.apply_pipeline(&self.pipeline);
        ctx.apply_bindings(&self.bindings);

        // make sure to not stretch
        let screen_size = ctx.screen_size();
        let ratio = screen_size.1 / screen_size.0;
        let (mut scale_x, mut scale_y) = if ratio <= 1.0 {
            (ratio, 1.0)
        } else {
            (1.0, 1.0 / ratio)
        };

        scale_x *= self.zoom;
        scale_y *= self.zoom;

        #[rustfmt::skip]
        ctx.apply_uniforms(&Uniforms {
            transform: [
                scale_x, 0.0, 0.0, 0.0,
                0.0, scale_y, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                (scale_x * self.center.0), (scale_y * self.center.1), 0.0, 1.0,
            ],
            num_colors: NUM_COLORS,
        });

        ctx.draw(0, 2 * 3, 1);

        ctx.end_render_pass();

        ctx.commit_frame();
    }

    fn mouse_button_down_event(&mut self, ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        let pos = self.norm_mouse_pos(ctx, x, y);

        if let MouseButton::Left = button {
            self.action = Action::ZoomingIn(pos.0, pos.1);
        } else if let MouseButton::Right = button {
            self.action = Action::ZoomingOut(pos.0, pos.1);
        }
    }

    fn mouse_button_up_event(&mut self, _ctx: &mut Context, _b: MouseButton, _x: f32, _y: f32) {
        self.action = Action::Idle;
    }

    fn mouse_motion_event(&mut self, ctx: &mut Context, x: f32, y: f32) {
        let pos = self.norm_mouse_pos(ctx, x, y);

        match self.action {
            Action::ZoomingIn(..) => {
                self.action = Action::ZoomingIn(pos.0, pos.1);
            }
            Action::ZoomingOut(..) => {
                self.action = Action::ZoomingOut(pos.0, pos.1);
            }
            _ => {}
        }
    }

    fn touch_event(&mut self, ctx: &mut Context, phase: TouchPhase, _id: u64, x: f32, y: f32) {
        let pos = self.norm_mouse_pos(ctx, x, y);

        match phase {
            TouchPhase::Started => {
                self.action = Action::ZoomingIn(pos.0, pos.1);
            }
            TouchPhase::Moved => {
                self.action = Action::ZoomingIn(pos.0, pos.1);
            }
            _ => {
                self.action = Action::Idle;
            }
        }
    }
}

fn main() {
    miniquad::start(Conf::default(), |mut ctx| {
        UserData::owning(Mandelbrot::new(&mut ctx), ctx)
    });
}

const SHADER_VERTEX: &str = r#"#version 100

uniform highp mat4 transform;

attribute highp vec2 pos;
varying highp vec2 texcoord;

void main() {
    gl_Position = transform * vec4(pos, 0, 1);
    texcoord = vec2(pos.x/2.0 + 0.5, 1.0 - (pos.y/2.0 + 0.5));
}"#;

const SHADER_FRAGMENT: &str = r#"#version 100

precision highp float;

varying highp vec2 texcoord;

uniform sampler2D tex;
uniform int num_colors;

const int max_iterations = 500;
const float cxmin = -2.0;
const float cxmax = 1.0;
const float cymin = -1.5;
const float cymax = 1.5;

const float scale_x = cxmax - cxmin;
const float scale_y = cymax - cymin;

vec2 square_complex(vec2 c) {
    return( vec2(
        c.x*c.x - c.y*c.y,
        2.0 * c.x * c.y
    ));
}

void main() {
    vec2 c = vec2(texcoord.x*scale_x + cxmin, texcoord.y*scale_y + cymin);
    vec2 z = vec2(0.0, 0.0);

    int b = -1;
    for (int i = 0; i < max_iterations; i++) {
        if (z.x*z.x + z.y*z.y > 4.0) {
            b = i;
            break;
        }
        z = square_complex(z) + c;
    }
    if(b == -1) {
        b = max_iterations;
    }
    if (b == max_iterations) {
        gl_FragColor = vec4(0, 0, 0, 1);
    } else {
        float x = float(b-((b / num_colors)*num_colors))/float(num_colors);
        gl_FragColor = texture2D(tex, vec2(x, 0.5));
    }
}"#;

const SHADER_META: ShaderMeta = ShaderMeta {
    images: &["tex"],
    uniforms: UniformBlockLayout {
        uniforms: &[
            ("transform", UniformType::Mat4),
            ("num_colors", UniformType::Int1),
        ],
    },
};
