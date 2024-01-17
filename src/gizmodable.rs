use bevy::{math::Vec3Swizzles, prelude::*, utils::HashMap};

pub struct Gizmotastic;

impl Plugin for Gizmotastic {
    fn build(&self, app: &mut App) {
        app.insert_resource::<DebugGizmosOn>(DebugGizmosOn {
            root: false,
            debug_systems: HashMap::new(),
        })
        .configure_sets(
            Update,
            (
                GizmoSystemSet::GizmoQueueDraw.run_if(debug_gizmos_enabled),
                GizmoSystemSet::GizmoClear,
            )
                .chain(),
        )
        .register_type::<DebugGizmosOn>()
        .add_systems(
            Update,
            (
                render_gizmos.in_set(GizmoSystemSet::GizmoQueueDraw),
                clear_gizmos.in_set(GizmoSystemSet::GizmoClear),
            ),
        );
    }
}

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct DebugGizmosOn {
    root: bool,
    debug_systems: HashMap<String, bool>,
}
#[allow(dead_code)]
impl DebugGizmosOn {
    pub fn render_enabled(&self) -> bool {
        self.root
    }
    pub fn system_enabled(&self, k: &str) -> bool {
        self.root && self.debug_systems.get(k).is_some()
    }
    pub fn register_system(&mut self, k: &str) {
        self.debug_systems.insert(k.clone().to_string(), false);
    }
}
fn debug_gizmos_enabled(g: Res<DebugGizmosOn>) -> bool {
    g.render_enabled()
}

#[derive(SystemSet, Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub enum GizmoSystemSet {
    GizmoQueueDraw,
    GizmoClear,
}

#[derive(Component)]

pub struct CircleArgs {
    position: Vec2,
    radius: f32,
    color: Color,
}
pub struct LineArgs {
    start: Vec2,
    end: Vec2,
    color: Color,
}
pub struct RectArgs {
    position: Vec2,
    rotation: f32,
    size: Vec2,
    color: Color,
}
#[derive(Component)]
pub struct VisualDebug {
    ops: Vec<GizmoDrawOp>,
    persistent_op: Option<GizmoDrawOp>,
}
impl Default for VisualDebug {
    fn default() -> Self {
        VisualDebug {
            ops: Vec::new(),
            persistent_op: None,
        }
    }
}
#[allow(dead_code)]
impl VisualDebug {
    pub fn clear(&mut self) {
        self.ops = Vec::new();
    }
    pub fn add(&mut self, op: GizmoDrawOp) {
        self.ops.push(op);
    }
    pub fn register_persistent(&mut self, op: GizmoDrawOp) {
        self.persistent_op = Some(op);
    }
    pub fn clear_persistent(&mut self) {
        self.persistent_op = None;
    }
    pub fn from_persistent(op: GizmoDrawOp) -> Self {
        VisualDebug {
            ops: Vec::new(),
            persistent_op: Some(op),
        }
    }
}
#[allow(dead_code)]
pub enum GizmoDrawOp {
    Circle(CircleArgs),
    Line(LineArgs),
    Rect(RectArgs),
}
#[allow(dead_code)]
impl GizmoDrawOp {
    pub fn circle(position: Vec2, radius: f32, color: Color) -> Self {
        GizmoDrawOp::Circle(CircleArgs {
            position,
            radius,
            color,
        })
    }
    pub fn line(start: Vec2, end: Vec2, color: Color) -> Self {
        GizmoDrawOp::Line(LineArgs { start, end, color })
    }
    pub fn rect(position: Vec2, rotation: f32, size: Vec2, color: Color) -> Self {
        GizmoDrawOp::Rect(RectArgs {
            position,
            rotation,
            size,
            color,
        })
    }
}

fn render_gizmos(mut gizmos: Gizmos, q: Query<(&VisualDebug, Option<&GlobalTransform>)>) {
    for (dbg, has_transform) in q.iter() {
        if let Some(op) = &dbg.persistent_op {
            let base = match has_transform {
                Some(transform) => transform.translation().xy(),
                None => Vec2::ZERO,
            };
            match op {
                GizmoDrawOp::Circle(args) => {
                    gizmos.circle_2d(args.position + base, args.radius, args.color);
                }
                GizmoDrawOp::Line(args) => gizmos.line_2d(args.start + base, args.end, args.color),
                GizmoDrawOp::Rect(args) => {
                    gizmos.rect_2d(args.position + base, args.rotation, args.size, args.color)
                }
            }
        }
        for op in dbg.ops.iter() {
            match &op {
                GizmoDrawOp::Circle(args) => {
                    gizmos.circle_2d(args.position, args.radius, args.color);
                }
                GizmoDrawOp::Line(args) => gizmos.line_2d(args.start, args.end, args.color),
                GizmoDrawOp::Rect(args) => {
                    gizmos.rect_2d(args.position, args.rotation, args.size, args.color)
                }
            }
        }
    }
}
fn clear_gizmos(mut q: Query<&mut VisualDebug>) {
    for mut dbg in q.iter_mut() {
        dbg.clear();
    }
}
