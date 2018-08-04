use backtrace::Backtrace;
use common::Color;
use common::prelude::*;
use events::EventPosition;
use events::ui_event_types;
use events::UIEvent;
use events::UIEventType;
use events::WidgetEvent;
use graphics::ImageIdentifier;
use gui::*;
use itertools::Itertools;
use piston_window::keyboard;
use piston_window::MouseButton;
use std::any::Any;
use std::any::TypeId;
use std::collections::HashMap;
use ui_event_types::*;
use widgets::*;
use widget_delegation::DelegateToWidget;

#[derive(Default)]
pub struct TextDisplayWidget {
    pub body: Widget,
    pub text: Widget,
}

impl WidgetContainer for TextDisplayWidget {
    fn for_all_widgets<F: FnMut(&mut Widget)>(&mut self, mut func: F) {
        (func)(&mut self.body);
        (func)(&mut self.text);
    }
}

impl DelegateToWidget for TextDisplayWidget {
    fn as_widget(&mut self) -> &mut Widget { &mut self.body }
    fn as_widget_immut(&self) -> &Widget { &self.body }
}

impl TextDisplayWidget {
    pub fn new<S>(text: S, font_size : u32, image: Option<ImageIdentifier>, segment: ImageSegmentation) -> TextDisplayWidget where S: Into<String> {
        let body = Widget::new(WidgetType::Window { image, segment })
            .border_width(1)
            .border_color(Color::black())
            .size(Sizing::SurroundChildren, Sizing::SurroundChildren)
            .margin(0.5.ux());

        let text = Widget::text(text, font_size)
            .size(Sizing::Derived, Sizing::Derived)
            .parent(&body);

        TextDisplayWidget { body, text }
    }

    pub fn wrapped(mut self, w : Sizing) -> Self {
        self.body.set_width(w);
        self.text.widget_type.set_text_wrap(Some(TextWrap::WithinParent));
        self
    }
}