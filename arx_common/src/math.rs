use num::Num;
use cgmath::Vector2;
use std::fmt::Debug;

#[derive(Debug, Clone, Copy)]
pub struct Rect<T : Num + Copy> {
    pub x : T,
    pub y : T,
    pub w : T,
    pub h : T
}

impl <T : Num + Copy + PartialOrd + Debug> Rect<T> {
    pub fn new(x : T, y : T, w : T, h : T) -> Rect<T> {
        Rect {
            x,y,w,h
        }
    }

    pub fn from_corners(x : T, y : T, x2 : T, y2 : T) -> Rect<T> {
        Rect {
            x,y,w : x2 - x,h : y2 - y
        }
    }

    pub fn from_corners_v(p1 : Vector2<T>, p2 : Vector2<T>) -> Rect<T> {
        Rect {
            x : p1.x, y : p1.y, w: p2.x - p1.x, h: p2.y - p1.y
        }
    }

    fn min_v(a : T, b : T) -> T {
        if a < b {
            a
        } else {
            b
        }
    }
    fn max_v(a : T, b : T) -> T {
        if a > b {
            a
        } else {
            b
        }
    }


    pub fn enclosing_both(r1 : Rect<T>, r2 : Rect<T>) -> Rect<T>{
        let min_x = Rect::min_v(r1.x,r2.x);
        let min_y = Rect::min_v(r1.y,r2.y);
        Rect {
            x : min_x,
            y : min_y,
            w : Rect::max_v(r1.max_x(), r2.max_x()) - min_x,
            h : Rect::max_v(r1.max_y(), r2.max_y()) - min_y
        }
    }

    pub fn intersect(&self, r2: Rect<T>) -> Option<Rect<T>> {
        let x = Rect::max_v(self.x, r2.x);
        let y = Rect::max_v(self.y, r2.y);
        let max_x = Rect::min_v(self.max_x(), r2.max_x());
        let max_y = Rect::min_v(self.max_y(), r2.max_y());

        let w = max_x - x;
        let h = max_y - y;
        if w > T::zero() && h > T::zero() {
            Some(Rect::new(x,y,w,h))
        } else {
            None
        }
    }

    pub fn translated(&self, dx : T, dy : T) -> Rect<T> {
        Rect::new(self.x + dx, self.y + dy, self.w, self.h)
    }
    pub fn resized(&self, w : T, h : T) -> Rect<T> {
        Rect::new(self.x, self.y, w, h)
    }

    pub fn dimensions(&self) -> Vector2<T> {
        Vector2 {
            x : self.w,
            y : self.h
        }
    }

    pub fn position(&self) -> Vector2<T> {
        Vector2 {
            x : self.x,
            y : self.y
        }
    }

    pub fn contains(&self, v : Vector2<T>) -> bool {
        self.x <= v.x && self.y <= v.y && self.max_x() >= v.x && self.max_y() >= v.y
    }

    pub fn max_x(&self) -> T {
        self.x + self.w
    }
    pub fn max_y(&self) -> T {
        self.y + self.h
    }
    pub fn min_x(&self) -> T {
        self.x
    }
    pub fn min_y(&self) -> T {
        self.y
    }

    pub fn max(&self) -> Vector2<T> {
        Vector2 {
            x : self.max_x(),
            y : self.max_y()
        }
    }
}