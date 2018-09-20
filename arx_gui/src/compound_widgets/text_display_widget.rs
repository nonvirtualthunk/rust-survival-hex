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
use events::MouseButton;
use std::any::Any;
use std::any::TypeId;
use std::collections::HashMap;
use ui_event_types::*;
use widgets::*;
use widget_delegation::DelegateToWidget;
use graphics::FontSize;

#[derive(Default, Clone)]
pub struct TextDisplayWidget {
    pub body: Widget,
    pub text: Widget,
}

impl WidgetContainer for TextDisplayWidget {
    fn for_each_widget<F: FnMut(&mut Widget)>(&mut self, mut func: F) {
        (func)(&mut self.body);
        (func)(&mut self.text);
    }
}

impl DelegateToWidget for TextDisplayWidget {
    fn as_widget(&mut self) -> &mut Widget { &mut self.body }
    fn as_widget_immut(&self) -> &Widget { &self.body }
}

impl TextDisplayWidget {
    pub fn new<S, I>(text: S, font_size : FontSize, image: I, segment: ImageSegmentation) -> TextDisplayWidget where S: Into<String>, I : OptionalStringArg {
        let body = Widget::new(WidgetType::Window { image : image.into_string_opt(), segment })
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

    pub fn reapply(&mut self, gui : &mut GUI) -> &mut Self {
        self.body.reapply(gui);
        self.text.reapply(gui);
        self
    }

    pub fn centered_text(mut self) -> Self {
        self.center_text_horizontally();
        self
    }

    pub fn center_text_horizontally(&mut self) -> &mut Self {
        self.text.set_x(Positioning::CenteredInParent);
        self
    }

    pub fn set_text<S : Into<String>>(&mut self, text : S) -> &mut Self {
        self.text.set_text(text);
        self
    }

    pub fn set_font_size(&mut self, font_size : FontSize) -> &mut Self {
        self.text.set_font_size(font_size);
        self
    }
}