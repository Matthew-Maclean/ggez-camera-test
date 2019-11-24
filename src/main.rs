use ggez::
{
    self,
    Context, ContextBuilder, GameResult,
    conf::{WindowSetup, WindowMode},
    event::{run, EventHandler},
    graphics::{self, clear, present, DrawParam, Mesh, MeshBuilder, draw},
    input::{mouse::MouseButton, keyboard::{KeyCode, KeyMods}},
    mint::{Point2, Vector2},
};

fn main() -> GameResult<()>
{
    let (mut ctx, mut eloop) = ContextBuilder::new("Camera Test", "Camera Tester")
        .window_setup(WindowSetup
        {
            title: "Camera Test".to_owned(),
            ..Default::default()
        })
        .window_mode(WindowMode
        {
            width: 500f32,
            height: 500f32,
            ..Default::default()
        })
        .build()?;

    let mut scene = Scene::new(&mut ctx);

    run(&mut ctx, &mut eloop, &mut scene)
}

// the camera
struct Camera
{
    pos: Point2<f32>,
    scale: Vector2<f32>,
}

impl Camera
{
    #[allow(dead_code)]
    pub fn new(pos: Point2<f32>, scale: Vector2<f32>) -> Camera
    {
        Camera
        {
            pos: pos,
            scale: scale,
        }
    }

    pub fn shift(&mut self, dx: f32, dy: f32)
    {
        self.pos.x += dx;
        self.pos.y += dy;
    }

    pub fn scale(&mut self, scale: Vector2<f32>, fixed: Option<Point2<f32>>)
    {
        if let Some(fixed) = fixed
        {
            let pre_scale = self.reverse_transform(fixed);

            self.scale.x *= scale.x;
            self.scale.y *= scale.y;

            let post_scale = self.reverse_transform(fixed);
            
            let dx = (pre_scale.x - post_scale.x) * self.scale.x;
            let dy = (pre_scale.y - post_scale.y) * self.scale.y;

            self.shift(dx, dy);
        }
        else
        {
            self.scale.x *= scale.x;
            self.scale.y *= scale.y;
        }
    }

    pub fn reset(&mut self)
    {
        self.pos = Point2{ x: 0f32, y: 0f32 };
        self.scale = Vector2{ x: 1f32, y: 1f32 };
    }

    pub fn reverse_transform(&self, mut p: Point2<f32>) -> Point2<f32>
    {
        p.x /= self.scale.x;
        p.y /= self.scale.y;

        p.x += self.pos.x / self.scale.x;
        p.y += self.pos.y / self.scale.y;

        p
    }

    pub fn transform(&self, input: &Transform) -> Transform
    {
        let mut t = input.clone();

        t.pos.x -= self.pos.x;
        t.pos.y -= self.pos.y;

        t.scale.x *= self.scale.x;
        t.scale.y *= self.scale.y;

        t
    }
}

impl Default for Camera
{
    fn default() -> Camera
    {
        Camera
        {
            pos: Point2{ x: 0f32, y: 0f32 },
            scale: Vector2{ x: 1f32, y: 1f32 },
        }
    }
}

// a holder struct for transform info for drawable things
#[derive(Copy, Clone, Debug)]
struct Transform
{
    pos: Point2<f32>,
    scale: Vector2<f32>,
}

impl Transform
{
    #[allow(dead_code)]
    pub fn new(pos: Point2<f32>, scale: Vector2<f32>) -> Transform
    {
        Transform
        {
            pos: pos,
            scale: scale,
        }
    }
}

impl Into<DrawParam> for Transform
{
    fn into(self) -> DrawParam
    {
        DrawParam::default().dest(self.pos).scale(self.scale)
    }
}

impl Default for Transform
{
    fn default() -> Transform
    {
        Transform
        {
            pos: Point2{ x: 0f32, y: 0f32 },
            scale: Vector2{ x: 1f32, y: 1f32 },
        }
    }
}

struct Scene
{
    objects: Vec<(Mesh, Transform)>,
    camera: Camera,
    mouse_data: MouseData,
    key_data: KeyData,
}

impl Scene
{
    pub fn new(ctx: &mut Context) -> Scene
    {
        let mut objects = Vec::new();

        objects.push(Scene::init_grid(ctx));

        Scene
        {
            objects: objects,
            camera: Camera::default(),
            mouse_data: MouseData::default(),
            key_data: KeyData::default(),
        }
    }

