use json::object;

pub struct StyleEditor {
    pub open: bool,
}

impl StyleEditor {
    const STYLE_FILE_NAME: &str = "egui.style.json";

    pub fn new() -> Self {
        StyleEditor { open: false }
    }

    fn load_current_style() -> anyhow::Result<egui::Style> {
        load(std::env::current_exe()?.join(Self::STYLE_FILE_NAME).as_path())
    }

    pub fn ui(&mut self, ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();
        let mut changed = false;
        egui::Window::new("Style")
            .open(&mut self.open)
            .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Reload").clicked() {
                    match Self::load_current_style() {
                        Ok(new_style) => {
                            style = new_style;
                            changed = true;
                        },
                        Err(e) => eprintln!("Unable to load `{}` file!\n{}", Self::STYLE_FILE_NAME, e),
                    }
                }
                if ui.button("Save").clicked() {
                    match std::env::current_exe() {
                        Ok(path) => save(&style, path.join(Self::STYLE_FILE_NAME).as_path())
                            .unwrap_or_else(|err| println!("{:?}", err)),
                        _ => eprintln!("Failed to get program path")
                    };
                }
                if ui.button("Default").clicked() {
                    style = default_style();
                    changed = true;
                }
            });
            
            ui.separator();
        
            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::CollapsingHeader::new("Visuals")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.collapsing("Widget visuals", |ui| {
                            [
                                (&mut style.visuals.widgets.noninteractive, "noninteractive"),
                                (&mut style.visuals.widgets.inactive, "inactive"),
                                (&mut style.visuals.widgets.hovered, "hovered"),
                                (&mut style.visuals.widgets.active, "active"),
                                (&mut style.visuals.widgets.open, "open"),
                            ].iter_mut().for_each(|(widget, name)| {
                                ui.collapsing(*name, |ui| {
                                    changed |= ui.color_edit_button_srgba(&mut widget.bg_fill).changed();
            
                                    [(&mut widget.bg_stroke, "Background stroke"), (&mut widget.fg_stroke, "Foreground stroke"),]
                                        .iter_mut().for_each(|(stroke, name)| {
                                            ui.horizontal(|ui| {
                                                changed |= ui.add(egui::DragValue::new(&mut stroke.width).speed(0.2)).changed();
                                                changed |= ui.color_edit_button_srgba(&mut stroke.color).changed();
                                                ui.label(*name);
                                            });
                                        });
                                    
                                    ui.horizontal(|ui| {
                                        ui.label("Rounding");
                                        changed |= ui.add(egui::DragValue::new(&mut widget.rounding.nw).speed(0.2)).changed();
                                    });
                                    widget.rounding = egui::Rounding::same(widget.rounding.nw);
            
                                    ui.horizontal(|ui| {
                                        ui.label("Frame expansion");
                                        changed |= ui.add(egui::DragValue::new(&mut widget.expansion).speed(0.2)).changed();
                                    });
                                });
                            });
                        });
            
                        ui.collapsing("Window", |ui| {
                            ui.horizontal(|ui| {
                                ui.label("Rounding");
                                changed |= ui.add(egui::DragValue::new(&mut style.visuals.window_rounding.nw).speed(0.2)).changed();
                            });
                            style.visuals.window_rounding = egui::Rounding::same(style.visuals.window_rounding.nw);
            
                            ui.label("Shadow:");
                            ui.horizontal(|ui| {
                                changed |= ui.add(egui::DragValue::new(&mut style.visuals.window_shadow.extrusion).speed(0.2)).changed();
                                changed |= ui.color_edit_button_srgba(&mut style.visuals.window_shadow.color).changed();
                            });
                        });
            
                        ui.collapsing("Color", |ui| {
                            ui.horizontal(|ui| {
                                changed |= ui.color_edit_button_srgba(& mut style.visuals.hyperlink_color).changed();
                                ui.label("hyperlink color");
                            });
                            ui.horizontal(|ui| {
                                changed |= ui.color_edit_button_srgba(& mut style.visuals.faint_bg_color).changed();
                                ui.label("faint bg color");
                            });
                            ui.horizontal(|ui| {
                                changed |= ui.color_edit_button_srgba(& mut style.visuals.extreme_bg_color).changed();
                                ui.label("extreme bg color");
                            });
                            ui.horizontal(|ui| {
                                changed |= ui.color_edit_button_srgba(& mut style.visuals.code_bg_color).changed();
                                ui.label("code bg color");
                            });
                            ui.horizontal(|ui| {
                                changed |= ui.color_edit_button_srgba(& mut style.visuals.warn_fg_color).changed();
                                ui.label("warn fg color");
                            });
                            ui.horizontal(|ui| {
                                changed |= ui.color_edit_button_srgba(& mut style.visuals.error_fg_color).changed();
                                ui.label("error fg color");
                            });
                        });
            
                        ui.horizontal(|ui| {
                            ui.label("Selection");
                            changed |= ui.add(egui::DragValue::new(&mut style.visuals.selection.stroke.width).speed(0.2)).changed();
                            changed |= ui.color_edit_button_srgba(&mut style.visuals.selection.bg_fill).changed();
                        });
            
                        ui.horizontal(|ui| {
                            ui.label("Popup shadow:");
                            changed |= ui.add(egui::DragValue::new(&mut style.visuals.popup_shadow.extrusion).speed(0.2)).changed();
                            changed |= ui.color_edit_button_srgba(& mut style.visuals.popup_shadow.color).changed();
                        });
            
                        ui.horizontal(|ui| {
                            ui.label("Resize corner size:");
                            changed |= ui.add(egui::DragValue::new(&mut style.visuals.resize_corner_size).speed(0.2)).changed();
                        });
            
                        ui.horizontal(|ui| {
                            ui.label("Text corner width:");
                            changed |= ui.add(egui::DragValue::new(&mut style.visuals.text_cursor_width).speed(0.2)).changed();
                            changed |= ui.checkbox(&mut style.visuals.text_cursor_preview, "Preview").changed();
                        });
            
                        ui.horizontal(|ui| {
                            ui.label("Clip rect margin:");
                            changed |= ui.add(egui::DragValue::new(&mut style.visuals.clip_rect_margin).speed(0.2)).changed();
                        });
            
                        changed |= ui.checkbox(&mut style.visuals.button_frame, "Button frame").changed();
                        changed |= ui.checkbox(&mut style.visuals.collapsing_header_frame, "Collapsing header frame").changed();
                
                    });
        
                ui.collapsing("Text", |ui| {
                    style.text_styles.iter_mut().for_each(|(style, font_id)| {
                        ui.horizontal(|ui| {
                            ui.label(format!("{:?}", style));
                            let mut size = font_id.size;
                            if ui.add(egui::DragValue::new(&mut size).speed(0.2)).changed() {
                                changed = true;
                                *font_id = egui::FontId::proportional(size);
                            }
                        });
                    });
                });    
            });
        });
    
        if changed {
            ctx.set_style(style);
        }
    }
}

