use std::sync::Arc;

use eframe::egui::{self, FontData, FontDefinitions, FontFamily};

pub fn configure_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    // 为 Proportional 字体族添加中文字体
    // 放置在最前面：egui 先尝试这个字体
    fonts.font_data.insert(
        "pht".to_owned(),
        Arc::new(FontData::from_owned(
            // include_bytes! 在编译时将文件嵌入二进制
            std::fs::read("assets/fonts/Alibaba-PuHuiTi-Regular.ttf").unwrap_or_default(),
        )),
    );

    // 将中文字体放在 proportional 族的第一位（最高优先级）
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "pht".to_owned());

    // 同理添加等宽中文字体
    // fonts.families
    //     .entry(FontFamily::Monospace)
    //     .or_default()
    //     .insert(0, "noto_sans_mono".to_owned());

    ctx.set_fonts(fonts);
}
