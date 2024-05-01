use fltk::{*, prelude::*};

use std::cell::RefCell;
use std::rc::Rc;
use std::collections::VecDeque;

const HISTORY_SIZE: usize = 80;

pub struct BarPlotWidget {
    inner: widget::Widget,
    min: Rc<RefCell<i64>>,
    max: Rc<RefCell<i64>>,
    history: Rc<RefCell<VecDeque<i64>>>,
}

impl BarPlotWidget {
    pub fn new() -> Self {
        let mut inner = widget::Widget::default();

        let min: i64 = 0;
        let max: i64 = 100;
        let history = VecDeque::<i64>::with_capacity(HISTORY_SIZE+1);

        let min = Rc::from(RefCell::from(min));
        let max = Rc::from(RefCell::from(max));
        let history = Rc::from(RefCell::from(history));

        inner.draw({
            let min = min.clone();
            let max = max.clone();
            let history = history.clone();
            move |i| {
                let min = *min.borrow();
                let max = *max.borrow();
                let history = history.borrow();

                const BG_COLOR: enums::Color = enums::Color::White;

                draw::push_clip(i.x(), i.y(), i.w(), i.h());
                
                let bg_color = if i.active() { BG_COLOR } else { enums::Color::Gray0 };
                draw::draw_rect_fill(i.x(), i.y(), i.w(), i.h(), bg_color);
                
                let range = (max - min) as f64;
                let dx = (i.w() as f64) / (HISTORY_SIZE as f64);

                if !history.is_empty() {
                    for k in 0..history.len() {
                        let n = history[k];
                        
                        let y = ((n - min) as f64) / range;
                        let yi = (100.0 * y) as i32;

                        const PALETTE: [(i32, enums::Color); 3] = [
                            (50, enums::Color::Red),
                            (85, enums::Color::from_rgb(255, 127, 0)),
                            (100, enums::Color::DarkGreen),
                        ];

                        let c: usize =
                            if yi < PALETTE[0].0 {
                                0
                            } else {
                                if yi < PALETTE[1].0 {
                                    1
                                } else {
                                    2
                                }
                            };

                        draw::set_draw_color(PALETTE[c].1);
                        draw::draw_polygon3(
                            draw::Coord::<i32>((i.x() as f64 + dx * (k as f64)) as i32, i.y() + ((i.h() as f64) * (1.0 - y)) as i32),
                            draw::Coord::<i32>((i.x() as f64 + dx * ((k + 1) as f64)) as i32 - 1, i.y() + ((i.h() as f64) * (1.0 - y)) as i32),
                            draw::Coord::<i32>((i.x() as f64 + dx * ((k + 1) as f64)) as i32 - 1, i.y() + i.h()),
                            draw::Coord::<i32>((i.x() as f64 + dx * (k as f64)) as i32, i.y() + i.h()));
                    }
                }

                draw::pop_clip();
            }
        });

        Self {
            inner,
            min, max,
            history,
        }
    }
    pub fn set_range(&mut self, min: i64, max: i64) {
        *self.min.borrow_mut() = min;
        *self.max.borrow_mut() = max;
    }
    pub fn push_value(&mut self, n: i64) {
        self.history.borrow_mut().push_back(n);
        if self.history.borrow().len() > HISTORY_SIZE {
            self.history.borrow_mut().pop_front();
        }
    }
    pub fn clear_history(&mut self) {
        self.history.borrow_mut().clear();
    }
}

widget_extends!(BarPlotWidget, widget::Widget, inner);