fn color_to_string(color: egui::Color32) -> String {
    format!(
        "0x{:08x}",
        color.to_array().iter().fold(0, |num, v| (num << 8) | *v as u32),
    )
}

fn string_to_color(str: &str) -> egui::Color32 {
    let num = usize::from_str_radix(&str[2..], 16).unwrap_or(0xff0000ff);
    let color_data: Vec<u8> = (0..4).map(|i| ((num >> (i * 8)) & 0xff) as u8).rev().collect();

    egui::Color32::from_rgba_unmultiplied(color_data[0], color_data[1], color_data[2], color_data[3])
}

pub fn default_style() -> egui::Style {
    match StyleEditor::load_current_style() {
        Ok(style) => style,
        _ => {
            let defualt_style_json: json::JsonValue = json::parse(include_str!("egui.style.json")).unwrap();
            match json_to_style(&defualt_style_json) {
                Ok(style) => style,
                _ => egui::Style::default(),
            }
        }
    }
}

pub fn load(path: &std::path::Path) -> anyhow::Result<egui::Style> {
    Ok(json_to_style(&json::parse(std::fs::read_to_string(path)?.as_str())?)?)
}
    
fn json_to_style(style_data: &json::JsonValue) -> anyhow::Result<egui::Style> {
    use egui::style::*;
    use json::JsonError::WrongType;

    const ESTR: &str = "err";

    let mut style = Style {
        visuals: Visuals {
            widgets: Widgets {
                noninteractive: WidgetVisuals {
                    bg_fill: string_to_color(style_data["visuals"]["widgets"]["noninteractive"]["bg_fill"].as_str().ok_or(WrongType(ESTR.to_string()))?),
                    bg_stroke: egui::Stroke {
                        width: style_data["visuals"]["widgets"]["noninteractive"]["bg_stroke"]["width"].as_f32().ok_or(WrongType(ESTR.to_string()))?,
                        color: string_to_color(style_data["visuals"]["widgets"]["noninteractive"]["bg_stroke"]["color"].as_str().ok_or(WrongType(ESTR.to_string()))?)
                    },
                    rounding: egui::Rounding::same(style_data["visuals"]["widgets"]["noninteractive"]["rounding"].as_f32().ok_or(WrongType(ESTR.to_string()))?),
                    fg_stroke: egui::Stroke {
                        width: style_data["visuals"]["widgets"]["noninteractive"]["fg_stroke"]["width"].as_f32().ok_or(WrongType(ESTR.to_string()))?,
                        color: string_to_color(style_data["visuals"]["widgets"]["noninteractive"]["fg_stroke"]["color"].as_str().ok_or(WrongType(ESTR.to_string()))?)
                    },
                    expansion: style_data["visuals"]["widgets"]["noninteractive"]["expansion"].as_f32().ok_or(WrongType(ESTR.to_string()))?,
                },
                inactive: WidgetVisuals {
                    bg_fill: string_to_color(style_data["visuals"]["widgets"]["inactive"]["bg_fill"].as_str().ok_or(WrongType(ESTR.to_string()))?),
                    bg_stroke: egui::Stroke {
                        width: style_data["visuals"]["widgets"]["inactive"]["bg_stroke"]["width"].as_f32().ok_or(WrongType(ESTR.to_string()))?,
                        color: string_to_color(style_data["visuals"]["widgets"]["inactive"]["bg_stroke"]["color"].as_str().ok_or(WrongType(ESTR.to_string()))?)
                    },
                    rounding: egui::Rounding::same(style_data["visuals"]["widgets"]["inactive"]["rounding"].as_f32().ok_or(WrongType(ESTR.to_string()))?),
                    fg_stroke: egui::Stroke {
                        width: style_data["visuals"]["widgets"]["inactive"]["fg_stroke"]["width"].as_f32().ok_or(WrongType(ESTR.to_string()))?,
                        color: string_to_color(style_data["visuals"]["widgets"]["inactive"]["fg_stroke"]["color"].as_str().ok_or(WrongType(ESTR.to_string()))?)
                    },
                    expansion: style_data["visuals"]["widgets"]["inactive"]["expansion"].as_f32().ok_or(WrongType(ESTR.to_string()))?,
                },
                hovered: WidgetVisuals {
                    bg_fill: string_to_color(style_data["visuals"]["widgets"]["hovered"]["bg_fill"].as_str().ok_or(WrongType(ESTR.to_string()))?),
                    bg_stroke: egui::Stroke {
                        width: style_data["visuals"]["widgets"]["hovered"]["bg_stroke"]["width"].as_f32().ok_or(WrongType(ESTR.to_string()))?,
                        color: string_to_color(style_data["visuals"]["widgets"]["hovered"]["bg_stroke"]["color"].as_str().ok_or(WrongType(ESTR.to_string()))?)
                    },
                    rounding: egui::Rounding::same(style_data["visuals"]["widgets"]["hovered"]["rounding"].as_f32().ok_or(WrongType(ESTR.to_string()))?),
                    fg_stroke: egui::Stroke {
                        width: style_data["visuals"]["widgets"]["hovered"]["fg_stroke"]["width"].as_f32().ok_or(WrongType(ESTR.to_string()))?,
                        color: string_to_color(style_data["visuals"]["widgets"]["hovered"]["fg_stroke"]["color"].as_str().ok_or(WrongType(ESTR.to_string()))?)
                    },
                    expansion: style_data["visuals"]["widgets"]["hovered"]["expansion"].as_f32().ok_or(WrongType(ESTR.to_string()))?,
                },
                active: WidgetVisuals {
                    bg_fill: string_to_color(style_data["visuals"]["widgets"]["active"]["bg_fill"].as_str().ok_or(WrongType(ESTR.to_string()))?),
                    bg_stroke: egui::Stroke {
                        width: style_data["visuals"]["widgets"]["active"]["bg_stroke"]["width"].as_f32().ok_or(WrongType(ESTR.to_string()))?,
                        color: string_to_color(style_data["visuals"]["widgets"]["active"]["bg_stroke"]["color"].as_str().ok_or(WrongType(ESTR.to_string()))?)
                    },
                    rounding: egui::Rounding::same(style_data["visuals"]["widgets"]["active"]["rounding"].as_f32().ok_or(WrongType(ESTR.to_string()))?),
                    fg_stroke: egui::Stroke {
                        width: style_data["visuals"]["widgets"]["active"]["fg_stroke"]["width"].as_f32().ok_or(WrongType(ESTR.to_string()))?,
                        color: string_to_color(style_data["visuals"]["widgets"]["active"]["fg_stroke"]["color"].as_str().ok_or(WrongType(ESTR.to_string()))?)
                    },
                    expansion: style_data["visuals"]["widgets"]["active"]["expansion"].as_f32().ok_or(WrongType(ESTR.to_string()))?,
                },
                open: WidgetVisuals {
                    bg_fill: string_to_color(style_data["visuals"]["widgets"]["open"]["bg_fill"].as_str().ok_or(WrongType(ESTR.to_string()))?),
                    bg_stroke: egui::Stroke {
                        width: style_data["visuals"]["widgets"]["open"]["bg_stroke"]["width"].as_f32().ok_or(WrongType(ESTR.to_string()))?,
                        color: string_to_color(style_data["visuals"]["widgets"]["open"]["bg_stroke"]["color"].as_str().ok_or(WrongType(ESTR.to_string()))?)
                    },
                    rounding: egui::Rounding::same(style_data["visuals"]["widgets"]["open"]["rounding"].as_f32().ok_or(WrongType(ESTR.to_string()))?),
                    fg_stroke: egui::Stroke {
                        width: style_data["visuals"]["widgets"]["open"]["fg_stroke"]["width"].as_f32().ok_or(WrongType(ESTR.to_string()))?,
                        color: string_to_color(style_data["visuals"]["widgets"]["open"]["fg_stroke"]["color"].as_str().ok_or(WrongType(ESTR.to_string()))?)
                    },
                    expansion: style_data["visuals"]["widgets"]["open"]["expansion"].as_f32().ok_or(WrongType(ESTR.to_string()))?,
                },
            },
            selection: Selection {
                bg_fill: string_to_color(style_data["visuals"]["selection"]["bg_fill"].as_str().ok_or(WrongType(ESTR.to_string()))?),
                stroke: egui::Stroke {
                    width: style_data["visuals"]["selection"]["stroke"]["width"].as_f32().ok_or(WrongType(ESTR.to_string()))?,
                    color: string_to_color(style_data["visuals"]["selection"]["stroke"]["color"].as_str().ok_or(WrongType(ESTR.to_string()))?)
                },
            },
            hyperlink_color: string_to_color(style_data["visuals"]["hyperlink_color"].as_str().ok_or(WrongType(ESTR.to_string()))?),
            faint_bg_color: string_to_color(style_data["visuals"]["faint_bg_color"].as_str().ok_or(WrongType(ESTR.to_string()))?),
            extreme_bg_color: string_to_color(style_data["visuals"]["extreme_bg_color"].as_str().ok_or(WrongType(ESTR.to_string()))?),
            code_bg_color: string_to_color(style_data["visuals"]["code_bg_color"].as_str().ok_or(WrongType(ESTR.to_string()))?),
            warn_fg_color: string_to_color(style_data["visuals"]["warn_fg_color"].as_str().ok_or(WrongType(ESTR.to_string()))?),
            error_fg_color: string_to_color(style_data["visuals"]["error_fg_color"].as_str().ok_or(WrongType(ESTR.to_string()))?),
            window_rounding: egui::Rounding::same(style_data["visuals"]["window_rounding"].as_f32().ok_or(WrongType(ESTR.to_string()))?),
            window_shadow: egui::epaint::Shadow {
                extrusion: style_data["visuals"]["window_shadow"]["extrusion"].as_f32().ok_or(WrongType(ESTR.to_string()))?,
                color: string_to_color(style_data["visuals"]["window_shadow"]["color"].as_str().ok_or(WrongType(ESTR.to_string()))?),
            },
            popup_shadow: egui::epaint::Shadow {
                extrusion: style_data["visuals"]["popup_shadow"]["extrusion"].as_f32().ok_or(WrongType(ESTR.to_string()))?,
                color: string_to_color(style_data["visuals"]["popup_shadow"]["color"].as_str().ok_or(WrongType(ESTR.to_string()))?),
            },
            resize_corner_size: style_data["visuals"]["resize_corner_size"].as_f32().ok_or(WrongType(ESTR.to_string()))?,
            text_cursor_width: style_data["visuals"]["text_cursor_width"].as_f32().ok_or(WrongType(ESTR.to_string()))?,
            clip_rect_margin: style_data["visuals"]["clip_rect_margin"].as_f32().ok_or(WrongType(ESTR.to_string()))?,
            text_cursor_preview: style_data["visuals"]["text_cursor_preview"].as_bool().ok_or(WrongType(ESTR.to_string()))?,
            button_frame: style_data["visuals"]["button_frame"].as_bool().ok_or(WrongType(ESTR.to_string()))?,
            collapsing_header_frame: style_data["visuals"]["collapsing_header_frame"].as_bool().ok_or(WrongType(ESTR.to_string()))?,
            ..Default::default()
        },
        ..Default::default()
    };

    let text_style_map: std::collections::HashMap<&str, TextStyle> = std::collections::HashMap::from([
        ("Small", TextStyle::Small),
        ("Body", TextStyle::Body),
        ("Monospace", TextStyle::Monospace),
        ("Button", TextStyle::Button),
        ("Heading", TextStyle::Heading),
    ]);
    for (text_style, size) in style_data["text"].entries() {
        match text_style_map.get(text_style) {
            Some(ts) => { style.text_styles.insert(ts.clone(),egui::FontId::proportional(size.as_f32().ok_or(WrongType(ESTR.to_string()))?)); }
            _ => {}
        }
    }

    Ok(style)
}

