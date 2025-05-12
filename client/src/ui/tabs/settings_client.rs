use crate::context::Context;
use crate::ui::modals::message::MessageModal;
use crate::ui::styles;
use crate::ui::styles::{spacing, themes};
use crate::ui::tabs::Tab;
use crate::{config, logging};
use egui::{Checkbox, DragValue, Grid, RichText, TextEdit};
use log::LevelFilter;
use std::collections::BTreeMap;
use std::sync::LazyLock;
use strum::IntoEnumIterator;

pub struct SettingsClientTab {
    // Fields that taking effect after logout
    compression: bool,

    // Fields that applied after restart
    language: config::Language,
    log_format_choice: String,
    log_level_choice: LevelFilter,

    // Fields that applied by button
    parsed_frames_limit_enabled: bool,
    parsed_frames_limit: usize,
    ping_delay_seconds: i64,
    theme: themes::Preference,
    unparsed_frames_drop: bool,
    unparsed_frames_threshold_enabled: bool,
    unparsed_frames_threshold: usize,
}

type ViewFn = fn(&mut SettingsClientTab, &mut egui::Ui, &mut Context);

static VIEWS: LazyLock<BTreeMap<String, ViewFn>> = LazyLock::new(|| {
    BTreeMap::from([
        (
            t!("Tab.SettingsClient.Label.SaveConfig").to_string(),
            save_client_config_view as ViewFn,
        ),
        (
            t!("Tab.SettingsClient.Label.Compression").to_string(),
            compression_view as ViewFn,
        ),
        (
            t!("Tab.SettingsClient.Label.Language").to_string(),
            language_view as ViewFn,
        ),
        (
            t!("Tab.SettingsClient.Label.LogFormat").to_string(),
            logs_format_view as ViewFn,
        ),
        (
            t!("Tab.SettingsClient.Label.LogLevel").to_string(),
            logs_level_view as ViewFn,
        ),
        (
            t!("Tab.SettingsClient.Label.SyncDelay").to_string(),
            ping_delay_view as ViewFn,
        ),
        (
            t!("Tab.SettingsClient.Label.Theme").to_string(),
            theme_view as ViewFn,
        ),
        (
            t!("Tab.SettingsClient.Label.UnparsedFramesDrop").to_string(),
            unparsed_drop_view as ViewFn,
        ),
        (
            t!("Tab.SettingsClient.Label.ParsedFramesLimit").to_string(),
            parsed_limit_view as ViewFn,
        ),
        (
            t!("Tab.SettingsClient.Label.UnparsedFramesThreshold").to_string(),
            unparsed_threshold_view as ViewFn,
        ),
    ])
});

impl SettingsClientTab {
    pub fn new(ctx: &Context) -> Self {
        Self {
            compression: ctx.client_settings.compression,

            language: ctx.config.language.clone(),
            log_format_choice: ctx.config.log_format.clone(),
            log_level_choice: ctx.config.log_level,

            parsed_frames_limit_enabled: ctx
                .client_settings
                .parsed_frames_limit
                .is_some(),
            parsed_frames_limit: ctx.client_settings.parsed_frames_limit.unwrap_or(0),
            ping_delay_seconds: ctx.client_settings.sync_delay_seconds,
            theme: ctx.client_settings.theme,

            unparsed_frames_drop: ctx.client_settings.unparsed_frames_drop,
            unparsed_frames_threshold_enabled: ctx
                .client_settings
                .unparsed_frames_threshold
                .is_some(),
            unparsed_frames_threshold: ctx
                .client_settings
                .unparsed_frames_threshold
                .unwrap_or(0),
        }
    }
}

impl SettingsClientTab {
    pub fn show_with_header(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        self.tab_heading(ui);

        self.show_content(ui, ctx);
    }

    pub fn show_content(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        const GRID_COLUMNS: usize = 6;

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.add_space(styles::space::SMALL);
                ui.with_layout(
                    egui::Layout::top_down_justified(egui::Align::Center),
                    |ui| {
                        spacing::with_temp_y(ui, spacing::GRID, |ui| {
                            Grid::new("Settings.Grid")
                                .striped(false)
                                .num_columns(GRID_COLUMNS)
                                .show(ui, |ui| {
                                    for (_, view) in VIEWS.iter() {
                                        view(self, ui, ctx);
                                        ui.end_row();
                                    }
                                });
                        });
                    },
                );
            });
    }

    fn tab_heading(&self, ui: &mut egui::Ui) {
        ui.add_space(styles::space::TAB);
        ui.heading(
            RichText::new(Tab::ClientSettings.to_string().as_str())
                .size(styles::heading::HUGE),
        );
    }

    fn option_to_setting(is_enabled: bool, amount: usize) -> Option<usize> {
        if is_enabled { Some(amount) } else { None }
    }
}

