use fltk::{*, prelude::*};

use std::cell::RefCell;
use std::rc::Rc;
use std::collections::VecDeque;

use crate::bandwidth_utils::*;

const HISTORY_SIZE: usize = 80;

const BG_COLOR: enums::Color = enums::Color::White;
const BORDER_COLOR: enums::Color = enums::Color::Dark2;

/*
 * BarPlotWidget
 */
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
        let history = VecDeque::<i64>::with_capacity(HISTORY_SIZE);

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

                draw::push_clip(i.x(), i.y(), i.w(), i.h());
                
                let bg_color = if i.active() { BG_COLOR } else { enums::Color::Gray0 };
                draw::draw_rect_fill(i.x(), i.y(), i.w(), i.h(), bg_color);
                
                let range = (max - min) as f64;
                let dx = ((i.w() - 2) as f64) / (HISTORY_SIZE as f64);

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

                        let x1 = (i.x() as f64 + dx * (k as f64)) as i32 + 1;
                        let x2 = (i.x() as f64 + dx * ((k + 1) as f64)) as i32 + 1;

                        let y0 = i.y() + i.h() - 1;
                        let y = i.y() + (((i.h() - 2) as f64) * (1.0 - y)) as i32 - 1;

                        draw::set_draw_color(PALETTE[c].1);
                        draw::draw_polygon3(
                            draw::Coord::<i32>(x1, y),
                            draw::Coord::<i32>(x2 - 1, y),
                            draw::Coord::<i32>(x2 - 1, y0),
                            draw::Coord::<i32>(x1, y0));
                    }

                    draw::draw_rect_with_color(i.x(), i.y(), i.w(), i.h(), BORDER_COLOR);
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
        if self.history.borrow().len() == HISTORY_SIZE {
            self.history.borrow_mut().pop_front();
        }
        self.history.borrow_mut().push_back(n);
    }
    pub fn clear_history(&mut self) {
        self.history.borrow_mut().clear();
    }
}

widget_extends!(BarPlotWidget, widget::Widget, inner);

/*
 * DlUlBarPlotWidget
 */

pub struct DlUlBarPlotWidget {
    inner: widget::Widget,
    history: Rc<RefCell<VecDeque<(i64, i64)>>>,
}

impl DlUlBarPlotWidget {
    pub fn new() -> Self {
        let mut inner = widget::Widget::default();

        let history = VecDeque::<(i64, i64)>::with_capacity(HISTORY_SIZE+1);

        let history = Rc::from(RefCell::from(history));

        inner.draw({
            let history = history.clone();
            move |i| {
                let history = history.borrow();

                draw::push_clip(i.x(), i.y(), i.w(), i.h());
                
                let bg_color = if i.active() { BG_COLOR } else { enums::Color::Gray0 };
                draw::draw_rect_fill(i.x(), i.y(), i.w(), i.h(), bg_color);
                
                let dx = ((i.w() - 2) as f64) / (HISTORY_SIZE as f64);

                if !history.is_empty() {
                    let h = history.clone();

                    let max_dlul: (i64, i64) = h.into_iter().reduce( |accum, current| {
                            (accum.0.max(current.0), accum.1.max(current.1))
                        }).unwrap();
                    let max_dlul = max_dlul.0.max(max_dlul.1);

                    let max_plot = nearest_fib(max_dlul / SIZE_MB);
                    let max_mb = max_plot * SIZE_MB;
                        
                    for k in 0..history.len() {
                        let (dl, ul) = history[k];

                        let ydl = (dl as f64) / (max_mb as f64);
                        let yul = (ul as f64) / (max_mb as f64);

                        const COLOR_DL: enums::Color = enums::Color::from_hex(0x332288);
                        const COLOR_UL: enums::Color = enums::Color::from_hex(0x88CCEE);
                        const COLOR_BOTH: enums::Color = enums::Color::from_hex(0xDDCC77);

                        let color1 = COLOR_BOTH;
                        let (y1, y2, color2) = {
                            if ul < dl {
                                (yul, ydl, COLOR_DL)
                            }
                            else {
                                (ydl, yul, COLOR_UL)
                            }
                        };

                        let x1 = (i.x() as f64 + dx * (k as f64)) as i32 + 1;
                        let x2 = (i.x() as f64 + dx * ((k + 1) as f64)) as i32 + 1;

                        let y0 = i.y() + i.h() - 1;
                        let y1 = i.y() + (((i.h() - 2) as f64) * (1.0 - y1)) as i32 - 1;
                        let y2 = i.y() + (((i.h() - 2) as f64) * (1.0 - y2)) as i32 - 1;

                        draw::set_draw_color(color1);
                        draw::draw_polygon3(
                            draw::Coord::<i32>(x1, y1),
                            draw::Coord::<i32>(x2 - 1, y1),
                            draw::Coord::<i32>(x2 - 1, y0),
                            draw::Coord::<i32>(x1, y0));

                        draw::set_draw_color(color2);
                        draw::draw_polygon3(
                            draw::Coord::<i32>(x1, y2),
                            draw::Coord::<i32>(x2 - 1, y2),
                            draw::Coord::<i32>(x2 - 1, y1),
                            draw::Coord::<i32>(x1, y1));
                    }

                    let str = format!("{max_plot} MBit/s");
                    draw::set_draw_color(enums::Color::Gray0);
                    draw::set_font(enums::Font::HelveticaBold, 16);
                    draw::draw_text2(&str, i.x() + i.w(), i.y(),
                        0, 0, enums::Align::TopRight);
                }
                    
                draw::draw_rect_with_color(i.x(), i.y(), i.w(), i.h(), BORDER_COLOR);

                draw::pop_clip();
            }
        });

        Self {
            inner,
            history,
        }
    }
    pub fn push_value(&mut self, dlul: (i64, i64)) {
        if self.history.borrow().len() == HISTORY_SIZE {
            self.history.borrow_mut().pop_front();
        }
        self.history.borrow_mut().push_back(dlul);
    }
    pub fn clear_history(&mut self) {
        self.history.borrow_mut().clear();
    }
}

widget_extends!(DlUlBarPlotWidget, widget::Widget, inner);
