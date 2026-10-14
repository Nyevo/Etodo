use chrono::{Datelike, Local, NaiveDate};
use eframe::egui;
use egui::RichText;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::config::configure_fonts;
mod config;

// ============================================================
// 数据模型
// ============================================================

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq)]
enum Priority {
    Low,
    Medium,
    High,
}

impl Priority {
    fn color(&self) -> egui::Color32 {
        match self {
            Priority::Low => egui::Color32::from_rgb(52, 168, 83),
            Priority::Medium => egui::Color32::from_rgb(251, 188, 4),
            Priority::High => egui::Color32::from_rgb(234, 67, 53),
        }
    }
    fn label(&self) -> &str {
        match self {
            Priority::Low => "低",
            Priority::Medium => "中",
            Priority::High => "高",
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct Task {
    id: u64,
    title: String,
    due_date: NaiveDate,
    done: bool,
    priority: Priority,
}

// ============================================================
// 应用状态
// ============================================================

const SAVE_PATH: &str = "todo_data.ron";

struct TodoApp {
    tasks: Vec<Task>,
    current_year: i32,
    current_month: u32,
    selected_date: Option<NaiveDate>,
    new_task_title: String,
    new_task_priority: Priority,
    next_id: u64,
}

impl TodoApp {
    fn new(tasks: Vec<Task>, year: i32, month: u32) -> Self {
        let today = Local::now().date_naive();
        let next_id = tasks.iter().map(|t| t.id).max().unwrap_or(0) + 1;
        Self {
            tasks,
            current_year: year,
            current_month: month,
            selected_date: Some(today),
            new_task_title: String::new(),
            new_task_priority: Priority::Medium,
            next_id,
        }
    }

    /// 自定义持久化方法，命名与 App::save 区分开
    fn persist(&self) {
        let data = (&self.tasks, self.current_year, self.current_month);
        if let Ok(text) = ron::to_string(&data) {
            let _ = std::fs::write(SAVE_PATH, text);
        }
    }

    fn load() -> (Vec<Task>, i32, u32) {
        let today = Local::now().date_naive();
        if let Ok(text) = std::fs::read_to_string(SAVE_PATH) {
            if let Ok((tasks, year, month)) = ron::from_str::<(Vec<Task>, i32, u32)>(&text) {
                return (tasks, year, month);
            }
        }
        (Vec::new(), today.year(), today.month())
    }

    fn tasks_by_date(&self) -> HashMap<NaiveDate, Vec<&Task>> {
        let mut map: HashMap<NaiveDate, Vec<&Task>> = HashMap::new();
        for task in &self.tasks {
            map.entry(task.due_date).or_default().push(task);
        }
        map
    }

    fn prev_month(&mut self) {
        if self.current_month == 1 {
            self.current_month = 12;
            self.current_year -= 1;
        } else {
            self.current_month -= 1;
        }
    }

    fn next_month(&mut self) {
        if self.current_month == 12 {
            self.current_month = 1;
            self.current_year += 1;
        } else {
            self.current_month += 1;
        }
    }
}

// ============================================================
// 月份网格计算
// ============================================================

fn month_grid(year: i32, month: u32) -> Vec<Vec<Option<NaiveDate>>> {
    let first = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let weekday_offset = first.weekday().num_days_from_monday() as usize;

    let days_in_month = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    }
    .unwrap()
    .signed_duration_since(first)
    .num_days() as usize;

    let mut grid = Vec::new();
    let mut week: Vec<Option<NaiveDate>> = vec![None; 7];
    let mut col = weekday_offset;

    for day in 1..=days_in_month {
        week[col] = NaiveDate::from_ymd_opt(year, month, day as u32);
        col += 1;
        if col == 7 {
            grid.push(week.clone());
            week = vec![None; 7];
            col = 0;
        }
    }
    if col > 0 {
        grid.push(week);
    }
    grid
}

// ============================================================
// 日历单元格（自由函数，避免与 self 借用冲突）
// ============================================================

/// 渲染单个日期格子，返回 true 表示被点击
fn render_day_cell(
    ui: &mut egui::Ui,
    date: NaiveDate,
    tasks: &[&Task],
    today: NaiveDate,
    selected: Option<NaiveDate>,
) -> bool {
    let is_today = date == today;
    let is_selected = selected == Some(date);

    let text = egui::RichText::new(format!("{}", date.day()));
    let text = if is_today {
        text.strong().color(egui::Color32::from_rgb(66, 133, 244))
    } else {
        text
    };

    let mut button = egui::Button::new(text);
    if is_selected {
        button = button.fill(egui::Color32::from_rgb(210, 227, 252));
    }

    let response = ui.add_sized(egui::vec2(48.0, 40.0), button);

    // 在格子右下角绘制任务状态指示点
    if !tasks.is_empty() {
        let done_count = tasks.iter().filter(|t| t.done).count();
        let dot_color = if done_count == tasks.len() {
            egui::Color32::from_rgb(52, 168, 83) // 全部完成
        } else {
            egui::Color32::from_rgb(234, 67, 53) // 有未完成
        };
        ui.painter().circle_filled(
            response.rect.right_bottom() - egui::vec2(7.0, 7.0),
            3.0,
            dot_color,
        );
    }

    response.clicked()
}

// ============================================================
// 渲染逻辑
// ============================================================

impl TodoApp {
    fn render_task_panel(&mut self, ui: &mut egui::Ui) {
        let Some(date) = self.selected_date else {
            ui.label("请选择日期");
            return;
        };

        ui.heading(date.format("%Y-%m-%d").to_string());
        ui.separator();

        // 新任务输入区
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut self.new_task_title);
            egui::ComboBox::from_id_salt("priority_combo")
                .selected_text(self.new_task_priority.label())
                .width(60.0)
                .show_ui(ui, |ui| {
                    for p in [Priority::Low, Priority::Medium, Priority::High] {
                        ui.selectable_value(&mut self.new_task_priority, p, p.label());
                    }
                });
            if ui.button("添加").clicked() && !self.new_task_title.trim().is_empty() {
                self.tasks.push(Task {
                    id: self.next_id,
                    title: std::mem::take(&mut self.new_task_title),
                    due_date: date,
                    done: false,
                    priority: self.new_task_priority,
                });
                self.next_id += 1;
            }
        });

