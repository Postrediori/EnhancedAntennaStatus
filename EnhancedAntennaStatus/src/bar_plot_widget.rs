#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::many_single_char_names)]
#![allow(clippy::similar_names)]

use fltk::{draw, enums, prelude::*, widget, widget_extends};

use chrono::{DateTime, Local};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::bandwidth_utils::{SIZE_MB, TrafficStatistics, format_bandwidth, nearest_fib};

const HISTORY_SIZE: usize = 80;

const COLOR_BACKGROUND: enums::Color = enums::Color::White;
const COLOR_BACKGROUND_INACTIVE: enums::Color = enums::Color::Gray0;
const COLOR_BORDER: enums::Color = enums::Color::Dark2;
const COLOR_BORDER_SELECT: enums::Color = enums::Color::Black;

const COLOR_SELECTION: enums::Color = enums::Color::Light1;
const COLOR_TOOLTIP: enums::Color = enums::Color::from_rgb(255, 219, 157);
const COLOR_TEXT: enums::Color = enums::Color::Gray0;

/*
 * BarPlotWidget
 */
pub struct BarPlotWidget {
    inner: widget::Widget,
    min: Rc<RefCell<i64>>,
    max: Rc<RefCell<i64>>,
    history: Rc<RefCell<VecDeque<(SystemTime, i64)>>>,
    unit: Rc<RefCell<String>>,
}

