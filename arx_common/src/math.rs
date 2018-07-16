use num::Num;
use cgmath::Vector2;

#[derive(Debug)]
pub struct Rect<T : Num + Copy> {
    pub x : T,
    pub y : T,
    pub w : T,
    pub h : T
}

impl <T : Num + Copy + PartialOrd> Rect<T> {
    pub fn new(x : T, y : T, w : T, h : T) -> Rect<T> {
        Rect {
            x,y,w,h
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