fn save_client_config_view(
    _: &mut SettingsClientTab, ui: &mut egui::Ui, ctx: &mut Context,
) {
    ui.add(egui::Label::new(styles::heading::normal(&t!(
        "Tab.SettingsClient.Label.SaveConfig"
    ))));

    styles::invisible(ui);
    styles::invisible(ui);

    if ui
        .button(t!("Button.Save"))
        .on_hover_text(t!("Tab.SettingsClient.Hover.SettingSavesConfig"))
        .clicked()
    {
        // Fields that taking effect after logout
        ctx.config.compression = ctx.client_settings.compression;

        // Fields that applied by button
        ctx.config.parsed_frames_limit = ctx.client_settings.parsed_frames_limit;
        ctx.config.theme = ctx.client_settings.theme;
        ctx.config.sync_delay_seconds = ctx.client_settings.sync_delay_seconds;
        ctx.config.unparsed_frames_drop = ctx.client_settings.unparsed_frames_drop;
        ctx.config.unparsed_frames_threshold =
            ctx.client_settings.unparsed_frames_threshold;

        match ctx.config.save_to_file() {
            Ok(_) => {
                log::info!("Client Settings: Successfully saved client config.");
                MessageModal::info(&t!("Message.Success.ClientConfigSaved"))
                    .try_send_by(&ctx.modals_tx);
            },
            Err(err) => {
                log::error!("Client Settings: Failed to save client config: {}", err);
                MessageModal::error(&format!(
                    "{} {}",
                    t!("Error.FailedSaveClientConfigIntoFile"),
                    err
                ))
                .try_send_by(&ctx.modals_tx);
            },
        };
    }
}

fn compression_view(tab: &mut SettingsClientTab, ui: &mut egui::Ui, ctx: &mut Context) {
    let label = styles::heading::normal(&t!("Tab.SettingsClient.Label.Compression"));
    let not_applied = tab.compression != ctx.client_settings.compression;
    styles::text::field_not_applied(ui, label, not_applied);

    ui.add(Checkbox::without_text(&mut tab.compression));
    styles::invisible(ui);

    if ui
        .button(t!("Button.Apply"))
        .on_hover_text(t!("Tab.SettingsClient.Note.FieldAppliedAfterLogout"))
        .clicked()
    {
        log::info!(
            "Client Settings: Compression changed to {}",
            tab.compression
        );
        ctx.client_settings.compression = tab.compression;
    }
    if ui.button("🔙").clicked() {
        tab.compression = ctx.client_settings.compression;
    }
}

fn language_view(tab: &mut SettingsClientTab, ui: &mut egui::Ui, ctx: &mut Context) {
    let label = styles::heading::normal(&t!("Tab.SettingsClient.Label.Language"));
    let not_applied = tab.language != ctx.config.language;
    styles::text::field_not_applied(ui, label, not_applied);

    styles::invisible(ui);

    ui.with_layout(
        egui::Layout::top_down(egui::Align::Min), |ui| {
            egui::ComboBox::from_label("")
                .selected_text(tab.language.localize()) // Display the currently selected option.
                .show_ui(ui, |ui| {
                    for language in config::Language::iter() {
                        ui.selectable_value(&mut tab.language, language.clone(), language.localize());
                    }
                });
        }
    );

    if ui
        .button(t!("Button.Apply"))
        .on_hover_text(t!("Tab.SettingsClient.Note.FieldAppliedAfterRestart"))
        .clicked()
    {
        log::info!("Client Settings: Language changed to {}", tab.language);
        ctx.config.language = tab.language.clone();
    }

    if ui.button("🔙").clicked() {
        tab.language = ctx.config.language.clone();
    }
}

fn logs_format_view(tab: &mut SettingsClientTab, ui: &mut egui::Ui, ctx: &mut Context) {
    let label = styles::heading::normal(&t!("Tab.SettingsClient.Label.LogFormat"));
    let not_applied = !tab
        .log_format_choice
        .eq_ignore_ascii_case(&ctx.config.log_format);
    styles::text::field_not_applied(ui, label, not_applied);

    styles::invisible(ui);

    ui.add(TextEdit::multiline(&mut tab.log_format_choice));

    if ui
        .button(t!("Button.Apply"))
        .on_hover_text(t!("Tab.SettingsClient.Note.FieldAppliedAfterRestart"))
        .clicked()
    {
        log::info!(
            "Client Settings: Log Format changed to {}",
            tab.log_format_choice
        );
        ctx.config.log_format = tab.log_format_choice.clone();
    }

    if ui.button("🔙").clicked() {
        tab.log_format_choice = ctx.config.log_format.clone();
    }
}

