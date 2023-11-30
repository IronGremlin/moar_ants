use std::sync::Arc;

use bevy::{gizmos, prelude::*, render::camera};

use crate::MainCamera;

pub struct Gizmodable;

impl Plugin for Gizmodable {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, render_gizmos);
    }
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
    op: Option<GizmoDrawOp>,
}
impl Default for VisualDebug {
    fn default() -> Self {
        VisualDebug { op: None }
    }
}
impl VisualDebug {
    pub fn circle(position: Vec2, radius: f32, color: Color) -> Self {
        VisualDebug {
            op: Some(GizmoDrawOp::Circle(CircleArgs {
                position,
                radius,
                color,
            })),
        }
    }
    pub fn line(start: Vec2, end: Vec2, color: Color) -> Self {
        VisualDebug {
            op: Some(GizmoDrawOp::Line(LineArgs { start, end, color })),
        }
    }
    pub fn rect(position: Vec2, rotation: f32, size: Vec2, color: Color) -> Self {
        VisualDebug {
            op: Some(GizmoDrawOp::Rect(RectArgs {
                position,
                rotation,
                size,
                color,
            })),
        }
    }
    pub fn clear() -> Self {
        Self::default()
    }
}
pub enum GizmoDrawOp {
    Circle(CircleArgs),
    Line(LineArgs),
    Rect(RectArgs),
}

fn render_gizmos(mut gizmos: Gizmos, q: Query<&VisualDebug>) {
    for op in q.iter() {
        match &op.op {
            Some(GizmoDrawOp::Circle(args)) => { gizmos.circle_2d(args.position, args.radius, args.color);},
            Some(GizmoDrawOp::Line(args)) => gizmos.line_2d(args.start, args.end, args.color),
            Some(GizmoDrawOp::Rect(args)) => gizmos.rect_2d(args.position, args.rotation, args.size, args.color),
            _ => {}
        }
    }
}