        ui.separator();

        // 当日任务列表
        let mut to_remove: Vec<u64> = Vec::new();
        egui::ScrollArea::vertical().show(ui, |ui| {
            let task_count = self.tasks.iter().filter(|t| t.due_date == date).count();
            if task_count == 0 {
                ui.weak("暂无任务");
            }

            for task in self.tasks.iter_mut().filter(|t| t.due_date == date) {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut task.done, "");
                    let mut text = egui::RichText::new(&task.title);
                    if task.done {
                        text = text.strikethrough().weak();
                    }
                    ui.label(text);

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.colored_label(task.priority.color(), "●");
                        if ui.small_button("删除").clicked() {
                            to_remove.push(task.id);
                        }
                    });
                });
            }
        });
        self.tasks.retain(|t| !to_remove.contains(&t.id));
    }

    fn render_calendar(&mut self, ui: &mut egui::Ui) {
        // 月份导航
        ui.horizontal(|ui| {
            if ui.button("◀").clicked() {
                self.prev_month();
            }
            ui.heading(format!(
                "{} 年 {} 月",
                self.current_year, self.current_month
            ));
            if ui.button("▶").clicked() {
                self.next_month();
            }
            if ui.button("今天").clicked() {
                let today = Local::now().date_naive();
                self.current_year = today.year();
                self.current_month = today.month();
                self.selected_date = Some(today);
            }
        });

        ui.separator();

        // 预先取好所有只读数据，避免闭包再借用 self
        let task_map = self.tasks_by_date();
        let grid = month_grid(self.current_year, self.current_month);
        let today = Local::now().date_naive();
        let selected = self.selected_date;

        // 闭包中只写局部变量，UI 交互结果结束后再写回 self
        let mut clicked_date: Option<NaiveDate> = None;

        egui::Grid::new("calendar_grid")
            .spacing([4.0, 4.0])
            .show(ui, |ui| {
                // 表头：周一为第一天
                for name in ["一", "二", "三", "四", "五", "六", "日"] {
                    ui.label(RichText::new(name).strong());
                }
                ui.end_row();

                for week in &grid {
                    for day_opt in week {
                        match day_opt {
                            Some(date) => {
                                let empty: Vec<&Task> = Vec::new();
                                let tasks = task_map.get(date).unwrap_or(&empty);
                                if render_day_cell(ui, *date, tasks, today, selected) {
                                    clicked_date = Some(*date);
                                }
                            }
                            None => {
                                ui.allocate_space(egui::vec2(48.0, 40.0));
                            }
                        }
                    }
                    ui.end_row();
                }
            });

        // 闭包结束后再修改 self，避开借用检查
        if let Some(d) = clicked_date {
            self.selected_date = Some(d);
        }

        ui.separator();

        // 图例
        ui.horizontal(|ui| {
            ui.label("优先级:");
            ui.colored_label(Priority::High.color(), "● 高");
            ui.colored_label(Priority::Medium.color(), "● 中");
            ui.colored_label(Priority::Low.color(), "● 低");
            ui.separator();
            ui.label("任务指示:");
            ui.colored_label(egui::Color32::from_rgb(234, 67, 53), "● 有未完成");
            ui.colored_label(egui::Color32::from_rgb(52, 168, 83), "● 全部完成");
        });
    }
}

// ============================================================
// eframe App 实现
// ============================================================

impl eframe::App for TodoApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::left("task_panel")
            .default_size(300.0)
            .resizable(true)
            .show_inside(ui, |ui| {
                self.render_task_panel(ui);
            });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            self.render_calendar(ui);
        });
    }

    /// egui 0.34 中 on_exit 已无 glow 参数
    fn on_exit(&mut self) {
        self.persist();
    }
}

// ============================================================
// 入口
// ============================================================

fn main() -> eframe::Result<()> {
    let (tasks, year, month) = TodoApp::load();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 650.0])
            .with_min_inner_size([600.0, 450.0]),
        ..Default::default()
    };

    eframe::run_native(
        "日历待办",
        native_options,
        Box::new(move |cc| {
            // 中文字体配置
            configure_fonts(&cc.egui_ctx);
            Ok(Box::new(TodoApp::new(tasks, year, month)))
        }),
    )
}
