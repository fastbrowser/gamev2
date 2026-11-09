//! Minimal immediate-mode UI helpers backed by macroquad.
//!
//! Widgets are measured from their contents, so rows and columns size
//! themselves automatically and can be centered on screen.

use std::cell::Cell;

use macroquad::prelude::*;

/// Screen and pointer state for the current frame.
pub struct Ui {
    pub screen: Rect,
    pub mouse: Vec2,
    click_consumed: Cell<bool>,
}

impl Ui {
    pub fn new() -> Self {
        let (mx, my) = mouse_position();
        Self {
            screen: Rect::new(0.0, 0.0, screen_width(), screen_height()),
            mouse: vec2(mx, my),
            click_consumed: Cell::new(false),
        }
    }

    /// Reports a click exactly once per frame.
    ///
    /// The first hovered widget drawn in a frame wins; any other widget drawn
    /// later in the same frame sees `false`, even if it overlaps.
    pub fn take_click(&self, hovered: bool) -> bool {
        if !hovered || self.click_consumed.get() {
            return false;
        }
        if is_mouse_button_pressed(MouseButton::Left) {
            self.click_consumed.set(true);
            true
        } else {
            false
        }
    }

    /// Fills the whole screen, useful for dimming the game behind a menu.
    pub fn overlay(&self, color: Color) {
        draw_rectangle(self.screen.x, self.screen.y, self.screen.w, self.screen.h, color);
    }

    pub fn button<'a>(&self, label: &'a str, rect: Rect) -> Button<'a> {
        Button::new(label, rect)
    }

    pub fn centered_button<'a>(&self, label: &'a str, width: f32, height: f32) -> Button<'a> {
        Button::centered(label, self.screen.center(), vec2(width, height))
    }

    /// Draws any widget centered on screen with its measured size.
    pub fn centered<'a, E: Into<Element<'a>>>(&self, element: E) -> Option<&'a str> {
        let element: Element<'a> = element.into();
        let size = element.size();
        element.draw_at(self, vec2(
            (self.screen.w - size.x) * 0.5,
            (self.screen.h - size.y) * 0.5,
        ))
    }

    pub fn centered_row<'a>(&self, row: Row<'a>) -> Option<&'a str> {
        self.centered(row)
    }

    pub fn centered_column<'a>(&self, column: Column<'a>) -> Option<&'a str> {
        self.centered(column)
    }

    pub fn row<'a>(&self, row: Row<'a>, pos: Vec2) -> Option<&'a str> {
        row.draw_at(self, pos)
    }

    pub fn column<'a>(&self, column: Column<'a>, pos: Vec2) -> Option<&'a str> {
        column.draw_at(self, pos)
    }
}

/// Anything that can be placed in a layout.
pub enum Element<'a> {
    Button(Button<'a>),
    Text(Text<'a>),
    Row(Row<'a>),
    Column(Column<'a>),
}

impl<'a> Element<'a> {
    pub fn size(&self) -> Vec2 {
        match self {
            Element::Button(button) => button.size(),
            Element::Text(text) => text.size(),
            Element::Row(row) => row.size(),
            Element::Column(column) => column.size(),
        }
    }

    pub fn draw_at(&self, ui: &Ui, pos: Vec2) -> Option<&'a str> {
        match self {
            Element::Button(button) => button.draw_at(ui, pos),
            Element::Text(text) => {
                text.draw_at(pos);
                None
            }
            Element::Row(row) => row.draw_at(ui, pos),
            Element::Column(column) => column.draw_at(ui, pos),
        }
    }
}

impl<'a> From<Button<'a>> for Element<'a> {
    fn from(button: Button<'a>) -> Self { Element::Button(button) }
}
impl<'a> From<Text<'a>> for Element<'a> {
    fn from(text: Text<'a>) -> Self { Element::Text(text) }
}
impl<'a> From<Row<'a>> for Element<'a> {
    fn from(row: Row<'a>) -> Self { Element::Row(row) }
}
impl<'a> From<Column<'a>> for Element<'a> {
    fn from(column: Column<'a>) -> Self { Element::Column(column) }
}

/// A simple rectangular text button.
pub struct Button<'a> {
    pub id: &'a str,
    pub label: &'a str,
    pub rect: Rect,
    pub font_size: f32,
    pub hover_scale: f32,
    pub color: Color,
    pub hover_color: Option<Color>,
    pub text_color: Color,
}

impl<'a> Button<'a> {
    pub fn new(label: &'a str, rect: Rect) -> Self {
        Self {
            id: label,
            label,
            rect,
            font_size: 20.0,
            hover_scale: 1.0,
            color: GRAY,
            hover_color: None,
            text_color: WHITE,
        }
    }

    pub fn centered(label: &'a str, center: Vec2, size: Vec2) -> Self {
        Self::new(label, Rect::new(
            center.x - size.x * 0.5,
            center.y - size.y * 0.5,
            size.x,
            size.y,
        ))
    }

    pub fn id(mut self, id: &'a str) -> Self {
        self.id = id;
        self
    }

    pub fn font_size(mut self, font_size: f32) -> Self {
        self.font_size = font_size;
        self
    }

    pub fn hover_scale(mut self, hover_scale: f32) -> Self {
        self.hover_scale = hover_scale;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn hover_color(mut self, color: Color) -> Self {
        self.hover_color = Some(color);
        self
    }

    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = color;
        self
    }

    pub fn size(&self) -> Vec2 {
        vec2(self.rect.w, self.rect.h)
    }

    /// Draws the button at its own rect and reports its id if clicked.
    ///
    /// Clicks are consumed through [`Ui::take_click`], so only one widget can
    /// report a click per frame.
    pub fn draw(&self, ui: &Ui) -> Option<&'a str> {
        self.draw_at(ui, self.rect.point())
    }

    /// Draws the button at `pos`, ignoring its own `rect` origin.
    pub fn draw_at(&self, ui: &Ui, pos: Vec2) -> Option<&'a str> {
        let base = Rect::new(pos.x, pos.y, self.rect.w, self.rect.h);
        let hovered = base.contains(ui.mouse);
        let rect = if hovered {
            let center = base.center();
            Rect::new(
                center.x - base.w * self.hover_scale * 0.5,
                center.y - base.h * self.hover_scale * 0.5,
                base.w * self.hover_scale,
                base.h * self.hover_scale,
            )
        } else {
            base
        };
        let font_size = if hovered { self.font_size * self.hover_scale } else { self.font_size };
        let background = if hovered { self.hover_color.unwrap_or(self.color) } else { self.color };
        draw_rectangle(rect.x, rect.y, rect.w, rect.h, background);
        let dims = measure_text(self.label, None, font_size as u16, 1.0);
        draw_text(
            self.label,
            rect.x + rect.w * 0.5 - dims.width * 0.5,
            dims.offset_y + rect.y + rect.h * 0.5 - dims.height * 0.5,
            font_size,
            self.text_color,
        );
        if ui.take_click(hovered) {
            Some(self.id)
        } else {
            None
        }
    }
}