fn logs_level_view(tab: &mut SettingsClientTab, ui: &mut egui::Ui, ctx: &mut Context) {
    let label = styles::heading::normal(&t!("Tab.SettingsClient.Label.LogLevel"));
    let not_applied = tab.log_level_choice != ctx.config.log_level;
    styles::text::field_not_applied(ui, label, not_applied);

    styles::invisible(ui);

    ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
        egui::ComboBox::from_id_salt("Settings.Client.Log.Level.ComboBox")
            .selected_text(logging::localize_log_level(&tab.log_level_choice))
            .show_ui(ui, |ui| {
                for level_filter in LevelFilter::iter() {
                    ui.selectable_value(
                        &mut tab.log_level_choice,
                        level_filter,
                        logging::localize_log_level(&level_filter),
                    );
                }
            });
    });

    if ui
        .button(t!("Button.Apply"))
        .on_hover_text(t!("Tab.SettingsClient.Note.FieldAppliedAfterRestart"))
        .clicked()
    {
        log::info!("Client Settings: Log Level changed to {}", tab.language);
        ctx.config.log_level = tab.log_level_choice;
    }

    if ui.button("🔙").clicked() {
        tab.log_level_choice = ctx.config.log_level;
    }
}

fn ping_delay_view(tab: &mut SettingsClientTab, ui: &mut egui::Ui, ctx: &mut Context) {
    let label = styles::heading::normal(&t!("Tab.SettingsClient.Label.SyncDelay"));
    let not_applied = tab.ping_delay_seconds != ctx.client_settings.sync_delay_seconds;
    styles::text::field_not_applied(ui, label, not_applied);

    styles::invisible(ui);

    ui.add(
        DragValue::new(&mut tab.ping_delay_seconds)
            .speed(1)
            .range(1..=i64::MAX)
            .suffix(format!(" {}", t!("Tab.SettingsClient.Suffix.SyncDelay"))),
    );

    if ui
        .button(t!("Button.Apply"))
        .on_hover_text(t!("Tab.SettingsClient.Note.FieldAppliedImmediately"))
        .clicked()
    {
        log::info!(
            "Client Settings: Sync Delay seconds changed to {}",
            tab.ping_delay_seconds
        );
        ctx.client_settings.sync_delay_seconds = tab.ping_delay_seconds;
    }

    if ui.button("🔙").clicked() {
        tab.ping_delay_seconds = ctx.client_settings.sync_delay_seconds;
    }
}

fn theme_view(tab: &mut SettingsClientTab, ui: &mut egui::Ui, ctx: &mut Context) {
    let label = styles::heading::normal(&t!("Tab.SettingsClient.Label.Theme"));
    let not_applied = tab.theme != ctx.client_settings.theme;
    styles::text::field_not_applied(ui, label, not_applied);

    styles::invisible(ui);

    ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
        egui::ComboBox::from_id_salt("Settings.Theme.ComboBox")
            .width(200.0)
            .selected_text(tab.theme.title())
            .show_ui(ui, |ui| {
                for theme in themes::Preference::iter() {
                    ui.selectable_value(&mut tab.theme, theme, theme.title());
                }
            });
    });

    if ui
        .button(t!("Button.Apply"))
        .on_hover_text(t!("Tab.SettingsClient.Note.FieldAppliedImmediately"))
        .clicked()
    {
        ctx.client_settings.theme = tab.theme;
        log::info!("Client Settings: Theme changed to {}", tab.theme.title());
        ui.ctx()
            .set_style(tab.theme.into_aesthetix_theme().custom_style());
    }

    if ui.button("🔙").clicked() {
        tab.theme = ctx.client_settings.theme;
    }
}

