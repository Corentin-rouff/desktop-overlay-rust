use chrono::{Datelike, Local, Timelike};
use eframe::egui;

pub fn show(
    ctx: &egui::Context,
    position: egui::Pos2,
    edit_mode: bool,
    show_seconds: bool,
    show_date: bool,
) -> egui::Pos2 {
    let now = Local::now();
    let time = if show_seconds {
        format!("{:02}:{:02}:{:02}", now.hour(), now.minute(), now.second())
    } else {
        format!("{:02}:{:02}", now.hour(), now.minute())
    };

    let date = format!(
        "{} {} {} {}",
        weekday_fr(now.weekday().num_days_from_monday()),
        now.day(),
        month_fr(now.month()),
        now.year()
    );

    let area = egui::Area::new(egui::Id::new("clock_widget"))
        .current_pos(position)
        .movable(edit_mode)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(egui::Color32::from_rgba_premultiplied(20, 20, 24, 235))
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(64)))
                .corner_radius(14)
                .inner_margin(14.0)
                .show(ui, |ui| {
                    ui.set_min_width(235.0);
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("HORLOGE").strong().size(14.0));
                        if edit_mode {
                            ui.label(
                                egui::RichText::new("• déplacer")
                                    .small()
                                    .color(egui::Color32::GRAY),
                            );
                        }
                    });
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new(time).strong().size(34.0));
                    if show_date {
                        ui.label(
                            egui::RichText::new(date)
                                .size(14.0)
                                .color(egui::Color32::from_gray(185)),
                        );
                    }
                });
        });

    area.response.rect.min
}

fn weekday_fr(day: u32) -> &'static str {
    match day {
        0 => "Lundi",
        1 => "Mardi",
        2 => "Mercredi",
        3 => "Jeudi",
        4 => "Vendredi",
        5 => "Samedi",
        _ => "Dimanche",
    }
}

fn month_fr(month: u32) -> &'static str {
    match month {
        1 => "janvier",
        2 => "février",
        3 => "mars",
        4 => "avril",
        5 => "mai",
        6 => "juin",
        7 => "juillet",
        8 => "août",
        9 => "septembre",
        10 => "octobre",
        11 => "novembre",
        _ => "décembre",
    }
}