/// A single line of text.
pub struct Text<'a> {
    pub label: &'a str,
    pub font_size: f32,
    pub color: Color,
}

impl<'a> Text<'a> {
    pub fn new(label: &'a str) -> Self {
        Self { label, font_size: 20.0, color: WHITE }
    }

    pub fn font_size(mut self, font_size: f32) -> Self {
        self.font_size = font_size;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn size(&self) -> Vec2 {
        if self.label.contains('\n') {
            let dims = measure_multiline_text(self.label, None, self.font_size as u16, 1.0, None);
            vec2(dims.width, dims.height)
        } else {
            let dims = measure_text(self.label, None, self.font_size as u16, 1.0);
            vec2(dims.width, dims.height)
        }
    }

    pub fn draw_at(&self, pos: Vec2) {
        if self.label.contains('\n') {
            let first_line = self.label.lines().next().unwrap_or("");
            let first = measure_text(first_line, None, self.font_size as u16, 1.0);
            draw_multiline_text(
                self.label,
                pos.x,
                pos.y + first.offset_y,
                self.font_size,
                None,
                self.color,
            );
        } else {
            let dims = measure_text(self.label, None, self.font_size as u16, 1.0);
            draw_text(self.label, pos.x, pos.y + dims.offset_y, self.font_size, self.color);
        }
    }
}

/// Lays widgets out left-to-right, centering them on the cross axis.
pub struct Row<'a> {
    pub items: Vec<Element<'a>>,
    pub gap: f32,
    pub padding: f32,
}

impl<'a> Row<'a> {
    pub fn new<I>(items: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<Element<'a>>,
    {
        Self { items: items.into_iter().map(Into::into).collect(), gap: 8.0, padding: 0.0 }
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    pub fn size(&self) -> Vec2 {
        let mut width = 0.0;
        let mut height: f32 = 0.0;
        for (index, item) in self.items.iter().enumerate() {
            let size = item.size();
            width += size.x;
            height = height.max(size.y);
            if index > 0 {
                width += self.gap;
            }
        }
        vec2(width + self.padding * 2.0, height + self.padding * 2.0)
    }

    pub fn draw_at(&self, ui: &Ui, pos: Vec2) -> Option<&'a str> {
        let inner_height = self.size().y - self.padding * 2.0;
        let mut x = pos.x + self.padding;
        let mut clicked = None;
        for item in &self.items {
            let size = item.size();
            let y = pos.y + self.padding + (inner_height - size.y) * 0.5;
            clicked = clicked.or(item.draw_at(ui, vec2(x, y)));
            x += size.x + self.gap;
        }
        clicked
    }
}

/// Lays widgets out top-to-bottom, centering them on the cross axis.
pub struct Column<'a> {
    pub items: Vec<Element<'a>>,
    pub gap: f32,
    pub padding: f32,
}

impl<'a> Column<'a> {
    pub fn new<I>(items: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<Element<'a>>,
    {
        Self { items: items.into_iter().map(Into::into).collect(), gap: 8.0, padding: 0.0 }
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    pub fn size(&self) -> Vec2 {
        let mut width: f32 = 0.0;
        let mut height = 0.0;
        for (index, item) in self.items.iter().enumerate() {
            let size = item.size();
            width = width.max(size.x);
            height += size.y;
            if index > 0 {
                height += self.gap;
            }
        }
        vec2(width + self.padding * 2.0, height + self.padding * 2.0)
    }

    pub fn draw_at(&self, ui: &Ui, pos: Vec2) -> Option<&'a str> {
        let inner_width = self.size().x - self.padding * 2.0;
        let mut y = pos.y + self.padding;
        let mut clicked = None;
        for item in &self.items {
            let size = item.size();
            let x = pos.x + self.padding + (inner_width - size.x) * 0.5;
            clicked = clicked.or(item.draw_at(ui, vec2(x, y)));
            y += size.y + self.gap;
        }
        clicked
    }
}