fn unparsed_drop_view(tab: &mut SettingsClientTab, ui: &mut egui::Ui, ctx: &mut Context) {
    let label =
        styles::heading::normal(&t!("Tab.SettingsClient.Label.UnparsedFramesDrop"));
    let not_applied =
        tab.unparsed_frames_drop != ctx.client_settings.unparsed_frames_drop;
    styles::text::field_not_applied(ui, label, not_applied);

    ui.add(Checkbox::without_text(&mut tab.unparsed_frames_drop));

    styles::invisible(ui);

    if ui
        .button(t!("Button.Apply"))
        .on_hover_text(t!("Tab.SettingsClient.Label.UnparsedFramesDrop.Note"))
        .on_hover_text(t!("Tab.SettingsClient.Note.FieldAppliedImmediately"))
        .clicked()
    {
        log::info!(
            "Client Settings: `Drop Unparsed Frames` changed to {}",
            tab.unparsed_frames_drop
        );
        ctx.client_settings.unparsed_frames_drop = tab.unparsed_frames_drop;
    }
    if ui.button("🔙").clicked() {
        tab.unparsed_frames_drop = ctx.client_settings.unparsed_frames_drop;
    }
}

fn parsed_limit_view(tab: &mut SettingsClientTab, ui: &mut egui::Ui, ctx: &mut Context) {
    let setting = SettingsClientTab::option_to_setting(
        tab.parsed_frames_limit_enabled,
        tab.parsed_frames_limit,
    );
    let label =
        styles::heading::normal(&t!("Tab.SettingsClient.Label.ParsedFramesLimit"));
    let not_applied = setting != ctx.client_settings.parsed_frames_limit;
    styles::text::field_not_applied(ui, label, not_applied);

    ui.add(Checkbox::without_text(&mut tab.parsed_frames_limit_enabled));
    ui.add_enabled(
        tab.parsed_frames_limit_enabled,
        DragValue::new(&mut tab.parsed_frames_limit)
            .speed(1)
            .range(1..=i64::MAX)
            .suffix(format!(" {}", t!("Tab.SettingsClient.Suffix.Frames"))),
    );

    if ui
        .button(t!("Button.Apply"))
        .on_hover_text(t!("Tab.SettingsClient.Label.ParsedFramesLimit.Note"))
        .on_hover_text(t!("Tab.SettingsClient.Note.FieldAppliedImmediately"))
        .clicked()
    {
        log::info!(
            "Client Settings: `Parsed Frames Limit` changed to {}:{}",
            tab.parsed_frames_limit_enabled,
            tab.parsed_frames_limit,
        );
        ctx.client_settings.parsed_frames_limit = setting;
    }
    if ui.button("🔙").clicked() {
        tab.parsed_frames_limit_enabled =
            ctx.client_settings.parsed_frames_limit.is_some();
        tab.parsed_frames_limit = ctx.client_settings.parsed_frames_limit.unwrap_or(0);
    }
}

fn unparsed_threshold_view(
    tab: &mut SettingsClientTab, ui: &mut egui::Ui, ctx: &mut Context,
) {
    let setting = SettingsClientTab::option_to_setting(
        tab.unparsed_frames_threshold_enabled,
        tab.unparsed_frames_threshold,
    );
    let label =
        styles::heading::normal(&t!("Tab.SettingsClient.Label.UnparsedFramesThreshold"));
    let not_applied = setting != ctx.client_settings.unparsed_frames_threshold;
    styles::text::field_not_applied(ui, label, not_applied);

    ui.add(Checkbox::without_text(
        &mut tab.unparsed_frames_threshold_enabled,
    ));
    ui.add_enabled(
        tab.unparsed_frames_threshold_enabled,
        DragValue::new(&mut tab.unparsed_frames_threshold)
            .speed(1)
            .range(1..=i64::MAX)
            .suffix(format!(" {}", t!("Tab.SettingsClient.Suffix.Frames"))),
    );

    if ui
        .button(t!("Button.Apply"))
        .on_hover_text(t!("Tab.SettingsClient.Label.UnparsedFramesThreshold.Note"))
        .on_hover_text(t!("Tab.SettingsClient.Note.FieldAppliedImmediately"))
        .clicked()
    {
        log::info!(
            "Client Settings: `Unparsed Frames Threshold` changed to {}:{}",
            tab.unparsed_frames_threshold_enabled,
            tab.unparsed_frames_threshold,
        );
        ctx.client_settings.unparsed_frames_threshold = setting;
        ctx.net_storage.raw.set_threshold(setting);
    }
    if ui.button("🔙").clicked() {
        tab.unparsed_frames_threshold_enabled =
            ctx.client_settings.unparsed_frames_threshold.is_some();
        tab.unparsed_frames_threshold =
            ctx.client_settings.unparsed_frames_threshold.unwrap_or(0);
    }
}