    fn init_grid(ctx: &mut Context) -> (Mesh, Transform)
    {
        let mut mb = MeshBuilder::new();

        for i in 0..10
        {
            let c = if i == 5
            {
                graphics::Color::from_rgb(255, 0, 0)
            }
            else
            {
                graphics::BLACK
            };

            let i = i as f32;
            // x line
            mb.line(
                &[[i * 50f32, 0f32], [i * 50f32, 500f32]],
                1f32,
                c
            ).unwrap();
            // y line
            mb.line(
                &[[0f32, i * 50f32], [500f32, i * 50f32]],
                1f32,
                c
            ).unwrap();
        }

        (
            mb.build(ctx).unwrap(),
            Transform::default()
        )
    }
}

impl EventHandler for Scene
{
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()>
    {
        if self.mouse_data.left_button_down
        {
            let (dx, dy) = (
                self.mouse_data.left_button_down_pos.unwrap().x - self.mouse_data.current_pos.x,
                self.mouse_data.left_button_down_pos.unwrap().y - self.mouse_data.current_pos.y
            );
            
            self.mouse_data.left_button_down_pos = Some(self.mouse_data.current_pos);

            self.camera.shift(dx, dy);
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()>
    {
        clear(ctx, graphics::WHITE);
        
        for (o, t) in self.objects.iter()
        {
            draw(ctx, o, self.camera.transform(t))?;
        }

        present(ctx)
    }

    fn mouse_wheel_event(&mut self, _ctx: &mut Context, _x: f32, y: f32)
    {
        if self.key_data.control_down || self.key_data.shift_down
        {
            if self.key_data.control_down
            {
                if y < 0f32
                {
                    self.camera.shift(10f32, 0f32);
                }
                else
                {
                    self.camera.shift(-10f32, 0f32);
                }
            }
            if self.key_data.shift_down
            {
                if y < 0f32
                {
                    self.camera.shift(0f32, 10f32);
                }
                else
                {
                    self.camera.shift(0f32, -10f32);
                }
            }
        }
        else
        {
            if y < 0f32
            {
                self.camera.scale(Vector2{ x: 0.95, y: 0.95 }, Some(self.mouse_data.current_pos));
            }
            else
            {
                self.camera.scale(Vector2{ x: 1.05, y: 1.05 }, Some(self.mouse_data.current_pos));
            }
        }
    }

    fn mouse_motion_event(&mut self, _ctx: &mut Context, x: f32, y: f32, _dx: f32, _dy: f32)
    {
        self.mouse_data.current_pos = Point2{ x: x, y: y};
    }

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, button: MouseButton, x: f32, y: f32)
    {
        if button == MouseButton::Left
        {
            self.mouse_data.left_button_down = true;
            self.mouse_data.left_button_down_pos = Some(Point2{ x: x, y: y });
        }
    }

    fn mouse_button_up_event(&mut self, _ctx: &mut Context, button: MouseButton, _x: f32, _y: f32)
    {
        if button == MouseButton::Left
        {
            self.mouse_data.left_button_down = false;
            self.mouse_data.left_button_down_pos = None;
        }
    }

    fn key_down_event(&mut self, ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods, repeat: bool)
    {
        if keycode == KeyCode::Escape
        {
            ggez::event::quit(ctx);
        }
        if repeat { return }
        if keycode == KeyCode::LControl
        {
            self.key_data.control_down = true;
        }
        if keycode == KeyCode::LShift
        {
            self.key_data.shift_down = true;
        }
        if keycode == KeyCode::R
        {
            self.camera.reset();
        }
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods)
    {
        if keycode == KeyCode::LControl
        {
            self.key_data.control_down = false;
        }
        if keycode == KeyCode::LShift
        {
            self.key_data.shift_down = false;
        }
    }
}

struct MouseData
{
    pub current_pos: Point2<f32>,
    pub left_button_down: bool,
    pub left_button_down_pos: Option<Point2<f32>>,
}

impl Default for MouseData
{
    fn default() -> MouseData
    {
        MouseData
        {
            current_pos: Point2{ x: 0f32, y: 0f32 },
            left_button_down: false,
            left_button_down_pos: None,
        }
    }
}

struct KeyData
{
    control_down: bool,
    shift_down: bool,
}

impl Default for KeyData
{
    fn default() -> KeyData
    {
        KeyData
        {
            control_down: false,
            shift_down: false,
        }
    }
}
