use common::Color;

use gui::*;
use graphics::ImageIdentifier;

use backtrace::Backtrace;

#[derive(Clone, PartialEq)]
pub enum WidgetType {
    Text { text: String, font: Option<&'static str>, font_size: u32, wrap: bool, color: Color },
    Window { image: Option<String>, color: Color, border_width: u32, border_color: Color, margin_width: u32 },
}

impl WidgetType {
    pub fn text<S>(text: S, font_size: u32) -> WidgetType where S: Into<String> {
        WidgetType::Text { text: text.into(), font_size, font: None, wrap: false, color: Color::black() }
    }
    pub fn window(color: Color, border_width: u32) -> WidgetType {
        WidgetType::Window { image: None, color, border_width, border_color: Color::black(), margin_width: 0 }
    }
    pub fn image<S>(image : S, color : Color, border_width : u32) -> WidgetType where S : Into<String> {
        WidgetType::Window { image : Some(image.into()), color, border_width, border_color : Color::black(), margin_width: 0 }
    }
}


#[derive(Clone, PartialEq)]
pub struct Widget {
    pub id: Wid,
    pub parent_id: Option<Wid>,
    pub widget_type: WidgetType,
    pub size: [Sizing; 2],
    pub position: [Positioning; 2],
    pub alignment: [Alignment; 2],
    pub showing: bool,
}

impl Widget {
    pub fn new(widget_type: WidgetType) -> Widget {
        Widget {
            id: NO_WID,
            parent_id: None,
            widget_type,
            size: [Sizing::Constant(10.0.ux()), Sizing::Constant(10.0.ux())],
            position: [Positioning::Constant(0.0.ux()), Positioning::Constant(0.0.ux())],
            alignment: [Alignment::Top, Alignment::Left],
            showing: true,
        }
    }

    pub fn none() -> Widget {
        Widget::new(WidgetType::Window { image: None, color: Color::new(1.0, 1.0, 0.0, 1.0), border_width: 0, border_color: Color::clear(), margin_width: 0 })
    }

    pub fn widget_type(mut self, widget_type: WidgetType) -> Self {
        self.widget_type = widget_type;
        self
    }

    pub fn showing(mut self, showing: bool) -> Self {
        self.showing = showing;
        self
    }

    pub fn parent(mut self, parent: &Widget) -> Self {
        if parent.id == NO_WID {
            error!("Attempting to add a widget to a parent that has no ID, this is not acceptable");
        }
        self.parent_id = Some(parent.id);
        self
    }

    pub fn position(mut self, x: Positioning, y: Positioning) -> Self {
        self.position = [x, y];
        self
    }
    pub fn x(mut self, x: Positioning) -> Self {
        self.position[0] = x;
        self
    }
    pub fn y(mut self, y: Positioning) -> Self {
        self.position[1] = y;
        self
    }
    pub fn alignment(mut self, x: Alignment, y: Alignment) -> Self {
        self.alignment = [x, y];
        self
    }
    pub fn size(mut self, w: Sizing, h: Sizing) -> Self {
        self.size = [w, h];
        self
    }
    pub fn width(mut self, w: Sizing) -> Self {
        self.size[0] = w;
        self
    }
    pub fn height(mut self, h: Sizing) -> Self {
        self.size[1] = h;
        self
    }
    pub fn apply(mut self, gui: &mut GUI) -> Self {
        if !self.validate() {
            error!("Constructing invalid widget\n{:?}", Backtrace::new());
        }
        if self.id == NO_WID {
            self.id = gui.new_id();
        }

        gui.apply_widget(&self);

        self
    }

    pub fn dependent_on_children(&self) -> bool {
        Widget::sizing_dependent_on_children(self.size[0]) || Widget::sizing_dependent_on_children(self.size[1])
    }
    pub fn sizing_dependent_on_children(sizing: Sizing) -> bool {
        match sizing {
            Sizing::SurroundChildren => true,
            _ => false
        }
    }

    // -------------------------------------------------- private functions -----------------------------------------------------------
    fn parent_based_size(sizing: Sizing) -> bool {
        match sizing {
            Sizing::Constant(_) => false,
            Sizing::Derived => false,
            Sizing::DeltaOfParent(_) => true,
            Sizing::PcntOfParent(_) => true,
            Sizing::SurroundChildren => false
        }
    }
    fn parent_based_pos(pos: Positioning) -> bool {
        match pos {
            Positioning::PcntOfParent(_) => true,
            Positioning::CenteredInParent => true,
            Positioning::Constant(_) => false,
            Positioning::DeltaOfWidget(_, _, _) => false,
        }
    }
    fn validate(&self) -> bool {
//        if self.parent_id.is_none() {
//            if Widget::parent_based_size(self.size[0]) || Widget::parent_based_size(self.size[1]) ||
//                Widget::parent_based_pos(self.position[0]) || Widget::parent_based_pos(self.position[1]) {
//                return false;
//            }
//        }
        if self.alignment[0] == Alignment::Top || self.alignment[0] == Alignment::Bottom ||
            self.alignment[1] == Alignment::Left || self.alignment[1] == Alignment::Right {
            return false;
        }
        true
    }
}

impl Default for Widget {
    fn default() -> Self {
        Widget::none()
    }
}


#[derive(Default)]
pub struct Button {
    pub body: Widget,
    pub text: Widget,
}

impl Button {
    pub fn new<S>(text: S) -> Button where S: Into<String> {
        let body = Widget::new(WidgetType::Window {
            color: Color::white(),
            border_width: 2,
            border_color: Color::black(),
            image: Some(String::from("ui/blank")),
            margin_width: 0,
        }).size(Sizing::SurroundChildren, Sizing::SurroundChildren);

        Button {
            text: Widget::new(WidgetType::Text { text: text.into(), color: Color::black(), font: None, wrap: false, font_size: 14 })
                .size(Sizing::Derived, Sizing::Derived)
                .position(Positioning::Constant(0.px()), Positioning::Constant(0.px())),
            body,
        }
    }

    pub fn apply(mut self, gui: &mut GUI) -> Self {
        self.body = self.body.apply(gui);
        self.text = self.text.parent(&self.body).apply(gui);
        self
    }

    pub fn position(mut self, x: Positioning, y: Positioning) -> Self {
        self.body = self.body.position(x, y);
        self
    }

    pub fn parent(mut self, parent: &Widget) -> Self {
        self.body = self.body.parent(parent);
        self
    }

    pub fn width(mut self, w: Sizing) -> Self {
        self.body = self.body.width(w);
        self.text = match w {
            Sizing::SurroundChildren => self.text.x(Positioning::Constant(0.px())),
            _ => self.text.x(Positioning::CenteredInParent)
        };
        self
    }
}