use std::time::Instant;

use egui::{
    plot::{Bar, BarChart, Plot},
    Context, DragValue, Response, Ui,
};
use itertools::Itertools;
use log::warn;

use crate::{
    calibration_module::CalibrationModule,
    camera_module::{CameraStream, Image},
    LARGEST_WAVELENGTH, SMALLEST_WAVELENGTH,
};

pub struct TracerModule {
    start_inst: Option<std::time::Instant>,
    time_s: Vec<f32>,
    tracers: Vec<PeakTrace>,
    record: bool,
    reconfigure_next: bool,
    add_new_next: bool,
}

impl TracerModule {
    pub fn display(
        &mut self,
        ctx: &Context,
        calib: &mut CalibrationModule,
        width: u32,
        height: u32,
    ) {
        egui::SidePanel::right("tracer_opts").show(ctx, |ui| self.side_panel(ui));

        egui::CentralPanel::default().show(ctx, |ui| {
            self.main_view(ui, calib, width, height);
        });
    }
}

impl TracerModule {
    pub fn main_view(
        &mut self,
        ui: &mut Ui,
        calib: &mut CalibrationModule,
        width: u32,
        height: u32,
    ) {
        if let Some(img) = CameraStream::get_img(width, height) {
            // update according to flags
            if self.record {
                let t0 = self
                    .start_inst
                    .expect("the start value should always be known while recording");
                self.time_s.push((Instant::now() - t0).as_secs_f32())
            }
            for tracer in &mut self.tracers {
                tracer.update(&img, calib, self.record);
            }
            if self.add_new_next {
                match PeakTrace::new(500.0, &img, calib) {
                    Some(tracer) => self.tracers.push(tracer),
                    None => warn!("could not add new tracer"),
                }
                self.add_new_next = false
            }
            if self.reconfigure_next {
                self.tracers
                    .sort_by(|a, b| a.wavelength.partial_cmp(&b.wavelength).unwrap());
                if self.record {
                    self.start_recording()
                } else {
                    self.take_reference()
                }
                self.reconfigure_next = false
            }

            if self.record {
                Plot::new("Tracer plot").show(ui, |ui| {
                    for tracer in self.tracers {
                        let mut line = tracer.make_line();
                        ui.line(line);
                        ui.text(tracer.wavelength.to_string())
                    }
                });
            } else {
                let bars = self
                    .tracers
                    .iter()
                    .enumerate()
                    .map(|(i, tracer)| {
                        Bar::new(i as f64, tracer.current_rel() as f64).name(tracer.wavelength)
                    })
                    .collect();
                let chart = BarChart::new(bars).vertical().name("abdbdb");
                Plot::new("Absorbance").show(ui, |plot_ui| plot_ui.bar_chart(chart));
            }
        } else {
            ui.strong("could not get image");
        }
    }

    pub fn side_panel(&mut self, ui: &mut Ui) {
        ui.label("trace wavelengths");
        for tracer in &mut self.tracers {
            self.reconfigure_next |= tracer.ui(ui).drag_released();
        }
        if ui.button("add new wavelength").clicked() {
            self.add_new_next = true;
        }

        if ui.button("Take reference").clicked() {
            self.take_reference()
        }

        if ui.button("start recording").clicked() {
            self.start_recording()
        }
    }
}

impl TracerModule {
    fn start_recording(&mut self) {
        self.take_reference();
        self.start_inst = Some(std::time::Instant::now());
        self.record = true;
    }

    pub fn take_reference(&mut self) {
        self.tracers.iter_mut().for_each(|tracer| {
            tracer.clear();
            tracer.take_reference()
        })
    }
}

impl Default for TracerModule {
    fn default() -> Self {
        Self {
            start_inst: None,
            time_s: Vec::new(),
            tracers: Vec::new(),
            record: false,
            reconfigure_next: false,
            add_new_next: true,
        }
    }
}

pub struct PeakTrace {
    wavelength: f32,
    reference: f32,
    current_abs: f32,
    abs_values: Vec<f32>,
}

impl PeakTrace {
    fn new(wavelength: f32, img: &Image, calib: &mut CalibrationModule) -> Option<Self> {
        let current_val = img.read_line_lightness(&calib.get_line(wavelength)?);
        Some(Self {
            wavelength,
            reference: current_val,
            current_abs: current_val,
            abs_values: Vec::new(),
        })
    }

    fn update(&mut self, img: &Image, calib: &mut CalibrationModule, record: bool) -> Option<()> {
        self.current_abs = img.read_line_lightness(&calib.get_line(self.wavelength)?);
        if record {
            self.abs_values.push(self.current_abs)
        }
        Some(())
    }

    fn take_reference(&mut self) {
        self.reference = self.current_abs;
    }

    fn clear(&mut self) {
        self.abs_values = Vec::new();
    }

    fn ui(&mut self, ui: &mut Ui) -> Response {
        ui.add(
            DragValue::new(&mut self.wavelength)
                .clamp_range(SMALLEST_WAVELENGTH as f32..=LARGEST_WAVELENGTH as f32)
                .prefix("λ: ")
                .suffix(" nm"),
        )
    }

    fn abs_and_ref(&self) -> (&[f32], f32) {
        (&self.abs_values, self.reference)
    }

    fn current_rel(&self) -> f32 {
        self.current_abs / self.reference
    }

    fn make_points(&self, ts: &[f32]) -> egui::plot::Line {
        egui::plot::Line::new(
            self.abs_values
                .iter()
                .zip(ts)
                .map(|(val, t)| [ts as f64, val / self.reference as f64])
                .collect_vec(),
        )
    }
}
