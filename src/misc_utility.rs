use bevy::math::{Vec2, Vec3};

pub trait NaNGuard {
    fn nan_guard(&self, default: Self) -> Self;
    fn if_nan(&mut self, default: Self);
}

impl NaNGuard for f32 {
    fn nan_guard(&self, default: f32) -> f32 {
        if self.is_nan() {
            default
        } else {
            *self
        }
    }

    fn if_nan(&mut self, default: Self) {
        if self.is_nan() {
            *self = default;
        }
    }
}

impl NaNGuard for Vec2 {
    fn nan_guard(&self, default: Vec2) -> Vec2 {
        if self.is_nan() {
            default
        } else {
            *self
        }
    }
    fn if_nan(&mut self, default: Self) {
        if self.is_nan() {
            *self = default;
        }
    }
}

impl NaNGuard for Vec3 {
    fn nan_guard(&self, default: Vec3) -> Vec3 {
        if self.is_nan() {
            default
        } else {
            *self
        }
    }
    fn if_nan(&mut self, default: Self) {
        if self.is_nan() {
            *self = default;
        }
    }
}