impl BarPlotWidget {
    pub fn new() -> Self {
        let mut inner = widget::Widget::default();

        let mouse_coord: Option<(i32, i32)> = None;
        let mouse_coord = Rc::from(RefCell::from(mouse_coord));

        let min: i64 = 0;
        let max: i64 = 100;
        let history = VecDeque::<(SystemTime, i64)>::with_capacity(HISTORY_SIZE);

        let min = Rc::from(RefCell::from(min));
        let max = Rc::from(RefCell::from(max));
        let history = Rc::from(RefCell::from(history));

        let unit = String::new();
        let unit = Rc::from(RefCell::from(unit));

        inner.draw({
            let min = min.clone();
            let max = max.clone();
            let history = history.clone();
            let mouse_coord = mouse_coord.clone();
            let unit = unit.clone();
            move |i| {
                const MARGIN_X: i32 = 2;
                const MARGIN_Y: i32 = 1;

                const PALETTE: [(i32, enums::Color); 3] = [
                    (50, enums::Color::Red),
                    (85, enums::Color::from_rgb(255, 127, 0)),
                    (100, enums::Color::DarkGreen),
                ];

                let min = *min.borrow();
                let max = *max.borrow();
                let history = history.borrow();
                let mouse_coord = mouse_coord.borrow();
                let unit = unit.borrow();

                draw::push_clip(i.x(), i.y(), i.w(), i.h());

                let bg_color = if i.active() {
                    COLOR_BACKGROUND
                } else {
                    COLOR_BACKGROUND_INACTIVE
                };
                draw::draw_rect_fill(i.x(), i.y(), i.w(), i.h(), bg_color);

                let range = (max - min) as f64;
                let dx = ((i.w() - 4) as f64) / (HISTORY_SIZE as f64);

                if !history.is_empty() {
                    let k = match *mouse_coord {
                        Some((cx, _)) => {
                            let x = cx - i.x() - MARGIN_X;
                            let k = (x as f64 / dx) as usize;

                            if k < history.len() { Some(k) } else { None }
                        }
                        _ => None,
                    };

                    if let Some(k) = k {
                        let y1 = i.y() + MARGIN_Y;
                        let y2 = i.y() + i.h() - MARGIN_Y;
                        let x1 = (i.x() as f64 + dx * (k as f64)) as i32 + MARGIN_X;
                        let x2 = (i.x() as f64 + dx * ((k + 1) as f64)) as i32 + MARGIN_X;

                        draw::set_draw_color(COLOR_SELECTION);
                        draw::draw_polygon3(
                            draw::Coord::<i32>(x1, y1),
                            draw::Coord::<i32>(x2 - 1, y1),
                            draw::Coord::<i32>(x2 - 1, y2),
                            draw::Coord::<i32>(x1, y2),
                        );
                    }

                    for k in 0..history.len() {
                        let n = history[k].1;

                        let y = ((n - min) as f64) / range;
                        let yi = (100.0 * y) as i32;

                        let c: usize = if yi < PALETTE[0].0 {
                            0
                        } else if yi < PALETTE[1].0 {
                            1
                        } else {
                            2
                        };

                        let x1 = (i.x() as f64 + dx * (k as f64)) as i32 + MARGIN_X;
                        let x2 = (i.x() as f64 + dx * ((k + 1) as f64)) as i32 + MARGIN_X;

                        let y0 = i.y() + i.h() - MARGIN_Y;
                        let y =
                            i.y() + (((i.h() - MARGIN_Y * 2) as f64) * (1.0 - y)) as i32 - MARGIN_Y;

                        draw::set_draw_color(PALETTE[c].1);
                        draw::draw_polygon3(
                            draw::Coord::<i32>(x1, y),
                            draw::Coord::<i32>(x2 - 1, y),
                            draw::Coord::<i32>(x2 - 1, y0),
                            draw::Coord::<i32>(x1, y0),
                        );
                    }

                    if let Some(k) = k {
                        let x1 = (i.x() as f64 + dx * (k as f64)) as i32 + MARGIN_X;
                        let x2 = (i.x() as f64 + dx * ((k + 1) as f64)) as i32 + MARGIN_X;

                        let (t, n) = history[k];
                        let dt: DateTime<Local> = t.into();

                        let n_str = format!("{n} {unit}");
                        let time_str = format!("{}", dt.format("%T"));

                        draw::set_font(enums::Font::Helvetica, 14);

                        let n_area = draw::text_extents(&n_str);
                        let t_area = draw::text_extents(&time_str);

                        let w = n_area.2.max(t_area.2) + MARGIN_X;
                        let h = n_area.3 + t_area.3 + MARGIN_Y * 4;

                        let x = if x2 + w < i.x() + i.w() {
                            // Align to the right
                            x2
                        } else {
                            // Align to the left
                            x1 - w
                        };

                        draw::draw_rect_fill(x, i.y() + MARGIN_Y, w, h, COLOR_TOOLTIP);

                        draw::set_draw_color(COLOR_TEXT);
                        draw::draw_text2(&n_str, x, i.y() + MARGIN_Y, 0, 0, enums::Align::TopLeft);
                        draw::draw_text2(
                            &time_str,
                            x,
                            i.y() + MARGIN_Y + n_area.3,
                            0,
                            0,
                            enums::Align::TopLeft,
                        );
                    }
                }

                let border_color = match *mouse_coord {
                    Some((_, _)) => COLOR_BORDER_SELECT,
                    None => COLOR_BORDER,
                };
                draw::draw_rect_with_color(i.x(), i.y(), i.w(), i.h(), border_color);

                draw::pop_clip();
            }
        });

        inner.handle({
            let mouse_coord = mouse_coord.clone();
            move |w, event| {
                let mut mouse_coord = mouse_coord.borrow_mut();
                let status = match event {
                    enums::Event::Enter | enums::Event::Move => {
                        *mouse_coord = Some(fltk::app::event_coords());
                        true
                    }
                    enums::Event::Leave => {
                        *mouse_coord = None;
                        true
                    }
                    _ => false,
                };
                if status {
                    w.redraw();
                }
                status
            }
        });

        Self {
            inner,
            min,
            max,
            history,
            unit,
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
        self.history.borrow_mut().push_back((SystemTime::now(), n));
    }
    pub fn clear_history(&mut self) {
        self.history.borrow_mut().clear();
    }
    pub fn set_unit(&mut self, unit: &str) {
        *self.unit.borrow_mut() = unit.to_string();
    }
}

widget_extends!(BarPlotWidget, widget::Widget, inner);

/*
 * DlUlBarPlotWidget
 */

pub const COLOR_DL: enums::Color = enums::Color::from_hex(0x00_33_22_88);
pub const COLOR_UL: enums::Color = enums::Color::from_hex(0x00_88_CC_EE);
pub const COLOR_DL_AND_UL: enums::Color = enums::Color::from_hex(0x00_DD_CC_77);

pub struct DlUlBarPlotWidget {
    inner: widget::Widget,
    history: Rc<RefCell<VecDeque<(SystemTime, TrafficStatistics)>>>,
}

impl DlUlBarPlotWidget {
    pub fn new() -> Self {
        let mut inner = widget::Widget::default();

        let mouse_coord: Option<(i32, i32)> = None;
        let mouse_coord = Rc::from(RefCell::from(mouse_coord));

        let history = VecDeque::<(SystemTime, TrafficStatistics)>::with_capacity(HISTORY_SIZE + 1);
        let history = Rc::from(RefCell::from(history));

        inner.draw({
            let history = history.clone();
            let mouse_coord = mouse_coord.clone();
            move |i| {
                const MARGIN_X: i32 = 2;
                const MARGIN_Y: i32 = 1;

                let history = history.borrow();
                let mouse_coord = mouse_coord.borrow();

                draw::push_clip(i.x(), i.y(), i.w(), i.h());

                let bg_color = if i.active() {
                    COLOR_BACKGROUND
                } else {
                    COLOR_BACKGROUND_INACTIVE
                };
                draw::draw_rect_fill(i.x(), i.y(), i.w(), i.h(), bg_color);

                let dx = ((i.w() - MARGIN_X * 2) as f64) / (HISTORY_SIZE as f64);

                if !history.is_empty() {
                    let h = history.clone();

                    let k = match *mouse_coord {
                        Some((cx, _)) => {
                            let x = cx - i.x() - MARGIN_X;
                            let k = (x as f64 / dx) as usize;

                            if k < h.len() { Some(k) } else { None }
                        }
                        _ => None,
                    };

                    if let Some(k) = k {
                        let y1 = i.y() + MARGIN_Y;
                        let y2 = i.y() + i.h() - MARGIN_Y;
                        let x1 = (i.x() as f64 + dx * (k as f64)) as i32 + MARGIN_X;
                        let x2 = (i.x() as f64 + dx * ((k + 1) as f64)) as i32 + MARGIN_X;

                        draw::set_draw_color(COLOR_SELECTION);
                        draw::draw_polygon3(
                            draw::Coord::<i32>(x1, y1),
                            draw::Coord::<i32>(x2 - 1, y1),
                            draw::Coord::<i32>(x2 - 1, y2),
                            draw::Coord::<i32>(x1, y2),
                        );
                    }

                    let max_dlul: (_, TrafficStatistics) = h
                        .into_iter()
                        .reduce(|accum, current| {
                            (
                                UNIX_EPOCH,
                                TrafficStatistics {
                                    dl: accum.1.dl.max(current.1.dl),
                                    ul: accum.1.ul.max(current.1.ul),
                                },
                            )
                        })
                        .unwrap();
                    let max_dlul = max_dlul.1.dl.max(max_dlul.1.ul);

                    let max_plot = nearest_fib(max_dlul / SIZE_MB);
                    let max_mb = max_plot * SIZE_MB;

                    for k in 0..history.len() {
                        let (_, dlul) = history[k];

                        let ydl = (dlul.dl as f64) / (max_mb as f64);
                        let yul = (dlul.ul as f64) / (max_mb as f64);

                        let (y1, y2, color1, color2) = {
                            if dlul.ul < dlul.dl {
                                (yul, ydl, COLOR_DL_AND_UL, COLOR_DL)
                            } else {
                                (ydl, yul, COLOR_DL_AND_UL, COLOR_UL)
                            }
                        };

                        let x1 = (i.x() as f64 + dx * (k as f64)) as i32 + MARGIN_X;
                        let x2 = (i.x() as f64 + dx * ((k + 1) as f64)) as i32 + MARGIN_X;

                        let y0 = i.y() + i.h() - MARGIN_Y;
                        let y1 = i.y() + (((i.h() - MARGIN_Y * 2) as f64) * (1.0 - y1)) as i32
                            - MARGIN_Y;
                        let y2 = i.y() + (((i.h() - MARGIN_Y * 2) as f64) * (1.0 - y2)) as i32
                            - MARGIN_Y;

                        draw::set_draw_color(color1);
                        draw::draw_polygon3(
                            draw::Coord::<i32>(x1, y1),
                            draw::Coord::<i32>(x2 - 1, y1),
                            draw::Coord::<i32>(x2 - 1, y0),
                            draw::Coord::<i32>(x1, y0),
                        );

                        draw::set_draw_color(color2);
                        draw::draw_polygon3(
                            draw::Coord::<i32>(x1, y2),
                            draw::Coord::<i32>(x2 - 1, y2),
                            draw::Coord::<i32>(x2 - 1, y1),
                            draw::Coord::<i32>(x1, y1),
                        );
                    }

                    let str = format!("{max_plot} MBit/s");
                    draw::set_draw_color(COLOR_TEXT);
                    draw::set_font(enums::Font::HelveticaBold, 16);
                    draw::draw_text2(
                        &str,
                        i.x() + i.w() - MARGIN_X,
                        i.y() + MARGIN_Y,
                        0,
                        0,
                        enums::Align::TopRight,
                    );

                    if let Some(k) = k {
                        let x1 = (i.x() as f64 + dx * (k as f64)) as i32 + MARGIN_X;
                        let x2 = (i.x() as f64 + dx * ((k + 1) as f64)) as i32 + MARGIN_X;

                        let (t, dlul) = history[k];
                        let dt: DateTime<Local> = t.into();

                        let dl_str = format!("DL: {}", format_bandwidth(dlul.dl));
                        let ul_str = format!("UL: {}", format_bandwidth(dlul.ul));
                        let time_str = format!("{}", dt.format("%T"));

                        draw::set_font(enums::Font::Helvetica, 14);

                        let dl_area = draw::text_extents(&dl_str);
                        let ul_area = draw::text_extents(&ul_str);
                        let t_area = draw::text_extents(&time_str);

                        let w = dl_area.2.max(ul_area.2.max(t_area.2)) + MARGIN_X * 2;
                        let h = dl_area.3 + ul_area.3 + t_area.3 + MARGIN_Y * 6;

                        let x = if x2 + w < i.x() + i.w() {
                            // Align to the right
                            x2
                        } else {
                            // Align to the left
                            x1 - w
                        };

                        draw::draw_rect_fill(x, i.y() + MARGIN_Y, w, h, COLOR_TOOLTIP);

                        draw::set_draw_color(COLOR_TEXT);
                        draw::draw_text2(&dl_str, x, i.y() + MARGIN_Y, 0, 0, enums::Align::TopLeft);
                        draw::draw_text2(
                            &ul_str,
                            x,
                            i.y() + MARGIN_Y + dl_area.3,
                            0,
                            0,
                            enums::Align::TopLeft,
                        );
                        draw::draw_text2(
                            &time_str,
                            x,
                            i.y() + MARGIN_Y * 3 + (dl_area.3 + ul_area.3),
                            0,
                            0,
                            enums::Align::TopLeft,
                        );
                    }
                }

                let border_color = match *mouse_coord {
                    Some((_, _)) => COLOR_BORDER_SELECT,
                    None => COLOR_BORDER,
                };
                draw::draw_rect_with_color(i.x(), i.y(), i.w(), i.h(), border_color);

                draw::pop_clip();
            }
        });

        inner.handle({
            let mouse_coord = mouse_coord.clone();
            move |w, event| {
                let mut mouse_coord = mouse_coord.borrow_mut();
                let status = match event {
                    enums::Event::Enter | enums::Event::Move => {
                        *mouse_coord = Some(fltk::app::event_coords());
                        true
                    }
                    enums::Event::Leave => {
                        *mouse_coord = None;
                        true
                    }
                    _ => false,
                };
                if status {
                    w.redraw();
                }
                status
            }
        });

        Self { inner, history }
    }
    pub fn push_value(&mut self, dlul: TrafficStatistics) {
        if self.history.borrow().len() == HISTORY_SIZE {
            self.history.borrow_mut().pop_front();
        }
        self.history
            .borrow_mut()
            .push_back((SystemTime::now(), dlul));
    }
    pub fn clear_history(&mut self) {
        self.history.borrow_mut().clear();
    }
}

widget_extends!(DlUlBarPlotWidget, widget::Widget, inner);
