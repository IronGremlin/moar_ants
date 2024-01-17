use bevy::{ecs::system::EntityCommands, prelude::*};

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

#[derive(Default)]
pub struct TextStyleBuilder {
    style: TextStyle,
}
#[allow(dead_code)]
impl TextStyleBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn from(style: &TextStyle) -> Self {
        TextStyleBuilder {
            style: style.clone(),
        }
    }
    pub fn set_size(&mut self, size: f32) -> &mut Self {
        self.style.font_size = size;
        self
    }
    pub fn set_color(&mut self, color: Color) -> &mut Self {
        self.style.color = color.into();
        self
    }
    pub fn build(&self) -> TextStyle {
        self.style.clone()
    }
}

pub trait UICommandsExt<'w, 's, T: Component> {
    fn make_button(
        &mut self,
        text: &str,
        text_style: TextStyle,
        button_style: Style,
        background_color: Color,
        marker: T,
    ) -> EntityCommands<'w, 's, '_>;
    fn make_text(
        &mut self,
        text: &str,
        text_style: TextStyle,
        marker: Option<T>,
    ) -> EntityCommands<'w, 's, '_>;
}
impl<'w, 's, T: Component> UICommandsExt<'w, 's, T> for Commands<'w, 's> {
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
    fn make_text(
        &mut self,
        text: &str,
        text_style: TextStyle,
        marker: Option<T>,
    ) -> EntityCommands<'w, 's, '_> {
        let mut ec = self.spawn(TextBundle {
            text: Text::from_section(text, text_style),
            ..default()
        });
        match marker {
            Some(x) => {
                ec.insert(x);
            }
            _ => {}
        }
        ec
    }
}
