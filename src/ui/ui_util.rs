use std::time::Duration;

use bevy::{ecs::system::EntityCommands, prelude::*};

pub const SMALL: f32 = 12.;
pub const MEDIUM: f32 = 16.;
pub const LARGE: f32 = 24.;

#[allow(non_snake_case)]
pub fn GREEN() -> Color { Color::rgb_u8(106, 190, 48) }
#[allow(non_snake_case)]
pub fn PURPLE() -> Color { Color::rgb_u8(69, 40, 60) }
#[allow(non_snake_case)]
pub fn RED() -> Color { Color::rgb_u8(172, 50, 50) }


#[derive(Default)]
pub struct StyleBuilder {
    style: Style,
}
#[allow(dead_code)]
impl StyleBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn from(style: &Style) -> Self {
        StyleBuilder {
            style: style.clone(),
        }
    }
    pub fn set_size(&mut self, height: Val, width: Val) -> &mut Self {
        self.style.height = height;
        self.style.width = width;
        self
    }
    pub fn set_margin(&mut self, rect: UiRect) -> &mut Self {
        self.style.margin = rect;
        self
    }
    pub fn build(&self) -> Style {
        self.style.clone()
    }
}

pub trait ProjectLocalStyle {
    fn local(font_size: f32, color: Color) -> TextStyle {
        TextStyle {
            font_size,
            color,
            ..default()
        }
    }
}
impl ProjectLocalStyle for TextStyle {}

pub trait UICommandsExtV<'w, 's, T: Component> {
    fn make_button(
        &mut self,
        text: &str,
        text_style: TextStyle,
        button_style: Style,
        background_color: Color,
        marker: T,
    ) -> EntityCommands<'w, 's, '_>;
}
impl<'w, 's, T: Component> UICommandsExtV<'w, 's, T> for Commands<'w, 's> {
    fn make_button(
        &mut self,
        text: &str,
        text_style: TextStyle,
        style: Style,
        background_color: Color,
        marker: T,
    ) -> EntityCommands<'w, 's, '_> {
        let mut ec = self.spawn((
            ButtonBundle {
                background_color: background_color.into(),
                style,
                ..default()
            },
            marker,
        ));
        ec.with_children(|c| {
            c.spawn(TextBundle {
                text: Text::from_section(text, text_style),
                ..default()
            });
        });

        return ec;
    }
}
pub trait UICommandsExt<'w, 's> {
    fn make_text(&mut self, text: &str, text_style: TextStyle) -> EntityCommands<'w, 's, '_>;
    fn make_text_sections(
        &mut self,
        sections: Vec<(&str, TextStyle)>,
    ) -> EntityCommands<'w, 's, '_>;
    fn make_icon(&mut self, icon_file: String) -> Entity;
}
impl<'w, 's> UICommandsExt<'w, 's> for Commands<'w, 's> {
    fn make_text(&mut self, text: &str, text_style: TextStyle) -> EntityCommands<'w, 's, '_> {
        self.spawn(TextBundle {
            text: Text::from_section(text, text_style),
            ..default()
        })
    }
    fn make_text_sections(
        &mut self,
        sections: Vec<(&str, TextStyle)>,
    ) -> EntityCommands<'w, 's, '_> {
        let sec: Vec<TextSection> = sections
            .iter()
            .map(|(s, st)| TextSection::new(*s, st.clone()))
            .collect();
        self.spawn(TextBundle::from_sections(sec))
    }

    fn make_icon(&mut self, icon_file: String) -> Entity {
        let ec = self.spawn_empty();
        let id = ec.id().clone();
        self.add(move |world: &mut World| {
            world.resource_scope(|world, asset_server: Mut<AssetServer>| {
                let handle = asset_server.load(icon_file);
                world.entity_mut(id).insert(ImageBundle {
                    image: UiImage {
                        texture: handle,
                        ..default()
                    },
                    ..default()
                });
            })
        });
        id
    }
}
pub trait UIEntityCommandsExt<'w, 's, 'a> {
    fn make_icon(&mut self, icon_file: String) -> Entity;
}
impl<'w, 's, 'a> UIEntityCommandsExt<'w, 's, 'a> for EntityCommands<'w, 's, 'a> {
    fn make_icon(&mut self, icon_file: String) -> Entity {
        let ec = self.commands().spawn_empty();
        let id = ec.id().clone();
        self.commands().add(move |world: &mut World| {
            world.resource_scope(|world, asset_server: Mut<AssetServer>| {
                let handle = asset_server.load(icon_file);
                world.entity_mut(id).insert(ImageBundle {
                    image: UiImage {
                        texture: handle,
                        ..default()
                    },
                    ..default()
                });
            })
        });
        id
    }
}
pub trait UIChildBuilderExt<'w, 's, 'a> {
    fn with_icon(&mut self, icon_file: String);
}
impl<'w, 's, 'a> UIChildBuilderExt<'w, 's, 'a> for ChildBuilder<'w, 's, 'a> {
    fn with_icon(&mut self, icon_file: String) {
        let id = self.parent_entity();
        self.add_command(move |world: &mut World| {
            world.resource_scope(|world, asset_server: Mut<AssetServer>| {
                let handle = asset_server.load(icon_file);
                world.entity_mut(id).insert(ImageBundle {
                    image: UiImage {
                        texture: handle,
                        ..default()
                    },
                    ..default()
                });
            })
        });
    }
}

pub const ALL: Val = Val::Percent(100.);

pub fn into_pct(v: f32) -> Val {
    Val::Percent(v * 100.)
}
pub fn px(v: f32) -> Val {
    Val::Px(v)
}


pub struct CoolDown {
    cooling_down: bool,
    elapsed: Timer,
}
impl Default for CoolDown {
    fn default() -> Self {
        Self {
            cooling_down: false,
            elapsed: Timer::from_seconds(0.25, TimerMode::Once),
        }
    }
}
impl CoolDown {
   pub  fn cooling_down(&self) -> bool {
        self.cooling_down
    }

    pub fn start_cooldown(&mut self) {
        self.cooling_down = true;
    }

    pub fn handle_time(&mut self, delta: Duration) {
        match (self.cooling_down, self.done()) {
            (false, _) => {
                return;
            }
            (true, false) => {
                self.tick(delta);
            }
            (true, true) => {
                self.clear();
            }
        }
    }
    fn done(&self) -> bool {
        self.elapsed.finished()
    }
    fn tick(&mut self, delta: Duration) {
        self.elapsed.tick(delta);
    }
    fn clear(&mut self) {
        self.cooling_down = false;
        self.elapsed.reset();
    }
}