pub fn save(style: &egui::Style, path: &std::path::Path) -> anyhow::Result<()> {
    let mut style_data = object!{
        visuals: {
            widgets: {
                noninteractive: {
                    bg_fill: color_to_string(style.visuals.widgets.noninteractive.bg_fill),
                    bg_stroke: {
                        width: style.visuals.widgets.noninteractive.bg_stroke.width,
                        color: color_to_string(style.visuals.widgets.noninteractive.bg_stroke.color),
                    },
                    rounding: style.visuals.widgets.noninteractive.rounding.nw,
                    fg_stroke: {
                        width: style.visuals.widgets.noninteractive.fg_stroke.width,
                        color: color_to_string(style.visuals.widgets.noninteractive.fg_stroke.color),
                    },
                    expansion: style.visuals.widgets.noninteractive.expansion,
                },
                inactive: {
                    bg_fill: color_to_string(style.visuals.widgets.inactive.bg_fill),
                    bg_stroke: {
                        width: style.visuals.widgets.inactive.bg_stroke.width,
                        color: color_to_string(style.visuals.widgets.inactive.bg_stroke.color),
                    },
                    rounding: style.visuals.widgets.inactive.rounding.nw,
                    fg_stroke: {
                        width: style.visuals.widgets.inactive.fg_stroke.width,
                        color: color_to_string(style.visuals.widgets.inactive.fg_stroke.color),
                    },
                    expansion: style.visuals.widgets.inactive.expansion,
                },
                hovered: {
                    bg_fill: color_to_string(style.visuals.widgets.hovered.bg_fill),
                    bg_stroke: {
                        width: style.visuals.widgets.hovered.bg_stroke.width,
                        color: color_to_string(style.visuals.widgets.hovered.bg_stroke.color),
                    },
                    rounding: style.visuals.widgets.hovered.rounding.nw,
                    fg_stroke: {
                        width: style.visuals.widgets.hovered.fg_stroke.width,
                        color: color_to_string(style.visuals.widgets.hovered.fg_stroke.color),
                    },
                    expansion: style.visuals.widgets.hovered.expansion,
                },
                active: {
                    bg_fill: color_to_string(style.visuals.widgets.active.bg_fill),
                    bg_stroke: {
                        width: style.visuals.widgets.active.bg_stroke.width,
                        color: color_to_string(style.visuals.widgets.active.bg_stroke.color),
                    },
                    rounding: style.visuals.widgets.active.rounding.nw,
                    fg_stroke: {
                        width: style.visuals.widgets.active.fg_stroke.width,
                        color: color_to_string(style.visuals.widgets.active.fg_stroke.color),
                    },
                    expansion: style.visuals.widgets.active.expansion,
                },
                open: {
                    bg_fill: color_to_string(style.visuals.widgets.open.bg_fill),
                    bg_stroke: {
                        width: style.visuals.widgets.open.bg_stroke.width,
                        color: color_to_string(style.visuals.widgets.open.bg_stroke.color),
                    },
                    rounding: style.visuals.widgets.open.rounding.nw,
                    fg_stroke: {
                        width: style.visuals.widgets.open.fg_stroke.width,
                        color: color_to_string(style.visuals.widgets.open.fg_stroke.color),
                    },
                    expansion: style.visuals.widgets.open.expansion,
                },
            },
            selection: {
                bg_fill: color_to_string(style.visuals.selection.bg_fill),
                stroke: {
                    width: style.visuals.selection.stroke.width,
                    color: color_to_string(style.visuals.selection.stroke.color),
                }
            },
            hyperlink_color: color_to_string(style.visuals.hyperlink_color),
            faint_bg_color: color_to_string(style.visuals.faint_bg_color),
            extreme_bg_color: color_to_string(style.visuals.extreme_bg_color),
            code_bg_color: color_to_string(style.visuals.code_bg_color),
            warn_fg_color: color_to_string(style.visuals.warn_fg_color),
            error_fg_color: color_to_string(style.visuals.error_fg_color),
            window_rounding: style.visuals.window_rounding.nw,
            window_shadow: {
                extrusion: style.visuals.window_shadow.extrusion,
                color: color_to_string(style.visuals.window_shadow.color),
            },
            popup_shadow: {
                extrusion: style.visuals.popup_shadow.extrusion,
                color: color_to_string(style.visuals.popup_shadow.color),
            },
            resize_corner_size: style.visuals.resize_corner_size,
            text_cursor_width: style.visuals.text_cursor_width,
            clip_rect_margin: style.visuals.clip_rect_margin,
            text_cursor_preview: style.visuals.text_cursor_preview,
            button_frame: style.visuals.button_frame,
            collapsing_header_frame: style.visuals.collapsing_header_frame,
        },
    };
    style_data["text"] = json::JsonValue::new_object();
    style.text_styles.iter().for_each(|(style, font_id)| {
        style_data["text"][format!("{:?}", style)] = font_id.size.into();
    });

    std::fs::write(path, style_data.pretty(4))?;

    Ok(())
}
